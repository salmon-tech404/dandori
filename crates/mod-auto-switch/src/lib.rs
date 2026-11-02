//! Module `auto-switch`: Tự động chuyển tài khoản khi hết hạn mức (P2-10).
//!
//! Tự động chuyển đổi tài khoản agent cùng nhà cung cấp khi xảy ra lỗi `QuotaExhausted`,
//! bảo toàn worktree và prompt, kiểm soát số lần chuyển tối đa chống vòng lặp,
//! và chuyển sang trạng thái `needs_human` khi hết tài khoản hoặc vượt giới hạn.

#![warn(missing_docs)]

use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, put};
use axum::{Json, Router};
use chrono::Utc;
use dandori_module_api::{
    AgentRole, EventBus, EventFilter, Manifest, Migration, Module, ModuleContext,
    ModuleSubscription, TaskRunner, WorkerSpec, can_access_workspace, can_administer_workspace,
    project_workspace, session_user, task_workspace,
};
use dandori_types::{
    AgentAccountId, Envelope, EventPayload, EventScope, ProjectId, TaskId, TaskStatus, UserId,
    WorkspaceId,
};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::sync::{Arc, LazyLock};
use tokio_util::sync::CancellationToken;

#[expect(
    clippy::expect_used,
    reason = "embedded static module.json validated at build/test time"
)]
static MANIFEST: LazyLock<Manifest> = LazyLock::new(|| {
    let json = include_str!("../module.json");
    serde_json::from_str(json).expect("module.json của mod-auto-switch phải hợp lệ")
});

static MIGRATIONS: [Migration; 1] = [Migration {
    id: "0001_auto_switch",
    sql: include_str!("../migrations/0001_auto_switch.sql"),
}];

/// Cấu hình tự chuyển tài khoản cho một dự án.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSwitchConfig {
    /// Định danh dự án.
    pub project_id: String,
    /// Số lần chuyển tối đa cho mỗi task trước khi dừng và báo needs_human (mặc định 2).
    pub max_switches: i64,
    /// Có cho phép chuyển sang nhà cung cấp khác không (mặc định false).
    pub allow_cross_provider: bool,
    /// Danh sách các nhà cung cấp được phép chuyển chéo nếu bật (JSON array).
    pub allowed_providers: Vec<String>,
}

/// Dữ liệu cập nhật cấu hình tự chuyển tài khoản.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAutoSwitchConfigPayload {
    /// Số lần chuyển tối đa cho mỗi task.
    pub max_switches: Option<i64>,
    /// Cho phép chuyển chéo nhà cung cấp hay không.
    pub allow_cross_provider: Option<bool>,
    /// Danh sách nhà cung cấp cho phép chuyển chéo.
    pub allowed_providers: Option<Vec<String>>,
}

/// Bản ghi lịch sử chuyển tài khoản của một task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSwitchHistoryItem {
    /// ID bản ghi lịch sử.
    pub id: String,
    /// ID nhiệm vụ.
    pub task_id: String,
    /// ID tài khoản cũ bị thay thế.
    pub from_account_id: String,
    /// ID tài khoản mới được chọn.
    pub to_account_id: String,
    /// Nhà cung cấp cũ.
    pub from_provider: String,
    /// Nhà cung cấp mới.
    pub to_provider: String,
    /// Lý do chuyển (ví dụ quota_exhausted).
    pub reason: String,
    /// Thứ tự lần chuyển trên task này (1, 2, ...).
    pub switch_count: i64,
    /// Thời điểm chuyển (ISO 8601).
    pub created_at: String,
}

/// Thông tin ứng viên tài khoản agent khả dụng.
#[derive(Debug, Clone)]
struct CandidateAccount {
    id: String,
    provider: String,
}

/// Trạng thái chia sẻ nội bộ cho các route và worker của module auto-switch.
#[derive(Clone)]
struct ModuleState {
    pool: Option<SqlitePool>,
    #[expect(dead_code, reason = "Giữ event_bus trong state")]
    event_bus: Arc<EventBus>,
    #[expect(dead_code, reason = "Giữ task_runner trong state")]
    task_runner: Option<Arc<dyn TaskRunner>>,
}

/// Struct đại diện cho module auto-switch.
#[derive(Debug, Default, Clone, Copy)]
pub struct AutoSwitchModule;

impl AutoSwitchModule {
    /// Khởi tạo một thực thể AutoSwitchModule mới.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for AutoSwitchModule {
    fn manifest(&self) -> &Manifest {
        &MANIFEST
    }

    fn migrations(&self) -> &'static [Migration] {
        &MIGRATIONS
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        let state = Arc::new(ModuleState {
            pool: ctx.pool().cloned(),
            event_bus: Arc::clone(ctx.event_bus()),
            task_runner: ctx.task_runner().cloned(),
        });

        Router::new()
            .route("/projects/{project_id}/config", get(get_project_config))
            .route("/projects/{project_id}/config", put(update_project_config))
            .route("/tasks/{task_id}/history", get(get_task_history))
            .with_state(state)
    }

    fn workers(&self, ctx: &ModuleContext) -> Vec<WorkerSpec> {
        let pool = ctx.pool().cloned();
        let event_bus = Arc::clone(ctx.event_bus());
        let runner = ctx.task_runner().cloned();

        vec![WorkerSpec::new(
            "auto_switch_worker",
            move |cancel_token: CancellationToken| {
                let pool = pool.clone();
                let event_bus = Arc::clone(&event_bus);
                let runner = runner.clone();
                Box::pin(async move {
                    run_auto_switch_worker(pool, event_bus, runner, cancel_token).await;
                })
            },
        )]
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }
}

async fn run_auto_switch_worker(
    pool: Option<SqlitePool>,
    event_bus: Arc<EventBus>,
    runner: Option<Arc<dyn TaskRunner>>,
    cancel_token: CancellationToken,
) {
    let mut rx = event_bus.subscribe(EventFilter::all());
    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                tracing::debug!("auto_switch_worker dừng êm theo token hủy");
                break;
            }
            envelope = rx.recv() => {
                let Some(env) = envelope else { break; };
                if let Some(ref p) = pool {
                    check_and_handle_run_failed(p, runner.as_ref(), &event_bus, &env).await;
                }
            }
        }
    }
}

async fn check_and_handle_run_failed(
    pool: &SqlitePool,
    runner: Option<&Arc<dyn TaskRunner>>,
    event_bus: &EventBus,
    env: &Envelope,
) {
    if let EventPayload::RunFailed {
        ref run_id,
        ref error_message,
    } = env.payload
        && (error_message.contains("Hết hạn mức") || error_message.contains("quota_exhausted"))
    {
        handle_quota_exhausted(pool, runner, event_bus, &env.scope, run_id).await;
    }
}

async fn handle_quota_exhausted(
    pool: &SqlitePool,
    runner: Option<&Arc<dyn TaskRunner>>,
    event_bus: &EventBus,
    scope: &EventScope,
    run_id: &dandori_types::RunId,
) {
    let (Some(ws_id), Some(prj_id), Some(t_id)) =
        (scope.workspace_id, scope.project_id, scope.task_id)
    else {
        return;
    };

    let (old_acc_id, old_provider) = match get_failed_run_account(pool, run_id).await {
        Some(pair) => pair,
        None => {
            set_task_needs_human(pool, event_bus, ws_id, prj_id, t_id).await;
            return;
        }
    };

    let config = get_project_auto_switch_config(pool, &prj_id.to_string()).await;
    let switch_count = get_task_switch_count(pool, &t_id.to_string()).await;

    if switch_count >= config.max_switches {
        set_task_needs_human(pool, event_bus, ws_id, prj_id, t_id).await;
        return;
    }

    let excluded_ids = get_excluded_account_ids(pool, &t_id.to_string(), &old_acc_id).await;
    let candidate = find_candidate_account(pool, &old_provider, &config, &excluded_ids).await;

    let Some(candidate_acc) = candidate else {
        set_task_needs_human(pool, event_bus, ws_id, prj_id, t_id).await;
        return;
    };

    record_switch_and_dispatch(
        pool,
        runner,
        ws_id,
        t_id,
        &old_acc_id,
        &candidate_acc,
        switch_count + 1,
        excluded_ids,
    )
    .await;
}

async fn get_failed_run_account(
    pool: &SqlitePool,
    run_id: &dandori_types::RunId,
) -> Option<(String, String)> {
    let run_row = sqlx::query("SELECT account FROM run WHERE id = ?")
        .bind(run_id.to_string())
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()?;

    let account_name: String = run_row.get("account");
    let acc_row = sqlx::query("SELECT id, provider FROM agent_account WHERE id = ? OR label = ?")
        .bind(&account_name)
        .bind(&account_name)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()?;

    let id: String = acc_row.get("id");
    let provider: String = acc_row.get("provider");
    Some((id, provider))
}

async fn get_task_switch_count(pool: &SqlitePool, task_id: &str) -> i64 {
    sqlx::query_as("SELECT count(*) FROM m_auto_switch_history WHERE task_id = ?")
        .bind(task_id)
        .fetch_one(pool)
        .await
        .map(|r: (i64,)| r.0)
        .unwrap_or(0)
}

async fn get_excluded_account_ids(
    pool: &SqlitePool,
    task_id: &str,
    current_acc_id: &str,
) -> Vec<String> {
    let mut ids = vec![current_acc_id.to_string()];
    let rows = sqlx::query("SELECT from_account_id FROM m_auto_switch_history WHERE task_id = ?")
        .bind(task_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

    for row in rows {
        let from_id: String = row.get("from_account_id");
        if !ids.contains(&from_id) {
            ids.push(from_id);
        }
    }
    ids
}

async fn find_candidate_account(
    pool: &SqlitePool,
    old_provider: &str,
    config: &AutoSwitchConfig,
    excluded_ids: &[String],
) -> Option<CandidateAccount> {
    let now = Utc::now().to_rfc3339();

    // Lấy toàn bộ tài khoản enabled = 1 và auth_type = 'cli'
    let rows = sqlx::query(
        "SELECT id, provider, priority, exhausted_until FROM agent_account \
         WHERE enabled = 1 AND auth_type = 'cli' \
         ORDER BY priority ASC, id ASC",
    )
    .fetch_all(pool)
    .await
    .ok()?;

    for row in rows {
        let id: String = row.get("id");
        let provider: String = row.get("provider");
        let exhausted_until: Option<String> = row.get("exhausted_until");

        if excluded_ids.contains(&id) {
            continue;
        }

        if exhausted_until.as_deref().unwrap_or("") > now.as_str() {
            continue;
        }

        // Kiểm tra quy tắc cùng nhà hay chuyển chéo
        if !config.allow_cross_provider {
            if provider == old_provider {
                return Some(CandidateAccount { id, provider });
            }
        } else if provider == old_provider || config.allowed_providers.contains(&provider) {
            return Some(CandidateAccount { id, provider });
        }
    }

    None
}

#[expect(
    clippy::too_many_arguments,
    reason = "Hàm điều phối chuyển tài khoản cần nhận đủ thông tin để ghi nhận lịch sử và kích hoạt lần chạy mới"
)]
async fn record_switch_and_dispatch(
    pool: &SqlitePool,
    runner: Option<&Arc<dyn TaskRunner>>,
    ws_id: WorkspaceId,
    t_id: TaskId,
    old_acc_id: &str,
    candidate_acc: &CandidateAccount,
    switch_count: i64,
    excluded_ids: Vec<String>,
) {
    let history_id = uuid::Uuid::new_v4().to_string();
    let now_str = Utc::now().to_rfc3339();

    let _ = sqlx::query(
        r#"
        INSERT INTO m_auto_switch_history (id, task_id, from_account_id, to_account_id, from_provider, to_provider, reason, switch_count, created_at)
        VALUES (?, ?, ?, ?, ?, ?, 'quota_exhausted', ?, ?)
        "#,
    )
    .bind(&history_id)
    .bind(t_id.to_string())
    .bind(old_acc_id)
    .bind(&candidate_acc.id)
    .bind(&candidate_acc.provider)
    .bind(&candidate_acc.provider)
    .bind(switch_count)
    .bind(&now_str)
    .execute(pool)
    .await;

    if let Some(runner) = runner {
        let user_id = get_workspace_user_id(pool, &ws_id.to_string()).await;
        let exclude_accs: Vec<AgentAccountId> = excluded_ids
            .iter()
            .filter_map(|s| s.parse::<AgentAccountId>().ok())
            .collect();

        let _ = runner
            .start_run(t_id, AgentRole::Worker, user_id, exclude_accs)
            .await;
    }
}

async fn set_task_needs_human(
    pool: &SqlitePool,
    event_bus: &EventBus,
    ws_id: WorkspaceId,
    prj_id: ProjectId,
    t_id: TaskId,
) {
    let now = Utc::now().to_rfc3339();
    let _ = sqlx::query("UPDATE task SET status = 'needs_human', updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(t_id.to_string())
        .execute(pool)
        .await;

    event_bus.publish(Envelope::new(
        Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::TaskStatusChanged {
            task_id: t_id,
            old_status: TaskStatus::Running.as_str().to_string(),
            new_status: TaskStatus::NeedsHuman.as_str().to_string(),
        },
    ));
}

async fn get_workspace_user_id(pool: &SqlitePool, ws_str: &str) -> UserId {
    let user_row = sqlx::query(
        "SELECT user_id FROM workspace_member WHERE workspace_id = ? ORDER BY role LIMIT 1",
    )
    .bind(ws_str)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    user_row
        .and_then(|r| r.try_get::<String, _>("user_id").ok())
        .and_then(|s| s.parse::<UserId>().ok())
        .unwrap_or_else(UserId::new)
}

async fn get_project_auto_switch_config(pool: &SqlitePool, project_id: &str) -> AutoSwitchConfig {
    let row = sqlx::query(
        "SELECT max_switches, allow_cross_provider, allowed_providers FROM m_auto_switch_config WHERE project_id = ?",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    match row {
        Some(r) => {
            let max_switches: i64 = r.get("max_switches");
            let cross: i64 = r.get("allow_cross_provider");
            let allowed_str: String = r.get("allowed_providers");
            let allowed: Vec<String> = serde_json::from_str(&allowed_str).unwrap_or_default();
            AutoSwitchConfig {
                project_id: project_id.to_string(),
                max_switches,
                allow_cross_provider: cross == 1,
                allowed_providers: allowed,
            }
        }
        None => AutoSwitchConfig {
            project_id: project_id.to_string(),
            max_switches: 2,
            allow_cross_provider: false,
            allowed_providers: vec![],
        },
    }
}

/// Yêu cầu quyền trên workspace của dự án (`write` = cần quyền quản trị workspace).
async fn authorize_project(
    pool: &SqlitePool,
    headers: &HeaderMap,
    project_id: &str,
    write: bool,
) -> Result<(), Box<axum::response::Response>> {
    let Some(user) = session_user(pool, headers).await else {
        return Err(Box::new(
            (StatusCode::UNAUTHORIZED, "Chưa đăng nhập").into_response(),
        ));
    };
    let Some(ws) = project_workspace(pool, project_id).await else {
        return Err(Box::new(
            (StatusCode::NOT_FOUND, "Không tìm thấy dự án").into_response(),
        ));
    };
    let allowed = if write {
        can_administer_workspace(pool, &user, &ws).await
    } else {
        can_access_workspace(pool, &user, &ws).await
    };
    if allowed {
        Ok(())
    } else {
        Err(Box::new(
            (StatusCode::FORBIDDEN, "Không có quyền với dự án này").into_response(),
        ))
    }
}

/// Yêu cầu quyền xem trên workspace của task.
async fn authorize_task(
    pool: &SqlitePool,
    headers: &HeaderMap,
    task_id: &str,
) -> Result<(), Box<axum::response::Response>> {
    let Some(user) = session_user(pool, headers).await else {
        return Err(Box::new(
            (StatusCode::UNAUTHORIZED, "Chưa đăng nhập").into_response(),
        ));
    };
    let Some(ws) = task_workspace(pool, task_id).await else {
        return Err(Box::new(
            (StatusCode::NOT_FOUND, "Không tìm thấy task").into_response(),
        ));
    };
    if can_access_workspace(pool, &user, &ws).await {
        Ok(())
    } else {
        Err(Box::new(
            (StatusCode::FORBIDDEN, "Không có quyền với task này").into_response(),
        ))
    }
}

async fn get_project_config(
    State(state): State<Arc<ModuleState>>,
    headers: HeaderMap,
    AxumPath(project_id): AxumPath<String>,
) -> impl IntoResponse {
    let Some(ref pool) = state.pool else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing").into_response();
    };

    if let Err(resp) = authorize_project(pool, &headers, &project_id, false).await {
        return *resp;
    }

    let config = get_project_auto_switch_config(pool, &project_id).await;
    Json(config).into_response()
}

async fn update_project_config(
    State(state): State<Arc<ModuleState>>,
    headers: HeaderMap,
    AxumPath(project_id): AxumPath<String>,
    Json(payload): Json<UpdateAutoSwitchConfigPayload>,
) -> impl IntoResponse {
    let Some(ref pool) = state.pool else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing").into_response();
    };

    if let Err(resp) = authorize_project(pool, &headers, &project_id, true).await {
        return *resp;
    }

    let max_switches = payload.max_switches.unwrap_or(2);
    let cross = if payload.allow_cross_provider.unwrap_or(false) {
        1
    } else {
        0
    };
    let allowed = serde_json::to_string(&payload.allowed_providers.unwrap_or_default())
        .unwrap_or_else(|_| "[]".to_string());
    let now = Utc::now().to_rfc3339();

    let res = sqlx::query(
        r#"
        INSERT INTO m_auto_switch_config (project_id, max_switches, allow_cross_provider, allowed_providers, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(project_id) DO UPDATE SET
            max_switches = excluded.max_switches,
            allow_cross_provider = excluded.allow_cross_provider,
            allowed_providers = excluded.allowed_providers,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(&project_id)
    .bind(max_switches)
    .bind(cross)
    .bind(allowed)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await;

    match res {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_task_history(
    State(state): State<Arc<ModuleState>>,
    headers: HeaderMap,
    AxumPath(task_id): AxumPath<String>,
) -> impl IntoResponse {
    let Some(ref pool) = state.pool else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing").into_response();
    };

    if let Err(resp) = authorize_task(pool, &headers, &task_id).await {
        return *resp;
    }

    let rows = match sqlx::query(
        "SELECT id, task_id, from_account_id, to_account_id, from_provider, to_provider, reason, switch_count, created_at \
         FROM m_auto_switch_history WHERE task_id = ? ORDER BY switch_count ASC",
    )
    .bind(&task_id)
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let items: Vec<AutoSwitchHistoryItem> = rows
        .into_iter()
        .map(|r| AutoSwitchHistoryItem {
            id: r.get("id"),
            task_id: r.get("task_id"),
            from_account_id: r.get("from_account_id"),
            to_account_id: r.get("to_account_id"),
            from_provider: r.get("from_provider"),
            to_provider: r.get("to_provider"),
            reason: r.get("reason"),
            switch_count: r.get("switch_count"),
            created_at: r.get("created_at"),
        })
        .collect();

    Json(items).into_response()
}

//! Module phân công Lead và Worker cho từng task theo vai trò và dự án.

#![warn(missing_docs)]

use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use dandori_module_api::{
    Manifest, Migration, Module, ModuleContext, ModuleSubscription, WorkerSpec,
};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::sync::{Arc, LazyLock};

#[expect(
    clippy::expect_used,
    reason = "embedded static module.json validated at build/test time"
)]
static MANIFEST: LazyLock<Manifest> = LazyLock::new(|| {
    let json = include_str!("../module.json");
    serde_json::from_str(json).expect("module.json của mod-lead-worker phải hợp lệ")
});

static MIGRATIONS: [Migration; 1] = [Migration {
    id: "0001_lead_worker_schema",
    sql: "CREATE TABLE IF NOT EXISTS m_lead_worker_default (\
            project_id TEXT NOT NULL,\
            role TEXT NOT NULL,\
            provider_order TEXT NOT NULL,\
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),\
            PRIMARY KEY (project_id, role)\
          );",
}];

/// Trạng thái chia sẻ nội bộ cho các route của module lead-worker.
#[derive(Clone, Debug)]
struct ModuleState {
    pool: Option<SqlitePool>,
}

/// Struct đại diện cho module lead-worker.
#[derive(Debug, Default, Clone, Copy)]
pub struct LeadWorkerModule;

impl LeadWorkerModule {
    /// Khởi tạo một thực thể LeadWorkerModule mới.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for LeadWorkerModule {
    fn manifest(&self) -> &Manifest {
        &MANIFEST
    }

    fn migrations(&self) -> &'static [Migration] {
        &MIGRATIONS
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        let state = Arc::new(ModuleState {
            pool: ctx.pool().cloned(),
        });

        Router::new()
            .route(
                "/projects/{project_id}/defaults",
                get(get_project_defaults).put(update_project_defaults),
            )
            .route("/tasks/{task_id}/assignment", get(get_task_assignment))
            .route("/tasks/{task_id}/dispatch", post(dispatch_task))
            .with_state(state)
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        Vec::new()
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }
}

/// Cấu hình mặc định cho một vai trò.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDefaultItem {
    /// Vai trò (`pm`, `worker`, `test`, `doc`).
    pub role: String,
    /// Danh sách nhà cung cấp theo thứ tự ưu tiên.
    pub provider_order: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GetDefaultsResponse {
    defaults: Vec<RoleDefaultItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateDefaultsRequest {
    defaults: Vec<RoleDefaultItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskAssignmentResponse {
    task_id: String,
    effective_lead_agent: String,
    effective_worker_agent: String,
    task_lead_agent: String,
    task_worker_agent: String,
    is_lead_overridden: bool,
    is_worker_overridden: bool,
    ready_providers: Vec<String>,
}

fn get_standard_default_orders() -> Vec<RoleDefaultItem> {
    vec![
        RoleDefaultItem {
            role: "pm".to_string(),
            provider_order: vec!["claude".into(), "gemini".into(), "codex".into()],
        },
        RoleDefaultItem {
            role: "worker".to_string(),
            provider_order: vec!["gemini".into(), "claude".into(), "codex".into()],
        },
        RoleDefaultItem {
            role: "test".to_string(),
            provider_order: vec!["codex".into(), "gemini".into(), "claude".into()],
        },
        RoleDefaultItem {
            role: "doc".to_string(),
            provider_order: vec!["claude".into(), "gemini".into(), "codex".into()],
        },
    ]
}

fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    let cookie_val = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    for pair in cookie_val.split(';') {
        let mut parts = pair.trim().splitn(2, '=');
        let key = parts.next()?.trim();
        let val = parts.next()?.trim();
        if key == "dandori_session" && !val.is_empty() {
            return Some(val.to_string());
        }
    }
    None
}

async fn get_user_id_from_headers(pool: &SqlitePool, headers: &HeaderMap) -> Option<String> {
    let token = extract_session_token(headers)?;
    let hash = dandori_module_api::hash_session_token(&token);
    let row = sqlx::query(
        "SELECT user_id FROM session WHERE token_hash = ? AND expires_at > datetime('now')",
    )
    .bind(hash)
    .fetch_optional(pool)
    .await
    .ok()??;
    row.try_get("user_id").ok()
}

async fn check_project_access(
    pool: &SqlitePool,
    headers: &HeaderMap,
    project_id: &str,
) -> Result<String, (StatusCode, &'static str)> {
    let user_id = get_user_id_from_headers(pool, headers)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Chưa xác thực"))?;

    let proj_row = sqlx::query("SELECT workspace_id FROM project WHERE id = ?")
        .bind(project_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi truy vấn dự án"))?
        .ok_or((StatusCode::NOT_FOUND, "Dự án không tồn tại"))?;

    let ws_id: String = proj_row.try_get("workspace_id").map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Lỗi đọc dữ liệu workspace",
        )
    })?;

    // Kiểm tra admin hệ thống
    let user_row = sqlx::query("SELECT system_role FROM user WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi truy vấn người dùng"))?;

    if let Some(ur) = user_row {
        let role: String = ur.try_get("system_role").unwrap_or_default();
        if role == "admin" {
            return Ok(user_id);
        }
    }

    // Kiểm tra thành viên workspace
    let member_row =
        sqlx::query("SELECT role FROM workspace_member WHERE workspace_id = ? AND user_id = ?")
            .bind(&ws_id)
            .bind(&user_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi kiểm tra thành viên"))?;

    if member_row.is_some() {
        Ok(user_id)
    } else {
        Err((
            StatusCode::FORBIDDEN,
            "Bạn không có quyền truy cập vào dự án này",
        ))
    }
}

async fn check_task_access(
    pool: &SqlitePool,
    headers: &HeaderMap,
    task_id: &str,
) -> Result<(String, String), (StatusCode, &'static str)> {
    let task_row = sqlx::query("SELECT project_id FROM task WHERE id = ?")
        .bind(task_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi truy vấn task"))?
        .ok_or((StatusCode::NOT_FOUND, "Task không tồn tại"))?;

    let project_id: String = task_row
        .try_get("project_id")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi đọc project_id"))?;

    let user_id = check_project_access(pool, headers, &project_id).await?;
    Ok((user_id, project_id))
}

async fn get_project_defaults(
    State(state): State<Arc<ModuleState>>,
    headers: HeaderMap,
    AxumPath(project_id): AxumPath<String>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "DB not ready"})),
            )
                .into_response();
        }
    };

    if let Err((status, msg)) = check_project_access(pool, &headers, &project_id).await {
        return (status, Json(serde_json::json!({"error": msg}))).into_response();
    }

    let rows = match sqlx::query(
        "SELECT role, provider_order FROM m_lead_worker_default WHERE project_id = ?",
    )
    .bind(&project_id)
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    let standards = get_standard_default_orders();
    let mut defaults = Vec::new();

    for std_item in standards {
        if let Some(r) = rows.iter().find(|row| {
            let role: String = row.try_get("role").unwrap_or_default();
            role == std_item.role
        }) {
            let order_json: String = r.try_get("provider_order").unwrap_or_default();
            let parsed_order: Vec<String> =
                serde_json::from_str(&order_json).unwrap_or(std_item.provider_order);
            defaults.push(RoleDefaultItem {
                role: std_item.role,
                provider_order: parsed_order,
            });
        } else {
            defaults.push(std_item);
        }
    }

    Json(GetDefaultsResponse { defaults }).into_response()
}

async fn update_project_defaults(
    State(state): State<Arc<ModuleState>>,
    headers: HeaderMap,
    AxumPath(project_id): AxumPath<String>,
    Json(payload): Json<UpdateDefaultsRequest>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "DB not ready"})),
            )
                .into_response();
        }
    };

    if let Err((status, msg)) = check_project_access(pool, &headers, &project_id).await {
        return (status, Json(serde_json::json!({"error": msg}))).into_response();
    }

    let now_str = chrono::Utc::now().to_rfc3339();

    for item in &payload.defaults {
        let order_json = serde_json::to_string(&item.provider_order).unwrap_or_default();
        let _ = sqlx::query(
            r#"
            INSERT INTO m_lead_worker_default (project_id, role, provider_order, updated_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT (project_id, role) DO UPDATE SET
                provider_order = excluded.provider_order,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&project_id)
        .bind(&item.role)
        .bind(order_json)
        .bind(&now_str)
        .execute(pool)
        .await;
    }

    Json(GetDefaultsResponse {
        defaults: payload.defaults,
    })
    .into_response()
}

#[expect(
    clippy::too_many_lines,
    reason = "Tính toán assignment kết hợp default và override"
)]
async fn get_task_assignment(
    State(state): State<Arc<ModuleState>>,
    headers: HeaderMap,
    AxumPath(task_id): AxumPath<String>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "DB not ready"})),
            )
                .into_response();
        }
    };

    let (_, project_id) = match check_task_access(pool, &headers, &task_id).await {
        Ok(res) => res,
        Err((status, msg)) => {
            return (status, Json(serde_json::json!({"error": msg}))).into_response();
        }
    };

    let task_row = match sqlx::query("SELECT lead_agent, worker_agent FROM task WHERE id = ?")
        .bind(&task_id)
        .fetch_one(pool)
        .await
    {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Task không tồn tại"})),
            )
                .into_response();
        }
    };

    let task_lead_agent: String = task_row.try_get("lead_agent").unwrap_or_default();
    let task_worker_agent: String = task_row.try_get("worker_agent").unwrap_or_default();

    // Lấy danh sách nhà cung cấp ready
    let acc_rows =
        sqlx::query("SELECT provider FROM agent_account WHERE enabled = 1 AND status = 'ready'")
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    let mut ready_providers: Vec<String> = Vec::new();
    for r in acc_rows {
        let p: String = r.try_get("provider").unwrap_or_default();
        if !ready_providers.contains(&p) {
            ready_providers.push(p);
        }
    }

    // Đọc project defaults
    let def_rows =
        sqlx::query("SELECT role, provider_order FROM m_lead_worker_default WHERE project_id = ?")
            .bind(&project_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    let standards = get_standard_default_orders();
    let mut pm_order = vec![
        "claude".to_string(),
        "gemini".to_string(),
        "codex".to_string(),
    ];
    let mut worker_order = vec![
        "gemini".to_string(),
        "claude".to_string(),
        "codex".to_string(),
    ];

    for r in def_rows {
        let role: String = r.try_get("role").unwrap_or_default();
        let order_json: String = r.try_get("provider_order").unwrap_or_default();
        if let Ok(orders) = serde_json::from_str::<Vec<String>>(&order_json) {
            if role == "pm" {
                pm_order = orders;
            } else if role == "worker" {
                worker_order = orders;
            }
        }
    }
    if pm_order.is_empty() {
        pm_order = standards
            .first()
            .map(|s| s.provider_order.clone())
            .unwrap_or_default();
    }
    if worker_order.is_empty() {
        worker_order = standards
            .get(1)
            .map(|s| s.provider_order.clone())
            .unwrap_or_default();
    }

    let is_lead_overridden = !task_lead_agent.is_empty();
    let effective_lead_agent = if is_lead_overridden {
        task_lead_agent.clone()
    } else {
        pm_order
            .iter()
            .find(|p| ready_providers.contains(p))
            .cloned()
            .unwrap_or_else(|| pm_order.first().cloned().unwrap_or_default())
    };

    let is_worker_overridden = !task_worker_agent.is_empty();
    let effective_worker_agent = if is_worker_overridden {
        task_worker_agent.clone()
    } else {
        worker_order
            .iter()
            .find(|p| ready_providers.contains(p))
            .cloned()
            .unwrap_or_else(|| worker_order.first().cloned().unwrap_or_default())
    };

    Json(TaskAssignmentResponse {
        task_id,
        effective_lead_agent,
        effective_worker_agent,
        task_lead_agent,
        task_worker_agent,
        is_lead_overridden,
        is_worker_overridden,
        ready_providers,
    })
    .into_response()
}

async fn dispatch_task(
    State(state): State<Arc<ModuleState>>,
    headers: HeaderMap,
    AxumPath(task_id): AxumPath<String>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "DB not ready"})),
            )
                .into_response();
        }
    };

    if let Err((status, msg)) = check_task_access(pool, &headers, &task_id).await {
        return (status, Json(serde_json::json!({"error": msg}))).into_response();
    }

    // Luôn trả về workdir_provider_missing nếu worktree chưa tích hợp
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({
            "error": {
                "code": "workdir_provider_missing",
                "message": "Thiếu môi trường làm việc (WorkdirProvider). Vui lòng bật module 'worktree' trong Cài đặt > Tính năng để bắt đầu giao việc."
            }
        })),
    )
        .into_response()
}

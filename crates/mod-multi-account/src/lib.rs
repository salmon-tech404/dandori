//! Module `multi-account` cho Dandori.
//!
//! Quản lý nhiều tài khoản agent cùng một nhà cung cấp:
//! - Nhóm tài khoản theo nhà cung cấp
//! - Quản lý mức ưu tiên (số nhỏ = ưu tiên cao) và bật/tắt từng tài khoản
//! - Kiểm tra năng lực cô lập HOME của nhà cung cấp (chặn thêm tài khoản thứ hai nếu không hỗ trợ)
//! - Hiển thị thông tin tác vụ đang chạy trên từng tài khoản

#![warn(missing_docs)]

use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use dandori_module_api::{
    Manifest, Migration, Module, ModuleContext, ModuleSubscription, WorkerSpec, session_user,
    supports_home_isolation,
};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

#[expect(
    clippy::expect_used,
    reason = "embedded static module.json validated at build/test time"
)]
static MANIFEST: LazyLock<Manifest> = LazyLock::new(|| {
    let json = include_str!("../module.json");
    serde_json::from_str(json).expect("module.json của mod-multi-account phải hợp lệ")
});

static MIGRATIONS: [Migration; 0] = [];

/// Trạng thái chia sẻ nội bộ cho các route của module multi-account.
#[derive(Clone, Debug)]
struct ModuleState {
    pool: Option<SqlitePool>,
}

/// Struct đại diện cho module multi-account.
#[derive(Debug, Default, Clone, Copy)]
pub struct MultiAccountModule;

impl MultiAccountModule {
    /// Khởi tạo một thực thể MultiAccountModule mới.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for MultiAccountModule {
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
            .route("/accounts", get(list_accounts_grouped))
            .route("/accounts/{id}", patch(update_account))
            .route("/accounts/validate-addition", post(validate_addition))
            .with_state(state)
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        Vec::new()
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }
}

/// Thông tin nhiệm vụ đang chạy trên một tài khoản.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningTaskInfo {
    /// ID nhiệm vụ.
    pub task_id: String,
    /// Tiêu đề nhiệm vụ.
    pub task_title: String,
    /// ID phiên chạy.
    pub run_id: String,
}

/// Chi tiết tài khoản agent kèm thông tin tác vụ đang thực thi.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDetail {
    /// Định danh tài khoản.
    pub id: String,
    /// Nhà cung cấp (claude, agy, fake, codex, v.v.).
    pub provider: String,
    /// Nhãn hiển thị.
    pub label: String,
    /// Đường dẫn thư mục HOME cô lập.
    pub home_dir: String,
    /// Trạng thái kết nối (ready, not_installed, not_logged_in, v.v.).
    pub status: String,
    /// Trạng thái bật / tắt.
    pub enabled: bool,
    /// Mức ưu tiên (số nhỏ = ưu tiên cao).
    pub priority: i64,
    /// Thời điểm kiểm tra gần nhất.
    pub last_checked_at: Option<String>,
    /// Nhiệm vụ đang chạy (nếu có).
    pub running_task: Option<RunningTaskInfo>,
}

/// Nhóm các tài khoản theo từng nhà cung cấp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderGroup {
    /// Tên nhà cung cấp.
    pub provider: String,
    /// Nhà cung cấp có hỗ trợ cô lập thư mục HOME hay không.
    pub supports_home_isolation: bool,
    /// Danh sách các tài khoản thuộc nhà cung cấp này.
    pub accounts: Vec<AccountDetail>,
}

/// Yêu cầu cập nhật thông tin tài khoản agent.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAccountRequest {
    /// Mức ưu tiên mới (nếu có).
    pub priority: Option<i64>,
    /// Trạng thái bật/tắt mới (nếu có).
    pub enabled: Option<bool>,
    /// Nhãn hiển thị mới (nếu có).
    pub label: Option<String>,
}

/// Yêu cầu kiểm tra khả năng thêm tài khoản cho một nhà cung cấp.
#[derive(Debug, Clone, Deserialize)]
pub struct ValidateAdditionRequest {
    /// Tên nhà cung cấp cần kiểm tra.
    pub provider: String,
}

/// Phản hồi kiểm tra khả năng thêm tài khoản.
#[derive(Debug, Clone, Serialize)]
pub struct ValidateAdditionResponse {
    /// Có được phép thêm tài khoản mới hay không.
    pub allowed: bool,
    /// Lý do nếu không được phép thêm.
    pub reason: Option<String>,
}

async fn require_system_admin(
    pool: &SqlitePool,
    headers: &HeaderMap,
) -> Result<(), Box<axum::response::Response>> {
    match session_user(pool, headers).await {
        Some(user) if user.is_system_admin() => Ok(()),
        Some(_) => Err(Box::new(
            (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": "Chỉ quản trị hệ thống mới có quyền quản lý tài khoản agent"
                })),
            )
                .into_response(),
        )),
        None => Err(Box::new(
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Phiên đăng nhập không hợp lệ hoặc đã hết hạn"
                })),
            )
                .into_response(),
        )),
    }
}

async fn fetch_active_runs_map(
    pool: &SqlitePool,
) -> Result<HashMap<(String, String), RunningTaskInfo>, String> {
    let rows = sqlx::query(
        r#"
        SELECT
            r.id AS run_id,
            r.task_id,
            t.title AS task_title,
            r.account AS account_label,
            r.agent AS agent_provider
        FROM run r
        JOIN task t ON r.task_id = t.id
        WHERE r.status = 'running'
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Lỗi truy vấn run đang chạy: {e}"))?;

    let mut map = HashMap::new();
    for row in rows {
        let run_id: String = row.try_get("run_id").unwrap_or_default();
        let task_id: String = row.try_get("task_id").unwrap_or_default();
        let task_title: String = row.try_get("task_title").unwrap_or_default();
        let account_label: String = row.try_get("account_label").unwrap_or_default();
        let agent_provider: String = row.try_get("agent_provider").unwrap_or_default();

        map.insert(
            (
                agent_provider.to_ascii_lowercase(),
                account_label.to_ascii_lowercase(),
            ),
            RunningTaskInfo {
                task_id,
                task_title,
                run_id,
            },
        );
    }
    Ok(map)
}

fn build_provider_groups(
    rows: Vec<sqlx::sqlite::SqliteRow>,
    running_map: &HashMap<(String, String), RunningTaskInfo>,
) -> Vec<ProviderGroup> {
    let mut groups_map: HashMap<String, Vec<AccountDetail>> = HashMap::new();
    let mut provider_order: Vec<String> = Vec::new();

    for r in rows {
        let id: String = r.try_get("id").unwrap_or_default();
        let provider: String = r.try_get("provider").unwrap_or_default();
        let label: String = r.try_get("label").unwrap_or_default();
        let home_dir: String = r.try_get("home_dir").unwrap_or_default();
        let status: String = r.try_get("status").unwrap_or_default();
        let enabled_int: i64 = r.try_get("enabled").unwrap_or(1);
        let priority: i64 = r.try_get("priority").unwrap_or(100);
        let last_checked_at: Option<String> = r.try_get("last_checked_at").ok();

        let running_task = running_map
            .get(&(provider.to_ascii_lowercase(), label.to_ascii_lowercase()))
            .cloned();

        let detail = AccountDetail {
            id,
            provider: provider.clone(),
            label,
            home_dir,
            status,
            enabled: enabled_int == 1,
            priority,
            last_checked_at,
            running_task,
        };

        if !groups_map.contains_key(&provider) {
            provider_order.push(provider.clone());
        }
        groups_map.entry(provider).or_default().push(detail);
    }

    provider_order
        .into_iter()
        .map(|prov| {
            let isolation = supports_home_isolation(&prov);
            let accounts = groups_map.remove(&prov).unwrap_or_default();
            ProviderGroup {
                provider: prov,
                supports_home_isolation: isolation,
                accounts,
            }
        })
        .collect()
}

async fn list_accounts_grouped(State(state): State<Arc<ModuleState>>) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Cơ sở dữ liệu chưa sẵn sàng"})),
            )
                .into_response();
        }
    };

    let running_map = match fetch_active_runs_map(pool).await {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e})),
            )
                .into_response();
        }
    };

    let account_rows = match sqlx::query(
        "SELECT id, provider, label, home_dir, status, enabled, priority, last_checked_at FROM agent_account ORDER BY priority ASC, id ASC",
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Lỗi truy vấn tài khoản: {e}")})),
            )
                .into_response();
        }
    };

    let groups = build_provider_groups(account_rows, &running_map);
    Json(groups).into_response()
}

async fn fetch_current_account(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<sqlx::sqlite::SqliteRow>, String> {
    sqlx::query(
        "SELECT id, provider, label, home_dir, status, enabled, priority, last_checked_at FROM agent_account WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Lỗi truy vấn tài khoản: {e}"))
}

#[expect(
    clippy::too_many_lines,
    reason = "Cập nhật tài khoản thực hiện xác thực admin, truy vấn và lưu DB"
)]
async fn update_account(
    AxumPath(id): AxumPath<String>,
    State(state): State<Arc<ModuleState>>,
    headers: HeaderMap,
    Json(payload): Json<UpdateAccountRequest>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Cơ sở dữ liệu chưa sẵn sàng"})),
            )
                .into_response();
        }
    };

    if let Err(resp) = require_system_admin(pool, &headers).await {
        return *resp;
    }

    let current = match fetch_current_account(pool, &id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Không tìm thấy tài khoản agent"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e})),
            )
                .into_response();
        }
    };

    let cur_enabled: i64 = current.try_get("enabled").unwrap_or(1);
    let cur_priority: i64 = current.try_get("priority").unwrap_or(100);
    let cur_label: String = current.try_get("label").unwrap_or_default();
    let provider: String = current.try_get("provider").unwrap_or_default();
    let home_dir: String = current.try_get("home_dir").unwrap_or_default();
    let status: String = current.try_get("status").unwrap_or_default();
    let last_checked_at: Option<String> = current.try_get("last_checked_at").ok();

    let new_priority = payload.priority.unwrap_or(cur_priority);
    let new_enabled = payload
        .enabled
        .map_or(cur_enabled, |b| if b { 1 } else { 0 });
    let new_label = payload.label.unwrap_or(cur_label);

    if let Err(e) =
        sqlx::query("UPDATE agent_account SET priority = ?, enabled = ?, label = ? WHERE id = ?")
            .bind(new_priority)
            .bind(new_enabled)
            .bind(&new_label)
            .bind(&id)
            .execute(pool)
            .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Lỗi cập nhật tài khoản: {e}")})),
        )
            .into_response();
    }

    Json(AccountDetail {
        id,
        provider,
        label: new_label,
        home_dir,
        status,
        enabled: new_enabled == 1,
        priority: new_priority,
        last_checked_at,
        running_task: None,
    })
    .into_response()
}

async fn validate_addition(
    State(state): State<Arc<ModuleState>>,
    Json(payload): Json<ValidateAdditionRequest>,
) -> impl IntoResponse {
    let pool = match &state.pool {
        Some(p) => p,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Cơ sở dữ liệu chưa sẵn sàng"})),
            )
                .into_response();
        }
    };

    let provider = payload.provider.to_ascii_lowercase();
    let existing_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM agent_account WHERE LOWER(provider) = LOWER(?)")
            .bind(&provider)
            .fetch_one(pool)
            .await
            .unwrap_or(0);

    let isolation = supports_home_isolation(&provider);
    if existing_count > 0 && !isolation {
        Json(ValidateAdditionResponse {
            allowed: false,
            reason: Some(format!(
                "Nhà cung cấp '{provider}' không hỗ trợ cô lập thư mục HOME, không thể tạo thêm tài khoản thứ hai."
            )),
        })
        .into_response()
    } else {
        Json(ValidateAdditionResponse {
            allowed: true,
            reason: None,
        })
        .into_response()
    }
}

//! Module kết nối subscription cho các CLI agent với HOME riêng biệt.

use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use chrono::Utc;
use dandori_module_api::{
    Manifest, Migration, Module, ModuleContext, ModuleSubscription, WorkerSpec, probe_login_status,
    session_user, supports_home_isolation,
};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use uuid::Uuid;

#[expect(
    clippy::expect_used,
    reason = "embedded static module.json validated at build/test time"
)]
static MANIFEST: LazyLock<Manifest> = LazyLock::new(|| {
    let json = include_str!("../module.json");
    serde_json::from_str(json).expect("module.json của mod-subscription-connect phải hợp lệ")
});

static MIGRATIONS: [Migration; 0] = [];

/// Trạng thái chia sẻ nội bộ cho các route của module subscription-connect.
#[derive(Clone, Debug)]
struct ModuleState {
    pool: Option<SqlitePool>,
    data_dir: PathBuf,
}

/// Struct đại diện cho module subscription-connect.
#[derive(Debug, Default, Clone, Copy)]
pub struct SubscriptionConnectModule;

impl SubscriptionConnectModule {
    /// Khởi tạo một thực thể SubscriptionConnectModule mới.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for SubscriptionConnectModule {
    fn manifest(&self) -> &Manifest {
        &MANIFEST
    }

    fn migrations(&self) -> &'static [Migration] {
        &MIGRATIONS
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        let state = Arc::new(ModuleState {
            pool: ctx.pool().cloned(),
            data_dir: ctx.data_dir(),
        });

        Router::new()
            .route("/accounts", get(list_accounts).post(create_account))
            .route("/accounts/{id}", delete(delete_account))
            .route(
                "/accounts/{id}/login-instructions",
                get(get_login_instructions),
            )
            .route("/accounts/{id}/check", post(check_account_status))
            .with_state(state)
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        Vec::new()
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateAccountRequest {
    provider: String,
    label: String,
    #[serde(default = "default_priority")]
    priority: i64,
}

const fn default_priority() -> i64 {
    100
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountResponse {
    id: String,
    provider: String,
    label: String,
    home_dir: String,
    auth_type: String,
    status: String,
    enabled: bool,
    priority: i64,
    last_checked_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginInstructionsResponse {
    account_id: String,
    provider: String,
    home_dir: String,
    command: String,
    instructions: String,
}

async fn list_accounts(State(state): State<Arc<ModuleState>>) -> impl IntoResponse {
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

    let rows = match sqlx::query(
        r#"
        SELECT id, provider, label, home_dir, auth_type, status, enabled, priority, last_checked_at
        FROM agent_account
        ORDER BY priority ASC, id ASC
        "#,
    )
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Lỗi truy vấn tài khoản: {e}")})),
            )
                .into_response();
        }
    };

    let accounts: Vec<AccountResponse> = rows
        .into_iter()
        .map(|r| {
            let enabled_int: i64 = r.try_get("enabled").unwrap_or(1);
            AccountResponse {
                id: r.try_get("id").unwrap_or_default(),
                provider: r.try_get("provider").unwrap_or_default(),
                label: r.try_get("label").unwrap_or_default(),
                home_dir: r.try_get("home_dir").unwrap_or_default(),
                auth_type: r.try_get("auth_type").unwrap_or_default(),
                status: r.try_get("status").unwrap_or_default(),
                enabled: enabled_int == 1,
                priority: r.try_get("priority").unwrap_or(100),
                last_checked_at: r.try_get("last_checked_at").ok(),
            }
        })
        .collect();

    Json(accounts).into_response()
}

/// Tài khoản agent dùng chung cho cả hệ thống: chỉ quản trị hệ thống được tạo, xóa hay kiểm tra.
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
                    "error": "Chỉ quản trị hệ thống được quản lý tài khoản agent"
                })),
            )
                .into_response(),
        )),
        None => Err(Box::new(
            (
                StatusCode::UNAUTHORIZED,
                Json(
                    serde_json::json!({ "error": "Phiên đăng nhập không hợp lệ hoặc đã hết hạn" }),
                ),
            )
                .into_response(),
        )),
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "Tạo tài khoản kiểm tra hợp lệ, tạo thư mục HOME cô lập và ghi cơ sở dữ liệu"
)]
async fn create_account(
    State(state): State<Arc<ModuleState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateAccountRequest>,
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

    let provider = payload.provider.to_lowercase();
    if !["claude", "agy", "gemini", "codex", "fake"].contains(&provider.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Nhà cung cấp không hợp lệ. Hỗ trợ: claude, agy, gemini, codex, fake"
            })),
        )
            .into_response();
    }

    let existing_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM agent_account WHERE LOWER(provider) = LOWER(?)")
            .bind(&provider)
            .fetch_one(pool)
            .await
            .unwrap_or(0);

    if existing_count > 0 && !supports_home_isolation(&provider) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Nhà cung cấp '{provider}' không hỗ trợ cô lập thư mục HOME, không thể tạo thêm tài khoản thứ hai.")
            })),
        )
            .into_response();
    }

    let account_id = Uuid::new_v4().to_string();
    let home_path = state
        .data_dir
        .join("accounts")
        .join(&account_id)
        .join("home");

    // Tạo thư mục HOME biệt lập cho tài khoản
    if let Err(e) = tokio::fs::create_dir_all(&home_path).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Không thể tạo thư mục HOME cô lập: {e}")
            })),
        )
            .into_response();
    }

    let home_dir_str = home_path.to_string_lossy().to_string();
    let initial_status = probe_initial_status(&provider);
    let now = Utc::now().to_rfc3339();

    let res = sqlx::query(
        r#"
        INSERT INTO agent_account (
            id, provider, label, home_dir, auth_type, status, enabled, priority, last_checked_at
        )
        VALUES (?, ?, ?, ?, 'cli', ?, 1, ?, ?)
        "#,
    )
    .bind(&account_id)
    .bind(&provider)
    .bind(&payload.label)
    .bind(&home_dir_str)
    .bind(initial_status)
    .bind(payload.priority)
    .bind(&now)
    .execute(pool)
    .await;

    if let Err(e) = res {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Lỗi lưu tài khoản agent: {e}")
            })),
        )
            .into_response();
    }

    (
        StatusCode::CREATED,
        Json(AccountResponse {
            id: account_id,
            provider,
            label: payload.label,
            home_dir: home_dir_str,
            auth_type: "cli".to_string(),
            status: initial_status.to_string(),
            enabled: true,
            priority: payload.priority,
            last_checked_at: Some(now),
        }),
    )
        .into_response()
}

#[expect(
    clippy::too_many_lines,
    reason = "Xóa tài khoản truy vấn cơ sở dữ liệu và dọn dẹp thư mục HOME cô lập"
)]
async fn delete_account(
    AxumPath(id): AxumPath<String>,
    State(state): State<Arc<ModuleState>>,
    headers: HeaderMap,
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

    // Lấy thông tin tài khoản để xóa thư mục HOME
    let row = sqlx::query(
        r#"
        SELECT home_dir FROM agent_account WHERE id = ?
        "#,
    )
    .bind(&id)
    .fetch_optional(pool)
    .await;

    let home_dir: Option<String> = match row {
        Ok(Some(r)) => r.try_get("home_dir").ok(),
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
                Json(serde_json::json!({"error": format!("Lỗi truy vấn: {e}")})),
            )
                .into_response();
        }
    };

    let del_res = sqlx::query("DELETE FROM agent_account WHERE id = ?")
        .bind(&id)
        .execute(pool)
        .await;

    if let Err(e) = del_res {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Lỗi xóa tài khoản: {e}")})),
        )
            .into_response();
    }

    if let Some(path_str) = home_dir {
        let path = PathBuf::from(path_str);
        if path.exists() {
            let _ = tokio::fs::remove_dir_all(&path).await;
        }
    }

    Json(serde_json::json!({"deleted": true, "id": id})).into_response()
}

#[expect(
    clippy::too_many_lines,
    reason = "Tra tài khoản, kiểm tra quyền và dựng lệnh đăng nhập theo từng nhà cung cấp"
)]
async fn get_login_instructions(
    AxumPath(id): AxumPath<String>,
    State(state): State<Arc<ModuleState>>,
    headers: HeaderMap,
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

    let row = match sqlx::query(
        r#"
        SELECT id, provider, home_dir FROM agent_account WHERE id = ?
        "#,
    )
    .bind(&id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Không tìm thấy tài khoản"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Lỗi truy vấn: {e}")})),
            )
                .into_response();
        }
    };

    let provider: String = row.try_get("provider").unwrap_or_default();
    let home_dir: String = row.try_get("home_dir").unwrap_or_default();

    let (command, instructions) = build_login_command(&provider, &home_dir);

    Json(LoginInstructionsResponse {
        account_id: id,
        provider,
        home_dir,
        command,
        instructions,
    })
    .into_response()
}

/// Yêu cầu tùy chọn của `POST /accounts/{id}/check`.
#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct CheckRequest {
    /// Chủ xác nhận đã đăng nhập thủ công (dùng cho CLI chưa có lệnh kiểm tra đã xác minh).
    #[serde(default)]
    confirm_logged_in: bool,
}

/// Trạng thái sau khi kiểm tra lại: `claude` thăm dò thật bằng `claude auth status` trong HOME cô lập;
/// CLI khác chưa có lệnh kiểm tra đã xác minh nên chỉ thành `ready` khi chủ xác nhận.
async fn determine_check_status(
    provider: &str,
    home_dir: &Path,
    confirm_logged_in: bool,
) -> &'static str {
    if provider == "claude" {
        return probe_login_status(provider, home_dir).await.as_status();
    }
    apply_owner_confirmation(probe_initial_status(provider), confirm_logged_in)
}

/// Chủ xác nhận đã đăng nhập chỉ có tác dụng khi CLI đã cài (trạng thái `unknown`); không cài thì vẫn
/// là `not_installed`.
fn apply_owner_confirmation(base: &'static str, confirm_logged_in: bool) -> &'static str {
    if confirm_logged_in && base == "unknown" {
        "ready"
    } else {
        base
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "Kiểm tra lại trạng thái CLI và cập nhật thời gian kiểm tra vào cơ sở dữ liệu"
)]
async fn check_account_status(
    AxumPath(id): AxumPath<String>,
    State(state): State<Arc<ModuleState>>,
    headers: HeaderMap,
    body: Option<Json<CheckRequest>>,
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

    let row = match sqlx::query(
        r#"
        SELECT id, provider, label, home_dir, auth_type, enabled, priority
        FROM agent_account WHERE id = ?
        "#,
    )
    .bind(&id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Không tìm thấy tài khoản"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Lỗi truy vấn: {e}")})),
            )
                .into_response();
        }
    };

    let provider: String = row.try_get("provider").unwrap_or_default();
    let label: String = row.try_get("label").unwrap_or_default();
    let home_dir: String = row.try_get("home_dir").unwrap_or_default();
    let auth_type: String = row.try_get("auth_type").unwrap_or_default();
    let enabled_int: i64 = row.try_get("enabled").unwrap_or(1);
    let priority: i64 = row.try_get("priority").unwrap_or(100);

    let confirm_logged_in = body.is_some_and(|Json(b)| b.confirm_logged_in);
    let new_status =
        determine_check_status(&provider, Path::new(&home_dir), confirm_logged_in).await;
    let now = Utc::now().to_rfc3339();

    let update_res = sqlx::query(
        r#"
        UPDATE agent_account
        SET status = ?, last_checked_at = ?
        WHERE id = ?
        "#,
    )
    .bind(new_status)
    .bind(&now)
    .bind(&id)
    .execute(pool)
    .await;

    if let Err(e) = update_res {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Lỗi cập nhật trạng thái: {e}")})),
        )
            .into_response();
    }

    Json(AccountResponse {
        id,
        provider,
        label,
        home_dir,
        auth_type,
        status: new_status.to_string(),
        enabled: enabled_int == 1,
        priority,
        last_checked_at: Some(now),
    })
    .into_response()
}

fn probe_initial_status(provider: &str) -> &'static str {
    let bin_name = match provider {
        "claude" => "claude",
        "agy" | "gemini" => "agy",
        "codex" => "codex",
        "fake" => return "ready",
        _ => return "unknown",
    };

    if !check_binary_exists(bin_name) {
        "not_installed"
    } else {
        "unknown"
    }
}

fn check_binary_exists(program_name: &str) -> bool {
    let direct = Path::new(program_name);
    if direct.is_file() {
        return true;
    }

    let path_val = match std::env::var("PATH") {
        Ok(v) => v,
        Err(_) => return false,
    };

    let separator = if cfg!(windows) { ';' } else { ':' };
    let extensions = if cfg!(windows) {
        vec!["", ".exe", ".cmd", ".bat", ".ps1"]
    } else {
        vec![""]
    };

    for dir in path_val.split(separator) {
        let trimmed = dir.trim();
        if trimmed.is_empty() {
            continue;
        }
        let dir_path = Path::new(trimmed);
        for ext in &extensions {
            let candidate = dir_path.join(format!("{program_name}{ext}"));
            if candidate.is_file() {
                return true;
            }
        }
    }

    false
}

fn build_login_command(provider: &str, home_dir: &str) -> (String, String) {
    if cfg!(windows) {
        let appdata = format!("{home_dir}\\AppData\\Roaming");
        let localappdata = format!("{home_dir}\\AppData\\Local");
        let env_prefix = format!(
            "$env:USERPROFILE=\"{home_dir}\"; $env:HOME=\"{home_dir}\"; $env:APPDATA=\"{appdata}\"; $env:LOCALAPPDATA=\"{localappdata}\""
        );

        match provider {
            "claude" => (
                format!("{env_prefix}; claude auth login"),
                "Mở terminal PowerShell, chạy lệnh trên để đăng nhập tài khoản Claude. CLI sẽ mở trình duyệt để xác thực và lưu phiên vào thư mục HOME cô lập của tài khoản.".to_string(),
            ),
            "agy" | "gemini" => (
                format!("{env_prefix}; agy"),
                "Mở terminal PowerShell, chạy lệnh trên để mở phiên CLI tương tác của Antigravity/Gemini. Phiên đăng nhập sẽ được lưu trong thư mục cô lập.".to_string(),
            ),
            "codex" => (
                format!("{env_prefix}; codex login"),
                "Mở terminal PowerShell, chạy lệnh trên sau khi đã cài đặt Codex CLI.".to_string(),
            ),
            _ => (
                format!("{env_prefix}; echo 'Vui lòng thực hiện đăng nhập thủ công'"),
                "Thực hiện đăng nhập thủ công trong môi trường HOME cô lập.".to_string(),
            ),
        }
    } else {
        match provider {
            "claude" => (
                format!("HOME=\"{home_dir}\" claude auth login"),
                "Mở terminal, chạy lệnh trên để đăng nhập tài khoản Claude vào thư mục HOME cô lập.".to_string(),
            ),
            "agy" | "gemini" => (
                format!("HOME=\"{home_dir}\" agy"),
                "Mở terminal, chạy lệnh trên để khởi tạo phiên làm việc trong thư mục HOME cô lập.".to_string(),
            ),
            "codex" => (
                format!("HOME=\"{home_dir}\" codex login"),
                "Mở terminal, chạy lệnh trên để đăng nhập Codex.".to_string(),
            ),
            _ => (
                format!("HOME=\"{home_dir}\""),
                "Đăng nhập thủ công trong thư mục HOME cô lập.".to_string(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::apply_owner_confirmation;

    #[test]
    fn test_owner_confirmation_only_upgrades_unknown_to_ready() {
        assert_eq!(apply_owner_confirmation("unknown", true), "ready");
        assert_eq!(apply_owner_confirmation("unknown", false), "unknown");
        assert_eq!(
            apply_owner_confirmation("not_installed", true),
            "not_installed"
        );
        assert_eq!(apply_owner_confirmation("ready", false), "ready");
    }
}

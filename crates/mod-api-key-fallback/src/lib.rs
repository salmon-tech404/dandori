//! Module `api-key-fallback`: Dự phòng bằng API key với trần chi tiêu bắt buộc (P2-11).

#![warn(missing_docs)]

use std::str::FromStr;
use std::sync::{Arc, LazyLock};

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post, put};
use axum::{Json, Router};
use chrono::Utc;
use dandori_module_api::{
    EventBus, Manifest, MasterKeyStore, Migration, Module, ModuleContext, ModuleSubscription,
    SecretBox, SqliteSecretStorage, Vault, WorkerSpec, default_key_store, session_user,
};
use dandori_types::{AgentAccountId, Envelope, EventPayload, EventScope, SecretId};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};

#[expect(
    clippy::expect_used,
    reason = "embedded static module.json validated at build/test time"
)]
static MANIFEST: LazyLock<Manifest> = LazyLock::new(|| {
    let json = include_str!("../module.json");
    serde_json::from_str(json).expect("module.json của mod-api-key-fallback phải hợp lệ")
});

static MIGRATIONS: [Migration; 1] = [Migration {
    id: "0001_api_key_fallback",
    sql: include_str!("../migrations/0001_api_key_fallback.sql"),
}];

/// Helper che giấu khóa API, chỉ hiển thị tối đa 4 ký tự cuối.
#[must_use]
pub fn mask_api_key(key: &str) -> String {
    let trimmed = key.trim();
    if trimmed.len() <= 4 {
        return "••••".to_string();
    }
    let last4 = &trimmed[trimmed.len() - 4..];
    format!("••••••••••••{last4}")
}

/// Trạng thái của module `api-key-fallback`.
#[derive(Clone, Debug)]
pub struct ApiKeyFallbackState {
    pool: Option<SqlitePool>,
    event_bus: Arc<EventBus>,
    vault: Option<Arc<Vault<SqliteSecretStorage, Arc<dyn MasterKeyStore>>>>,
}

/// Yêu cầu tạo mới tài khoản API key.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateApiKeyAccountRequest {
    /// Nhà cung cấp (ví dụ `claude`, `gemini`, `codex`, `fake`).
    pub provider: String,
    /// Nhãn mô tả tài khoản.
    pub label: String,
    /// Khóa API bí mật (chỉ nhập một lần).
    pub api_key: String,
    /// Cấu hình trần chi tiêu bắt buộc.
    pub spending_limit: Option<SpendingLimitInput>,
}

/// Dữ liệu đầu vào cấu hình trần chi tiêu.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct SpendingLimitInput {
    /// Chu kỳ (`daily` hoặc `monthly`).
    pub period: String,
    /// Loại đơn vị (`tokens` hoặc `currency`).
    pub limit_type: String,
    /// Giá trị trần (> 0).
    pub limit_value: f64,
    /// Ngưỡng cảnh báo phần trăm (mặc định 80%).
    pub alert_threshold: Option<f64>,
}

/// Thông tin xem cấu hình trần chi tiêu.
#[derive(Debug, Serialize, Deserialize)]
pub struct SpendingLimitView {
    /// Chu kỳ.
    pub period: String,
    /// Loại đơn vị.
    pub limit_type: String,
    /// Giá trị trần tối đa.
    pub limit_value: f64,
    /// Giá trị đã sử dụng.
    pub used_value: f64,
    /// Phần trăm đã sử dụng.
    pub used_percentage: f64,
    /// Ngưỡng cảnh báo.
    pub alert_threshold: f64,
    /// Thời điểm bắt đầu chu kỳ.
    pub period_start: String,
}

/// Thông tin tài khoản API key trả về cho giao diện (không bao giờ lộ raw key).
#[derive(Debug, Serialize)]
pub struct ApiKeyAccountView {
    /// Định danh tài khoản.
    pub id: String,
    /// Nhà cung cấp.
    pub provider: String,
    /// Nhãn hiển thị.
    pub label: String,
    /// Khóa đã được che giấu.
    pub masked_key: String,
    /// Trạng thái kích hoạt.
    pub enabled: bool,
    /// Trần chi tiêu.
    pub spending_limit: Option<SpendingLimitView>,
    /// Thời điểm hết hạn mức tạm dừng (nếu có).
    pub exhausted_until: Option<String>,
}

/// Yêu cầu cập nhật cấu hình dự phòng của dự án.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateProjectFallbackRequest {
    /// Bật/tắt dự phòng API key cho dự án.
    pub fallback_enabled: bool,
    /// Danh sách tài khoản API key được phép dùng.
    pub allowed_accounts: Option<Vec<String>>,
}

/// Yêu cầu ghi nhận sử dụng quota/tiền.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordUsageRequest {
    /// Số token đã dùng.
    pub tokens: Option<i64>,
    /// Chi phí tiền đã dùng.
    pub cost: Option<f64>,
}

fn err_resp(status: StatusCode, msg: &str) -> Response {
    (status, Json(json!({ "error": msg }))).into_response()
}

async fn check_system_admin(pool: &SqlitePool, headers: &HeaderMap) -> Result<String, StatusCode> {
    let Some(user) = session_user(pool, headers).await else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if user.is_system_admin() {
        Ok(user.user_id)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

async fn check_project_access(
    pool: &SqlitePool,
    headers: &HeaderMap,
    prj_id: &str,
) -> Result<String, StatusCode> {
    let Some(user) = session_user(pool, headers).await else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if user.is_system_admin() {
        return Ok(user.user_id);
    }

    let prj_row = sqlx::query("SELECT workspace_id FROM project WHERE id = ?")
        .bind(prj_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let ws_id: String = prj_row.get("workspace_id");

    let member_row =
        sqlx::query("SELECT role FROM workspace_member WHERE workspace_id = ? AND user_id = ?")
            .bind(&ws_id)
            .bind(&user.user_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if member_row.is_some() {
        Ok(user.user_id)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "Xử lý danh sách tài khoản kèm tính toán phần trăm hạn mức"
)]
async fn list_accounts(State(state): State<ApiKeyFallbackState>, headers: HeaderMap) -> Response {
    let Some(ref pool) = state.pool else {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing");
    };
    if let Err(st) = check_system_admin(pool, &headers).await {
        return err_resp(st, "Không có quyền quản lý tài khoản API key");
    }

    let rows = match sqlx::query(
        "SELECT a.id, a.provider, a.label, a.enabled, a.exhausted_until, s.label as secret_label, \
         l.period, l.limit_type, l.limit_value, l.used_value, l.alert_threshold, l.period_start \
         FROM agent_account a \
         JOIN secret s ON a.secret_id = s.id \
         LEFT JOIN m_api_key_fallback_spending_limit l ON a.id = l.account_id \
         WHERE a.auth_type = 'api_key' \
         ORDER BY a.priority ASC, a.id ASC",
    )
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let accounts: Vec<ApiKeyAccountView> = rows
        .into_iter()
        .map(|r| {
            let limit_val: Option<f64> = r.get("limit_value");
            let limit = limit_val.map(|lv| {
                let uv: f64 = r.get::<Option<f64>, _>("used_value").unwrap_or(0.0);
                let pct = if lv > 0.0 {
                    (uv / lv * 100.0).min(100.0)
                } else {
                    0.0
                };
                SpendingLimitView {
                    period: r.get::<Option<String>, _>("period").unwrap_or_default(),
                    limit_type: r.get::<Option<String>, _>("limit_type").unwrap_or_default(),
                    limit_value: lv,
                    used_value: uv,
                    used_percentage: pct,
                    alert_threshold: r.get::<Option<f64>, _>("alert_threshold").unwrap_or(80.0),
                    period_start: r
                        .get::<Option<String>, _>("period_start")
                        .unwrap_or_default(),
                }
            });

            let secret_label: String = r.get("secret_label");
            let enabled_int: i64 = r.get("enabled");

            ApiKeyAccountView {
                id: r.get("id"),
                provider: r.get("provider"),
                label: r.get("label"),
                masked_key: mask_api_key(&secret_label),
                enabled: enabled_int == 1,
                spending_limit: limit,
                exhausted_until: r.get("exhausted_until"),
            }
        })
        .collect();

    Json(json!({ "accounts": accounts })).into_response()
}

#[expect(
    clippy::too_many_lines,
    reason = "Xử lý tạo tài khoản API key, mã hóa vault và gán hạn mức"
)]
async fn create_account(
    State(state): State<ApiKeyFallbackState>,
    headers: HeaderMap,
    Json(req): Json<CreateApiKeyAccountRequest>,
) -> Response {
    let Some(ref pool) = state.pool else {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing");
    };
    let Some(ref vault) = state.vault else {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Vault missing");
    };
    if let Err(st) = check_system_admin(pool, &headers).await {
        return err_resp(st, "Không có quyền quản lý tài khoản API key");
    }

    let raw_key = req.api_key.trim();
    if raw_key.is_empty() {
        return err_resp(StatusCode::BAD_REQUEST, "Khóa API không được để trống");
    }

    let masked = mask_api_key(raw_key);
    let secret_box = SecretBox::from(raw_key.to_string());
    let secret_id = match vault.put(&masked, secret_box).await {
        Ok(id) => id,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let acc_id = AgentAccountId::new().to_string();
    let sec_id_str = secret_id.to_string();
    let has_valid_limit = req
        .spending_limit
        .as_ref()
        .is_some_and(|l| l.limit_value > 0.0);
    let enabled_int = if has_valid_limit { 1 } else { 0 };

    let res = sqlx::query(
        "INSERT INTO agent_account (id, provider, label, home_dir, auth_type, secret_id, status, enabled, priority) \
         VALUES (?, ?, ?, '', 'api_key', ?, 'ready', ?, 150)"
    )
    .bind(&acc_id)
    .bind(&req.provider)
    .bind(&req.label)
    .bind(&sec_id_str)
    .bind(enabled_int)
    .execute(pool)
    .await;

    if let Err(e) = res {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
    }

    if let Some(limit) = req.spending_limit {
        let now_str = Utc::now().to_rfc3339();
        let threshold = limit.alert_threshold.unwrap_or(80.0);
        let _ = sqlx::query(
            "INSERT INTO m_api_key_fallback_spending_limit \
             (account_id, period, limit_type, limit_value, used_value, period_start, alert_threshold, created_at, updated_at) \
             VALUES (?, ?, ?, ?, 0.0, ?, ?, ?, ?)"
        )
        .bind(&acc_id)
        .bind(&limit.period)
        .bind(&limit.limit_type)
        .bind(limit.limit_value)
        .bind(&now_str)
        .bind(threshold)
        .bind(&now_str)
        .bind(&now_str)
        .execute(pool)
        .await;
    }

    Json(json!({
        "id": acc_id,
        "provider": req.provider,
        "label": req.label,
        "masked_key": masked,
        "enabled": enabled_int == 1
    }))
    .into_response()
}

#[expect(
    clippy::too_many_lines,
    reason = "Xử lý kiểm tra điều kiện kích hoạt tài khoản và hạn mức"
)]
async fn toggle_account(
    State(state): State<ApiKeyFallbackState>,
    Path(acc_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let Some(ref pool) = state.pool else {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing");
    };
    if let Err(st) = check_system_admin(pool, &headers).await {
        return err_resp(st, "Không có quyền quản lý tài khoản API key");
    }

    let acc = match sqlx::query("SELECT enabled FROM agent_account WHERE id = ?")
        .bind(&acc_id)
        .fetch_optional(pool)
        .await
    {
        Ok(Some(a)) => a,
        Ok(None) => return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy tài khoản"),
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let cur_enabled: i64 = acc.get("enabled");
    let target_enabled = cur_enabled == 0;
    if target_enabled {
        let limit = match sqlx::query(
            "SELECT limit_value, used_value FROM m_api_key_fallback_spending_limit WHERE account_id = ?"
        )
        .bind(&acc_id)
        .fetch_optional(pool)
        .await
        {
            Ok(Some(l)) => l,
            Ok(None) => {
                return err_resp(
                    StatusCode::BAD_REQUEST,
                    "Không thể kích hoạt tài khoản API key khi chưa đặt trần chi tiêu bắt buộc",
                );
            }
            Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
        };

        let limit_val: f64 = limit.get("limit_value");
        let used_val: f64 = limit.get("used_value");

        if limit_val <= 0.0 {
            return err_resp(
                StatusCode::BAD_REQUEST,
                "Không thể kích hoạt tài khoản API key khi chưa đặt trần chi tiêu bắt buộc",
            );
        }

        if used_val >= limit_val {
            return err_resp(
                StatusCode::BAD_REQUEST,
                "Tài khoản đã chạm trần chi tiêu, không thể kích hoạt cho đến chu kỳ tiếp theo",
            );
        }
    }

    let new_int = if target_enabled { 1 } else { 0 };
    let _ = sqlx::query("UPDATE agent_account SET enabled = ? WHERE id = ?")
        .bind(new_int)
        .bind(&acc_id)
        .execute(pool)
        .await;

    Json(json!({ "id": acc_id, "enabled": target_enabled })).into_response()
}

async fn set_spending_limit(
    State(state): State<ApiKeyFallbackState>,
    Path(acc_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<SpendingLimitInput>,
) -> Response {
    let Some(ref pool) = state.pool else {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing");
    };
    if let Err(st) = check_system_admin(pool, &headers).await {
        return err_resp(st, "Không có quyền quản lý tài khoản API key");
    }

    if req.limit_value <= 0.0 {
        return err_resp(
            StatusCode::BAD_REQUEST,
            "Giá trị trần chi tiêu phải lớn hơn 0",
        );
    }

    let now_str = Utc::now().to_rfc3339();
    let threshold = req.alert_threshold.unwrap_or(80.0);

    let res = sqlx::query(
        "INSERT INTO m_api_key_fallback_spending_limit \
         (account_id, period, limit_type, limit_value, used_value, period_start, alert_threshold, created_at, updated_at) \
         VALUES (?, ?, ?, ?, 0.0, ?, ?, ?, ?) \
         ON CONFLICT(account_id) DO UPDATE SET \
            period = excluded.period, \
            limit_type = excluded.limit_type, \
            limit_value = excluded.limit_value, \
            alert_threshold = excluded.alert_threshold, \
            updated_at = excluded.updated_at"
    )
    .bind(&acc_id)
    .bind(&req.period)
    .bind(&req.limit_type)
    .bind(req.limit_value)
    .bind(&now_str)
    .bind(threshold)
    .bind(&now_str)
    .bind(&now_str)
    .execute(pool)
    .await;

    match res {
        Ok(_) => Json(json!({ "success": true, "limit": req })).into_response(),
        Err(e) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn delete_account(
    State(state): State<ApiKeyFallbackState>,
    Path(acc_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let Some(ref pool) = state.pool else {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing");
    };
    if let Err(st) = check_system_admin(pool, &headers).await {
        return err_resp(st, "Không có quyền quản lý tài khoản API key");
    }

    let row = match sqlx::query("SELECT secret_id FROM agent_account WHERE id = ?")
        .bind(&acc_id)
        .fetch_optional(pool)
        .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy tài khoản"),
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let _ = sqlx::query("DELETE FROM agent_account WHERE id = ?")
        .bind(&acc_id)
        .execute(pool)
        .await;

    let sec_str: Option<String> = row.get("secret_id");
    if let Some(s) = sec_str
        && let Ok(sec_id) = SecretId::from_str(&s)
        && let Some(ref vault) = state.vault
    {
        let _ = vault.delete(&sec_id).await;
    }

    Json(json!({ "deleted": true })).into_response()
}

async fn get_project_config(
    State(state): State<ApiKeyFallbackState>,
    Path(prj_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let Some(ref pool) = state.pool else {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing");
    };
    if let Err(st) = check_project_access(pool, &headers, &prj_id).await {
        return err_resp(st, "Không có quyền truy cập dự án");
    }

    let row = match sqlx::query(
        "SELECT fallback_enabled, allowed_accounts FROM m_api_key_fallback_project_config WHERE project_id = ?"
    )
    .bind(&prj_id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(r)) => {
            let fb: i64 = r.get("fallback_enabled");
            let allowed_str: String = r.get("allowed_accounts");
            let allowed: Vec<String> = serde_json::from_str(&allowed_str).unwrap_or_default();
            json!({
                "project_id": prj_id,
                "fallback_enabled": fb == 1,
                "allowed_accounts": allowed
            })
        }
        Ok(None) => json!({
            "project_id": prj_id,
            "fallback_enabled": false,
            "allowed_accounts": []
        }),
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    Json(row).into_response()
}

async fn update_project_config(
    State(state): State<ApiKeyFallbackState>,
    Path(prj_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<UpdateProjectFallbackRequest>,
) -> Response {
    let Some(ref pool) = state.pool else {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing");
    };
    if let Err(st) = check_project_access(pool, &headers, &prj_id).await {
        return err_resp(st, "Không có quyền sửa cấu hình dự án");
    }

    let now_str = Utc::now().to_rfc3339();
    let fb_int = if req.fallback_enabled { 1 } else { 0 };
    let allowed_str = serde_json::to_string(&req.allowed_accounts.unwrap_or_default())
        .unwrap_or_else(|_| "[]".to_string());

    let res = sqlx::query(
        "INSERT INTO m_api_key_fallback_project_config \
         (project_id, fallback_enabled, allowed_accounts, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?) \
         ON CONFLICT(project_id) DO UPDATE SET \
            fallback_enabled = excluded.fallback_enabled, \
            allowed_accounts = excluded.allowed_accounts, \
            updated_at = excluded.updated_at",
    )
    .bind(&prj_id)
    .bind(fb_int)
    .bind(&allowed_str)
    .bind(&now_str)
    .bind(&now_str)
    .execute(pool)
    .await;

    match res {
        Ok(_) => Json(json!({
            "project_id": prj_id,
            "fallback_enabled": req.fallback_enabled
        }))
        .into_response(),
        Err(e) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "Xử lý cộng dồn chi phí, kiểm tra ngưỡng cảnh báo và tự ngắt tài khoản"
)]
async fn record_usage(
    State(state): State<ApiKeyFallbackState>,
    Path(acc_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<RecordUsageRequest>,
) -> Response {
    let Some(ref pool) = state.pool else {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing");
    };
    if let Err(st) = check_system_admin(pool, &headers).await {
        return err_resp(st, "Không có quyền quản lý tài khoản API key");
    }

    let limit = match sqlx::query(
        "SELECT limit_type, limit_value, used_value, alert_threshold, alerted_above_threshold \
         FROM m_api_key_fallback_spending_limit \
         WHERE account_id = ?",
    )
    .bind(&acc_id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(l)) => l,
        Ok(None) => return err_resp(StatusCode::NOT_FOUND, "Không có cấu hình trần chi tiêu"),
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let limit_type: String = limit.get("limit_type");
    let limit_value: f64 = limit.get("limit_value");
    let used_value: f64 = limit.get("used_value");
    let alert_threshold: f64 = limit.get("alert_threshold");
    let mut alerted: i64 = limit.get("alerted_above_threshold");

    let delta = match limit_type.as_str() {
        "tokens" => req.tokens.unwrap_or(0) as f64,
        _ => req.cost.unwrap_or(0.0),
    };

    let new_used = used_value + delta;
    let now_str = Utc::now().to_rfc3339();

    if limit_value > 0.0 {
        let pct = new_used / limit_value * 100.0;
        if pct >= alert_threshold && alerted == 0 {
            alerted = 1;
            let envelope = Envelope::new(
                Utc::now(),
                EventScope::empty(),
                EventPayload::NotificationRequested {
                    title: "Cảnh báo trần chi tiêu API key".to_string(),
                    message: format!(
                        "Tài khoản API key {acc_id} đã sử dụng {pct:.1}% trần cho phép"
                    ),
                    level: "warning".to_string(),
                },
            );
            state.event_bus.publish(envelope);
        }

        if new_used >= limit_value {
            let _ = sqlx::query(
                "UPDATE agent_account SET enabled = 0, exhausted_until = ? WHERE id = ?",
            )
            .bind(&now_str)
            .bind(&acc_id)
            .execute(pool)
            .await;

            let envelope = Envelope::new(
                Utc::now(),
                EventScope::empty(),
                EventPayload::NotificationRequested {
                    title: "Chạm trần chi tiêu API key".to_string(),
                    message: format!(
                        "Tài khoản API key {acc_id} đã chạm trần chi tiêu ({new_used} / {limit_value}) và bị tạm ngưng"
                    ),
                    level: "error".to_string(),
                },
            );
            state.event_bus.publish(envelope);
        }
    }

    let _ = sqlx::query(
        "UPDATE m_api_key_fallback_spending_limit \
         SET used_value = ?, alerted_above_threshold = ?, updated_at = ? \
         WHERE account_id = ?",
    )
    .bind(new_used)
    .bind(alerted)
    .bind(&now_str)
    .bind(&acc_id)
    .execute(pool)
    .await;

    Json(json!({ "account_id": acc_id, "used_value": new_used })).into_response()
}

/// Module quản lý dự phòng khóa API và trần chi tiêu.
#[derive(Debug)]
pub struct ApiKeyFallbackModule;

impl Default for ApiKeyFallbackModule {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiKeyFallbackModule {
    /// Khởi tạo module `api-key-fallback` mới.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for ApiKeyFallbackModule {
    fn manifest(&self) -> &Manifest {
        &MANIFEST
    }

    fn migrations(&self) -> &'static [Migration] {
        &MIGRATIONS
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        let pool = ctx.pool().cloned();
        let event_bus = Arc::clone(ctx.event_bus());
        let vault = pool.as_ref().map(|p| {
            let storage = SqliteSecretStorage::new(p.clone());
            Arc::new(Vault::new(storage, default_key_store()))
        });
        let state = ApiKeyFallbackState {
            pool,
            event_bus,
            vault,
        };

        Router::new()
            .route("/accounts", get(list_accounts).post(create_account))
            .route("/accounts/{id}/toggle", patch(toggle_account))
            .route("/accounts/{id}/spending-limit", put(set_spending_limit))
            .route("/accounts/{id}", delete(delete_account))
            .route("/accounts/{id}/record-usage", post(record_usage))
            .route(
                "/projects/{id}/config",
                get(get_project_config).put(update_project_config),
            )
            .with_state(state)
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        vec![]
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        vec![]
    }
}

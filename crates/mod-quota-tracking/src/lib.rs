//! Module `quota-tracking`: Theo dõi hạn mức từng tài khoản agent trung thực (P2-09).
//!
//! Hỗ trợ hai nguồn số liệu:
//! - `reported`: Số liệu báo cáo trực tiếp từ nhà cung cấp/CLI (khi CLI có hỗ trợ).
//! - `estimated`: Số liệu ước tính từ lịch sử các lần chạy (`run`) trong cửa sổ trượt 5 giờ và 7 ngày,
//!   so với giới hạn do chủ dự án cấu hình cho từng tài khoản.
//! - `no_data`: Khi tài khoản chưa được thiết lập giới hạn và chưa có báo cáo từ CLI.

#![warn(missing_docs)]

use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Duration, Utc};
use dandori_module_api::{
    EventBus, Manifest, Migration, Module, ModuleContext, ModuleSubscription, WorkerSpec,
    session_user,
};
use dandori_types::{AgentAccountId, Envelope, EventPayload, EventScope};
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
    serde_json::from_str(json).expect("module.json của mod-quota-tracking phải hợp lệ")
});

static MIGRATIONS: [Migration; 1] = [Migration {
    id: "0001_quota_tracking",
    sql: include_str!("../migrations/0001_quota_tracking.sql"),
}];

/// Cấu hình hạn mức của một tài khoản.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountQuotaConfigPayload {
    /// Giới hạn số lượt chạy trong cửa sổ 5 giờ.
    pub limit_5h: Option<i64>,
    /// Giới hạn số lượt chạy trong cửa sổ 7 ngày (tuần).
    pub limit_weekly: Option<i64>,
    /// Ngưỡng phát cảnh báo (phần trăm, mặc định 80).
    pub alert_threshold_percent: Option<i64>,
}

/// Dữ liệu báo cáo số liệu hạn mức thực tế từ CLI / Adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordReportedPayload {
    /// Số lượng đã sử dụng trong 5 giờ.
    pub used_5h: i64,
    /// Giới hạn 5 giờ theo nhà cung cấp (nếu có).
    pub limit_5h: Option<i64>,
    /// Số lượng đã sử dụng trong tuần (nếu có).
    pub used_weekly: Option<i64>,
    /// Giới hạn tuần theo nhà cung cấp (nếu có).
    pub limit_weekly: Option<i64>,
}

/// Yêu cầu đánh dấu tài khoản hết hạn mức.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkExhaustedPayload {
    /// Lý do hết hạn mức (ví dụ lỗi 429 hoặc thông báo CLI).
    pub reason: Option<String>,
    /// Số giờ chờ phục hồi (mặc định 5 giờ).
    pub wait_hours: Option<i64>,
}

/// Trạng thái hạn mức chi tiết của một tài khoản trả về cho frontend/API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaStatusResponse {
    /// Định danh tài khoản.
    pub account_id: String,
    /// Nguồn dữ liệu: "reported", "estimated" hoặc "no_data".
    pub source: String,
    /// Số lượng đã sử dụng trong 5 giờ.
    pub used_5h: i64,
    /// Giới hạn trong 5 giờ (nếu có).
    pub limit_5h: Option<i64>,
    /// Phần trăm đã sử dụng trong 5 giờ (null nếu không có giới hạn).
    pub percentage_5h: Option<f64>,
    /// Số lượng đã sử dụng trong tuần.
    pub used_weekly: i64,
    /// Giới hạn trong tuần (nếu có).
    pub limit_weekly: Option<i64>,
    /// Phần trăm đã sử dụng trong tuần (null nếu không có giới hạn).
    pub percentage_weekly: Option<f64>,
    /// Ngưỡng cảnh báo cấu hình.
    pub alert_threshold_percent: i64,
    /// Thời điểm hết hạn mức (nếu tài khoản đang bị tạm ngưng).
    pub exhausted_until: Option<String>,
}

/// Trạng thái chia sẻ nội bộ cho các route của module quota-tracking.
#[derive(Clone)]
struct ModuleState {
    pool: Option<SqlitePool>,
    event_bus: Arc<EventBus>,
}

/// Struct đại diện cho module quota-tracking.
#[derive(Debug, Default, Clone, Copy)]
pub struct QuotaTrackingModule;

impl QuotaTrackingModule {
    /// Khởi tạo một thực thể QuotaTrackingModule mới.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for QuotaTrackingModule {
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
        });

        Router::new()
            .route("/accounts/{id}/status", get(get_quota_status))
            .route("/accounts/{id}/config", put(update_quota_config))
            .route(
                "/accounts/{id}/mark-exhausted",
                post(mark_account_exhausted),
            )
            .route(
                "/accounts/{id}/record-reported",
                post(record_reported_quota),
            )
            .with_state(state)
    }

    fn workers(&self, ctx: &ModuleContext) -> Vec<WorkerSpec> {
        let pool = ctx.pool().cloned();
        let event_bus = Arc::clone(ctx.event_bus());

        vec![WorkerSpec::new(
            "quota_snapshot_calculator",
            move |cancel_token: CancellationToken| {
                let pool = pool.clone();
                let event_bus = Arc::clone(&event_bus);
                Box::pin(async move {
                    run_periodic_quota_worker(pool, event_bus, cancel_token).await;
                })
            },
        )]
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }
}

async fn run_periodic_quota_worker(
    pool: Option<SqlitePool>,
    _event_bus: Arc<EventBus>,
    cancel_token: CancellationToken,
) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                tracing::debug!("quota_snapshot_calculator worker dừng êm theo token hủy");
                break;
            }
            _ = interval.tick() => {
                if let Some(ref p) = pool {
                    let _ = scan_and_reset_exhausted_accounts(p).await;
                }
            }
        }
    }
}

async fn scan_and_reset_exhausted_accounts(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE agent_account SET exhausted_until = NULL WHERE exhausted_until IS NOT NULL AND exhausted_until <= ?")
        .bind(now)
        .execute(pool)
        .await?;
    Ok(())
}

/// Hạn mức là dữ liệu dùng chung của hệ thống: chỉ quản trị hệ thống được sửa cấu hình, đánh dấu hết hạn
/// hay ghi số liệu báo cáo (nếu không, bất kỳ ai đăng nhập cũng khóa được tài khoản của mọi người).
async fn require_system_admin(
    pool: &SqlitePool,
    headers: &HeaderMap,
) -> Result<(), Box<axum::response::Response>> {
    match session_user(pool, headers).await {
        Some(user) if user.is_system_admin() => Ok(()),
        Some(_) => Err(Box::new(
            (
                StatusCode::FORBIDDEN,
                "Chỉ quản trị hệ thống được thay đổi hạn mức",
            )
                .into_response(),
        )),
        None => Err(Box::new(
            (StatusCode::UNAUTHORIZED, "Chưa đăng nhập").into_response(),
        )),
    }
}

async fn get_quota_status(
    State(state): State<Arc<ModuleState>>,
    headers: HeaderMap,
    AxumPath(account_id): AxumPath<String>,
) -> impl IntoResponse {
    let Some(ref pool) = state.pool else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing").into_response();
    };

    if session_user(pool, &headers).await.is_none() {
        return (StatusCode::UNAUTHORIZED, "Chưa đăng nhập").into_response();
    }

    let acc_row =
        match sqlx::query("SELECT id, label, exhausted_until FROM agent_account WHERE id = ?")
            .bind(&account_id)
            .fetch_optional(pool)
            .await
        {
            Ok(Some(row)) => row,
            Ok(None) => return (StatusCode::NOT_FOUND, "Không tìm thấy tài khoản").into_response(),
            Err(err) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response();
            }
        };

    let label: String = acc_row.get("label");
    let exhausted_until: Option<String> = acc_row.get("exhausted_until");

    if let Ok(Some(rep_status)) =
        check_reported_status(pool, &account_id, exhausted_until.clone()).await
    {
        return Json(rep_status).into_response();
    }

    match check_estimated_status(pool, &account_id, &label, exhausted_until.clone()).await {
        Ok(Some(est_status)) => Json(est_status).into_response(),
        Ok(None) => Json(no_data_status(account_id, exhausted_until)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

async fn check_reported_status(
    pool: &SqlitePool,
    account_id: &str,
    exhausted_until: Option<String>,
) -> Result<Option<QuotaStatusResponse>, sqlx::Error> {
    let rep = sqlx::query(
        "SELECT used_5h, limit_5h, used_weekly, limit_weekly FROM m_quota_tracking_reported WHERE account_id = ?",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = rep else {
        return Ok(None);
    };

    let used_5h: i64 = row.get("used_5h");
    let limit_5h: Option<i64> = row.get("limit_5h");
    let used_weekly: i64 = row.get("used_weekly");
    let limit_weekly: Option<i64> = row.get("limit_weekly");

    Ok(Some(QuotaStatusResponse {
        account_id: account_id.to_string(),
        source: "reported".to_string(),
        used_5h,
        limit_5h,
        percentage_5h: calc_percentage(used_5h, limit_5h),
        used_weekly,
        limit_weekly,
        percentage_weekly: calc_percentage(used_weekly, limit_weekly),
        alert_threshold_percent: 80,
        exhausted_until,
    }))
}

async fn check_estimated_status(
    pool: &SqlitePool,
    account_id: &str,
    label: &str,
    exhausted_until: Option<String>,
) -> Result<Option<QuotaStatusResponse>, sqlx::Error> {
    let cfg = sqlx::query(
        "SELECT limit_5h, limit_weekly, alert_threshold_percent FROM m_quota_tracking_config WHERE account_id = ?",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await?;

    let Some(cfg_row) = cfg else {
        return Ok(None);
    };

    let limit_5h: Option<i64> = cfg_row.get("limit_5h");
    let limit_weekly: Option<i64> = cfg_row.get("limit_weekly");
    let alert_threshold_percent: i64 = cfg_row.get("alert_threshold_percent");

    if limit_5h.is_none() && limit_weekly.is_none() {
        return Ok(None);
    }

    let (used_5h, used_weekly) = count_runs(pool, account_id, label).await?;

    Ok(Some(QuotaStatusResponse {
        account_id: account_id.to_string(),
        source: "estimated".to_string(),
        used_5h,
        limit_5h,
        percentage_5h: calc_percentage(used_5h, limit_5h),
        used_weekly,
        limit_weekly,
        percentage_weekly: calc_percentage(used_weekly, limit_weekly),
        alert_threshold_percent,
        exhausted_until,
    }))
}

fn no_data_status(account_id: String, exhausted_until: Option<String>) -> QuotaStatusResponse {
    QuotaStatusResponse {
        account_id,
        source: "no_data".to_string(),
        used_5h: 0,
        limit_5h: None,
        percentage_5h: None,
        used_weekly: 0,
        limit_weekly: None,
        percentage_weekly: None,
        alert_threshold_percent: 80,
        exhausted_until,
    }
}

fn calc_percentage(used: i64, limit: Option<i64>) -> Option<f64> {
    limit.and_then(|lim| {
        if lim > 0 {
            Some(((used as f64 / lim as f64) * 100.0).round())
        } else {
            None
        }
    })
}

async fn count_runs(
    pool: &SqlitePool,
    account_id: &str,
    label: &str,
) -> Result<(i64, i64), sqlx::Error> {
    let now = Utc::now();
    let week_ago = (now - Duration::days(7)).to_rfc3339();

    let rows = sqlx::query(
        "SELECT started_at FROM run WHERE (account = ? OR account = ?) AND started_at >= ?",
    )
    .bind(account_id)
    .bind(label)
    .bind(&week_ago)
    .fetch_all(pool)
    .await?;

    let threshold_5h = now - Duration::hours(5);
    let mut count_5h = 0;
    let mut count_weekly = 0;

    for row in rows {
        let started_at_str: String = row.get("started_at");
        if let Ok(ts) = DateTime::parse_from_rfc3339(&started_at_str) {
            let utc_ts = ts.with_timezone(&Utc);
            if utc_ts >= threshold_5h {
                count_5h += 1;
            }
            count_weekly += 1;
        }
    }

    Ok((count_5h, count_weekly))
}

async fn update_quota_config(
    State(state): State<Arc<ModuleState>>,
    headers: HeaderMap,
    AxumPath(account_id): AxumPath<String>,
    Json(payload): Json<AccountQuotaConfigPayload>,
) -> impl IntoResponse {
    let Some(ref pool) = state.pool else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing").into_response();
    };

    if let Err(resp) = require_system_admin(pool, &headers).await {
        return *resp;
    }

    let now = Utc::now().to_rfc3339();
    let threshold = payload.alert_threshold_percent.unwrap_or(80);

    let res = sqlx::query(
        r#"
        INSERT INTO m_quota_tracking_config (account_id, limit_5h, limit_weekly, alert_threshold_percent, alerted_above_threshold, created_at, updated_at)
        VALUES (?, ?, ?, ?, 0, ?, ?)
        ON CONFLICT(account_id) DO UPDATE SET
            limit_5h = excluded.limit_5h,
            limit_weekly = excluded.limit_weekly,
            alert_threshold_percent = excluded.alert_threshold_percent,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(&account_id)
    .bind(payload.limit_5h)
    .bind(payload.limit_weekly)
    .bind(threshold)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await;

    match res {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

async fn mark_account_exhausted(
    State(state): State<Arc<ModuleState>>,
    headers: HeaderMap,
    AxumPath(account_id): AxumPath<String>,
    Json(payload): Json<MarkExhaustedPayload>,
) -> impl IntoResponse {
    let Some(ref pool) = state.pool else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing").into_response();
    };

    if let Err(resp) = require_system_admin(pool, &headers).await {
        return *resp;
    }

    let wait_hours = payload.wait_hours.unwrap_or(5);
    let exhausted_until = Utc::now() + Duration::hours(wait_hours);

    let res = sqlx::query("UPDATE agent_account SET exhausted_until = ? WHERE id = ?")
        .bind(exhausted_until.to_rfc3339())
        .bind(&account_id)
        .execute(pool)
        .await;

    if let Err(err) = res {
        return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response();
    }

    // Phát sự kiện cập nhật hạn mức lên EventBus
    let parsed_id = account_id.parse::<AgentAccountId>().unwrap_or_default();
    let event = Envelope::new(
        Utc::now(),
        EventScope::empty(),
        EventPayload::AccountQuotaUpdated {
            account_id: parsed_id,
            exhausted_until: Some(exhausted_until),
        },
    );
    state.event_bus.publish(event);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "exhausted_until": exhausted_until.to_rfc3339()
        })),
    )
        .into_response()
}

async fn record_reported_quota(
    State(state): State<Arc<ModuleState>>,
    headers: HeaderMap,
    AxumPath(account_id): AxumPath<String>,
    Json(payload): Json<RecordReportedPayload>,
) -> impl IntoResponse {
    let Some(ref pool) = state.pool else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing").into_response();
    };

    if let Err(resp) = require_system_admin(pool, &headers).await {
        return *resp;
    }

    let now = Utc::now().to_rfc3339();
    let res = sqlx::query(
        r#"
        INSERT INTO m_quota_tracking_reported (account_id, used_5h, limit_5h, used_weekly, limit_weekly, reported_at)
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(account_id) DO UPDATE SET
            used_5h = excluded.used_5h,
            limit_5h = excluded.limit_5h,
            used_weekly = excluded.used_weekly,
            limit_weekly = excluded.limit_weekly,
            reported_at = excluded.reported_at
        "#,
    )
    .bind(&account_id)
    .bind(payload.used_5h)
    .bind(payload.limit_5h)
    .bind(payload.used_weekly.unwrap_or(0))
    .bind(payload.limit_weekly)
    .bind(&now)
    .execute(pool)
    .await;

    if let Err(err) = res {
        return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response();
    }

    // Kiểm tra ngưỡng cảnh báo (chỉ phát một lần mỗi lần vượt ngưỡng)
    check_threshold_alert(pool, &state.event_bus, &account_id, &payload).await;

    (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response()
}

fn is_quota_above_threshold(
    cfg_limit_5h: Option<i64>,
    cfg_limit_w: Option<i64>,
    threshold: i64,
    reported: &RecordReportedPayload,
) -> bool {
    let eff_5h = reported.limit_5h.or(cfg_limit_5h);
    let eff_w = reported.limit_weekly.or(cfg_limit_w);

    let is_5h = eff_5h
        .map(|lim| lim > 0 && (reported.used_5h * 100 >= lim * threshold))
        .unwrap_or(false);

    let is_w = eff_w
        .map(|lim| {
            let used = reported.used_weekly.unwrap_or(0);
            lim > 0 && (used * 100 >= lim * threshold)
        })
        .unwrap_or(false);

    is_5h || is_w
}

async fn check_threshold_alert(
    pool: &SqlitePool,
    event_bus: &EventBus,
    account_id: &str,
    reported: &RecordReportedPayload,
) {
    let Ok(Some(cfg)) = sqlx::query(
        "SELECT limit_5h, limit_weekly, alert_threshold_percent, alerted_above_threshold FROM m_quota_tracking_config WHERE account_id = ?",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await else {
        return;
    };

    let threshold: i64 = cfg.get("alert_threshold_percent");
    let already_alerted: i64 = cfg.get("alerted_above_threshold");
    let is_above = is_quota_above_threshold(
        cfg.get("limit_5h"),
        cfg.get("limit_weekly"),
        threshold,
        reported,
    );

    if is_above && already_alerted == 0 {
        let envelope = Envelope::new(
            Utc::now(),
            EventScope::empty(),
            EventPayload::NotificationRequested {
                title: "Cảnh báo hạn mức tài khoản".to_string(),
                message: format!(
                    "Tài khoản {account_id} đã chạm hoặc vượt ngưỡng cảnh báo {threshold}% hạn mức"
                ),
                level: "warning".to_string(),
            },
        );
        event_bus.publish(envelope);

        let _ = sqlx::query(
            "UPDATE m_quota_tracking_config SET alerted_above_threshold = 1 WHERE account_id = ?",
        )
        .bind(account_id)
        .execute(pool)
        .await;
    } else if !is_above && already_alerted == 1 {
        let _ = sqlx::query(
            "UPDATE m_quota_tracking_config SET alerted_above_threshold = 0 WHERE account_id = ?",
        )
        .bind(account_id)
        .execute(pool)
        .await;
    }
}

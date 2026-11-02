//! Các endpoint REST API cho module `reports`.

use axum::Json;
use axum::Router;
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use chrono::Utc;
use dandori_module_api::{
    ModuleContext, SessionUser, can_access_workspace, can_administer_workspace, session_user,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::generator::{
    calculate_period_bounds, export_report_to_markdown, generate_report_data, parse_timezone,
};
use crate::model::{ReportConfig, ReportSnapshot, UpdateReportConfigRequest};
use crate::worker::{Clock, RealClock};

/// Trạng thái Router của module reports.
#[derive(Clone, Debug)]
pub struct ReportsState {
    /// Ngữ cảnh module.
    pub ctx: ModuleContext,
    /// Đồng hồ thời gian.
    pub clock: Arc<dyn Clock>,
}

impl ReportsState {
    /// Khởi tạo trạng thái mới với đồng hồ thực tế.
    #[must_use]
    pub fn new(ctx: ModuleContext) -> Self {
        Self {
            ctx,
            clock: Arc::new(RealClock),
        }
    }

    /// Khởi tạo trạng thái với đồng hồ tùy chỉnh.
    #[must_use]
    pub fn with_clock(ctx: ModuleContext, clock: Arc<dyn Clock>) -> Self {
        Self { ctx, clock }
    }
}

fn err_resp(status: StatusCode, msg: &'static str) -> Response {
    (status, Json(json!({ "error": { "message": msg } }))).into_response()
}

fn get_pool(state: &ReportsState) -> Result<SqlitePool, (StatusCode, &'static str)> {
    state.ctx.pool().cloned().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Không có kết nối cơ sở dữ liệu",
    ))
}

async fn get_user(
    headers: &HeaderMap,
    pool: &SqlitePool,
) -> Result<SessionUser, (StatusCode, &'static str)> {
    session_user(pool, headers).await.ok_or((
        StatusCode::UNAUTHORIZED,
        "Yêu cầu đăng nhập để truy cập tài nguyên",
    ))
}

/// Xây dựng Router cho module reports.
pub fn reports_router(state: ReportsState) -> Router {
    Router::new()
        .route(
            "/workspaces/{ws_id}/reports/current",
            get(get_current_report),
        )
        .route(
            "/workspaces/{ws_id}/reports/snapshots",
            get(list_report_snapshots),
        )
        .route(
            "/workspaces/{ws_id}/reports/snapshots/{snapshot_id}",
            get(get_report_snapshot),
        )
        .route(
            "/workspaces/{ws_id}/reports/config",
            get(get_report_config).put(update_report_config),
        )
        .with_state(state)
}

async fn ensure_config_exists(pool: &SqlitePool, ws_id: &str) -> Result<ReportConfig, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        INSERT INTO m_report_config (
            workspace_id, auto_generate, timezone, daily_time, weekly_day, weekly_time, updated_at
        ) VALUES (?, 0, 'UTC', '23:59', 7, '23:59', ?)
        ON CONFLICT(workspace_id) DO NOTHING
        "#,
    )
    .bind(ws_id)
    .bind(&now)
    .execute(pool)
    .await?;

    sqlx::query_as("SELECT * FROM m_report_config WHERE workspace_id = ?")
        .bind(ws_id)
        .fetch_one(pool)
        .await
}

/// Tham số truy vấn báo cáo hiện tại.
#[derive(Debug, Deserialize)]
pub struct CurrentReportQuery {
    /// Loại kỳ (`daily` hoặc `weekly`, mặc định `daily`).
    #[serde(default = "default_period")]
    pub period_type: String,
    /// Múi giờ (`UTC`, `+07:00`...).
    pub timezone: Option<String>,
}

fn default_period() -> String {
    "daily".to_string()
}

#[expect(
    clippy::too_many_lines,
    reason = "Xử lý tham số truy vấn và tổng hợp báo cáo hiện tại"
)]
async fn get_current_report(
    AxumPath(ws_id): AxumPath<String>,
    State(state): State<ReportsState>,
    headers: HeaderMap,
    Query(query): Query<CurrentReportQuery>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((status, msg)) => return err_resp(status, msg),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((status, msg)) => return err_resp(status, msg),
    };

    if !can_access_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Không có quyền truy cập không gian làm việc này",
        );
    }

    let config = match ensure_config_exists(&pool, &ws_id).await {
        Ok(cfg) => cfg,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    let tz_str = query.timezone.unwrap_or(config.timezone);
    let tz = parse_timezone(&tz_str);
    let now = state.clock.now();
    let (start_utc, end_utc) = calculate_period_bounds(now, &query.period_type, tz);

    match generate_report_data(
        &pool,
        &ws_id,
        &query.period_type,
        start_utc,
        end_utc,
        &tz_str,
    )
    .await
    {
        Ok(data) => {
            let markdown = export_report_to_markdown(&data);
            (
                StatusCode::OK,
                Json(json!({ "report": data, "markdown": markdown })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn list_report_snapshots(
    AxumPath(ws_id): AxumPath<String>,
    State(state): State<ReportsState>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((status, msg)) => return err_resp(status, msg),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((status, msg)) => return err_resp(status, msg),
    };

    if !can_access_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Không có quyền truy cập không gian làm việc này",
        );
    }

    let snapshots: Result<Vec<ReportSnapshot>, _> = sqlx::query_as(
        "SELECT * FROM m_report_snapshot WHERE workspace_id = ? ORDER BY period_start DESC LIMIT 50",
    )
    .bind(&ws_id)
    .fetch_all(&pool)
    .await;

    match snapshots {
        Ok(list) => (StatusCode::OK, Json(json!({ "snapshots": list }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn get_report_snapshot(
    AxumPath((ws_id, snapshot_id)): AxumPath<(String, String)>,
    State(state): State<ReportsState>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((status, msg)) => return err_resp(status, msg),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((status, msg)) => return err_resp(status, msg),
    };

    if !can_access_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Không có quyền truy cập không gian làm việc này",
        );
    }

    let snapshot: Result<Option<ReportSnapshot>, _> =
        sqlx::query_as("SELECT * FROM m_report_snapshot WHERE workspace_id = ? AND id = ?")
            .bind(&ws_id)
            .bind(&snapshot_id)
            .fetch_optional(&pool)
            .await;

    match snapshot {
        Ok(Some(s)) => (StatusCode::OK, Json(json!({ "snapshot": s }))).into_response(),
        Ok(None) => err_resp(StatusCode::NOT_FOUND, "Không tìm thấy bản chụp báo cáo"),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn get_report_config(
    AxumPath(ws_id): AxumPath<String>,
    State(state): State<ReportsState>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((status, msg)) => return err_resp(status, msg),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((status, msg)) => return err_resp(status, msg),
    };

    if !can_access_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Không có quyền truy cập không gian làm việc này",
        );
    }

    match ensure_config_exists(&pool, &ws_id).await {
        Ok(cfg) => (StatusCode::OK, Json(json!({ "config": cfg }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn update_report_config(
    AxumPath(ws_id): AxumPath<String>,
    State(state): State<ReportsState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateReportConfigRequest>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((status, msg)) => return err_resp(status, msg),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((status, msg)) => return err_resp(status, msg),
    };

    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Cần quyền quản trị viên không gian làm việc để thay đổi cấu hình báo cáo",
        );
    }

    let _ = ensure_config_exists(&pool, &ws_id).await;

    let now = Utc::now().to_rfc3339();
    let res = sqlx::query(
        r#"
        UPDATE m_report_config
        SET auto_generate = COALESCE(?, auto_generate),
            timezone = COALESCE(?, timezone),
            daily_time = COALESCE(?, daily_time),
            weekly_day = COALESCE(?, weekly_day),
            weekly_time = COALESCE(?, weekly_time),
            updated_at = ?
        WHERE workspace_id = ?
        "#,
    )
    .bind(payload.auto_generate.map(|b| if b { 1i64 } else { 0i64 }))
    .bind(payload.timezone)
    .bind(payload.daily_time)
    .bind(payload.weekly_day)
    .bind(payload.weekly_time)
    .bind(&now)
    .bind(&ws_id)
    .execute(&pool)
    .await;

    match res {
        Ok(_) => (StatusCode::OK, Json(json!({ "status": "updated" }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

//! Định nghĩa các route HTTP cho module `quiet-hours`.

#![expect(
    clippy::too_many_lines,
    clippy::result_large_err,
    reason = "Xử lý các endpoint REST của quiet hours kèm kiểm tra BOLA và xác thực dữ liệu"
)]

use axum::Json;
use axum::Router;
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use chrono::Utc;
use dandori_module_api::{
    ModuleContext, SessionUser, can_access_workspace, can_administer_workspace, session_user,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

use crate::guard::{Clock, RealClock};
use crate::model::{CreateQuietWindowDto, QuietWindow, UpdateQuietWindowDto};
use crate::window::{evaluate_windows, parse_days_of_week, parse_hh_mm, parse_timezone_offset};

/// Trạng thái dùng chung cho Router của module quiet-hours.
#[derive(Clone, Debug)]
pub struct QuietHoursState {
    /// Ngữ cảnh runtime của module.
    pub ctx: ModuleContext,
    /// Đồng hồ thời gian.
    pub clock: Arc<dyn Clock>,
}

impl QuietHoursState {
    /// Khởi tạo trạng thái với đồng hồ thực tế.
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

fn err_resp_dyn(status: StatusCode, msg: String) -> Response {
    (status, Json(json!({ "error": { "message": msg } }))).into_response()
}

fn get_pool(state: &QuietHoursState) -> Result<SqlitePool, Response> {
    state.ctx.pool().cloned().ok_or_else(|| {
        err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Không có kết nối cơ sở dữ liệu",
        )
    })
}

async fn get_user(headers: &HeaderMap, pool: &SqlitePool) -> Result<SessionUser, Response> {
    session_user(pool, headers).await.ok_or_else(|| {
        err_resp(
            StatusCode::UNAUTHORIZED,
            "Phiên đăng nhập không hợp lệ hoặc đã hết hạn",
        )
    })
}

/// Khởi tạo router HTTP cho module `quiet-hours`.
pub fn quiet_hours_router(state: QuietHoursState) -> Router {
    Router::new()
        .route(
            "/workspaces/{ws_id}/windows",
            get(list_windows).post(create_window),
        )
        .route(
            "/workspaces/{ws_id}/windows/{window_id}",
            put(update_window).delete(delete_window),
        )
        .route(
            "/workspaces/{ws_id}/windows/{window_id}/toggle",
            post(toggle_window),
        )
        .route("/workspaces/{ws_id}/status", get(get_quiet_status))
        .with_state(state)
}

/// `GET /workspaces/{ws_id}/windows` - Lấy danh sách khung giờ cấm.
async fn list_windows(
    AxumPath(ws_id): AxumPath<String>,
    State(state): State<QuietHoursState>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return r,
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    if !can_access_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Không có quyền truy cập không gian làm việc này",
        );
    }

    let windows: Vec<QuietWindow> = match sqlx::query_as(
        r#"
        SELECT id, workspace_id, project_id, name, days_of_week, start_time,
               end_time, timezone, block_manual_runs, enabled, created_at, updated_at
        FROM m_quiet_hours_window
        WHERE workspace_id = ?
        ORDER BY created_at ASC
        "#,
    )
    .bind(&ws_id)
    .fetch_all(&pool)
    .await
    {
        Ok(w) => w,
        Err(e) => return err_resp_dyn(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    Json(json!({ "windows": windows })).into_response()
}

/// `POST /workspaces/{ws_id}/windows` - Tạo mới một khung giờ cấm.
async fn create_window(
    AxumPath(ws_id): AxumPath<String>,
    State(state): State<QuietHoursState>,
    headers: HeaderMap,
    Json(dto): Json<CreateQuietWindowDto>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return r,
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Cần quyền quản trị viên để tạo khung giờ cấm",
        );
    }

    let name = dto.name.trim();
    if name.is_empty() {
        return err_resp(
            StatusCode::BAD_REQUEST,
            "Tên khung giờ cấm không được để trống",
        );
    }

    if let Err(e) = parse_hh_mm(&dto.start_time) {
        return err_resp_dyn(StatusCode::BAD_REQUEST, e);
    }
    if let Err(e) = parse_hh_mm(&dto.end_time) {
        return err_resp_dyn(StatusCode::BAD_REQUEST, e);
    }

    let days = parse_days_of_week(&dto.days_of_week);
    if days.is_empty() {
        return err_resp(
            StatusCode::BAD_REQUEST,
            "Cần chọn ít nhất một ngày trong tuần (1..7)",
        );
    }

    let tz = dto.timezone.unwrap_or_else(|| "UTC".to_string());
    if let Err(e) = parse_timezone_offset(&tz) {
        return err_resp_dyn(StatusCode::BAD_REQUEST, e);
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let block_manual = if dto.block_manual_runs.unwrap_or(false) {
        1
    } else {
        0
    };

    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO m_quiet_hours_window (
            id, workspace_id, project_id, name, days_of_week, start_time,
            end_time, timezone, block_manual_runs, enabled, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&ws_id)
    .bind(&dto.project_id)
    .bind(name)
    .bind(&dto.days_of_week)
    .bind(&dto.start_time)
    .bind(&dto.end_time)
    .bind(&tz)
    .bind(block_manual)
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    {
        return err_resp_dyn(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    let window: Option<QuietWindow> = sqlx::query_as(
        r#"
        SELECT id, workspace_id, project_id, name, days_of_week, start_time,
               end_time, timezone, block_manual_runs, enabled, created_at, updated_at
        FROM m_quiet_hours_window
        WHERE id = ?
        "#,
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    (StatusCode::CREATED, Json(json!({ "window": window }))).into_response()
}

/// `PUT /workspaces/{ws_id}/windows/{window_id}` - Cập nhật khung giờ cấm.
async fn update_window(
    AxumPath((ws_id, window_id)): AxumPath<(String, String)>,
    State(state): State<QuietHoursState>,
    headers: HeaderMap,
    Json(dto): Json<UpdateQuietWindowDto>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return r,
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Cần quyền quản trị viên để cập nhật khung giờ cấm",
        );
    }

    let current: Option<QuietWindow> = match sqlx::query_as(
        r#"
        SELECT id, workspace_id, project_id, name, days_of_week, start_time,
               end_time, timezone, block_manual_runs, enabled, created_at, updated_at
        FROM m_quiet_hours_window
        WHERE id = ? AND workspace_id = ?
        "#,
    )
    .bind(&window_id)
    .bind(&ws_id)
    .fetch_optional(&pool)
    .await
    {
        Ok(w) => w,
        Err(e) => return err_resp_dyn(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    let current = match current {
        Some(w) => w,
        None => return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy khung giờ cấm"),
    };

    let new_name = dto.name.unwrap_or(current.name);
    let new_start = dto.start_time.unwrap_or(current.start_time);
    let new_end = dto.end_time.unwrap_or(current.end_time);
    let new_days = dto.days_of_week.unwrap_or(current.days_of_week);
    let new_tz = dto.timezone.unwrap_or(current.timezone);
    let new_block_manual = dto.block_manual_runs.unwrap_or(current.block_manual_runs);
    let new_enabled = dto.enabled.unwrap_or(current.enabled);

    if let Err(e) = parse_hh_mm(&new_start) {
        return err_resp_dyn(StatusCode::BAD_REQUEST, e);
    }
    if let Err(e) = parse_hh_mm(&new_end) {
        return err_resp_dyn(StatusCode::BAD_REQUEST, e);
    }
    if parse_days_of_week(&new_days).is_empty() {
        return err_resp(
            StatusCode::BAD_REQUEST,
            "Cần chọn ít nhất một ngày trong tuần",
        );
    }
    if let Err(e) = parse_timezone_offset(&new_tz) {
        return err_resp_dyn(StatusCode::BAD_REQUEST, e);
    }

    let now = Utc::now().to_rfc3339();
    let block_manual_int = if new_block_manual { 1 } else { 0 };
    let enabled_int = if new_enabled { 1 } else { 0 };

    if let Err(e) = sqlx::query(
        r#"
        UPDATE m_quiet_hours_window
        SET name = ?, days_of_week = ?, start_time = ?, end_time = ?,
            timezone = ?, block_manual_runs = ?, enabled = ?, updated_at = ?
        WHERE id = ? AND workspace_id = ?
        "#,
    )
    .bind(&new_name)
    .bind(&new_days)
    .bind(&new_start)
    .bind(&new_end)
    .bind(&new_tz)
    .bind(block_manual_int)
    .bind(enabled_int)
    .bind(&now)
    .bind(&window_id)
    .bind(&ws_id)
    .execute(&pool)
    .await
    {
        return err_resp_dyn(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    let updated: Option<QuietWindow> = sqlx::query_as(
        r#"
        SELECT id, workspace_id, project_id, name, days_of_week, start_time,
               end_time, timezone, block_manual_runs, enabled, created_at, updated_at
        FROM m_quiet_hours_window
        WHERE id = ?
        "#,
    )
    .bind(&window_id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    Json(json!({ "window": updated })).into_response()
}

/// `DELETE /workspaces/{ws_id}/windows/{window_id}` - Xóa khung giờ cấm.
async fn delete_window(
    AxumPath((ws_id, window_id)): AxumPath<(String, String)>,
    State(state): State<QuietHoursState>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return r,
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Cần quyền quản trị viên để xóa khung giờ cấm",
        );
    }

    let res =
        match sqlx::query("DELETE FROM m_quiet_hours_window WHERE id = ? AND workspace_id = ?")
            .bind(&window_id)
            .bind(&ws_id)
            .execute(&pool)
            .await
        {
            Ok(r) => r,
            Err(e) => return err_resp_dyn(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

    if res.rows_affected() == 0 {
        return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy khung giờ cấm để xóa");
    }

    Json(json!({ "deleted": true })).into_response()
}

/// `POST /workspaces/{ws_id}/windows/{window_id}/toggle` - Bật/tắt nhanh khung giờ cấm.
async fn toggle_window(
    AxumPath((ws_id, window_id)): AxumPath<(String, String)>,
    State(state): State<QuietHoursState>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return r,
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Cần quyền quản trị viên để bật/tắt khung giờ cấm",
        );
    }

    let now = Utc::now().to_rfc3339();
    let res = match sqlx::query(
        r#"
        UPDATE m_quiet_hours_window
        SET enabled = CASE WHEN enabled = 1 THEN 0 ELSE 1 END,
            updated_at = ?
        WHERE id = ? AND workspace_id = ?
        "#,
    )
    .bind(&now)
    .bind(&window_id)
    .bind(&ws_id)
    .execute(&pool)
    .await
    {
        Ok(r) => r,
        Err(e) => return err_resp_dyn(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    if res.rows_affected() == 0 {
        return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy khung giờ cấm");
    }

    let updated: Option<QuietWindow> = sqlx::query_as(
        r#"
        SELECT id, workspace_id, project_id, name, days_of_week, start_time,
               end_time, timezone, block_manual_runs, enabled, created_at, updated_at
        FROM m_quiet_hours_window
        WHERE id = ?
        "#,
    )
    .bind(&window_id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    Json(json!({ "window": updated })).into_response()
}

/// Tham số truy vấn trạng thái quiet hours.
#[derive(Debug, Deserialize)]
pub struct QuietStatusQuery {
    /// Định danh dự án tùy chọn để kiểm tra cấu hình riêng của dự án.
    pub project_id: Option<String>,
}

/// `GET /workspaces/{ws_id}/status` - Kiểm tra xem hiện tại có trong khung giờ cấm không.
async fn get_quiet_status(
    AxumPath(ws_id): AxumPath<String>,
    Query(query): Query<QuietStatusQuery>,
    State(state): State<QuietHoursState>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return r,
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    if !can_access_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Không có quyền truy cập không gian làm việc này",
        );
    }

    let windows: Vec<QuietWindow> = match sqlx::query_as(
        r#"
        SELECT id, workspace_id, project_id, name, days_of_week, start_time,
               end_time, timezone, block_manual_runs, enabled, created_at, updated_at
        FROM m_quiet_hours_window
        WHERE workspace_id = ? AND (project_id = ? OR project_id IS NULL) AND enabled = 1
        "#,
    )
    .bind(&ws_id)
    .bind(&query.project_id)
    .fetch_all(&pool)
    .await
    {
        Ok(w) => w,
        Err(e) => return err_resp_dyn(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    let now = state.clock.now();
    match evaluate_windows(&windows, now, None) {
        Ok(status) => Json(json!({ "status": status })).into_response(),
        Err(e) => err_resp_dyn(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

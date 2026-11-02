//! Định nghĩa các route HTTP và bộ xử lý yêu cầu cho module `calendar`.

#![expect(
    clippy::too_many_lines,
    reason = "Xử lý các endpoint CRUD của lịch kèm xác thực BOLA và kiểm tra dữ liệu"
)]

use axum::Json;
use axum::Router;
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use chrono::Utc;
use dandori_module_api::{
    ModuleContext, SessionUser, can_access_workspace, can_administer_workspace, project_workspace,
    session_user,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

use crate::model::{CreateScheduleRequest, ScheduleDto, ScheduleRecord, UpdateScheduleRequest};
use crate::schedule::compute_next_fire;
use crate::worker::{Clock, RealClock, fire_single_schedule};

/// Trạng thái chia sẻ cho router của module `calendar`.
#[derive(Clone, Debug)]
pub struct CalendarState {
    /// Ngữ cảnh runtime của module.
    pub ctx: ModuleContext,
    /// Đồng hồ trừu tượng phục vụ kiểm thử và vận hành.
    pub clock: Arc<dyn Clock>,
}

impl CalendarState {
    /// Khởi tạo trạng thái mới từ ModuleContext.
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

/// Helper tạo phản hồi lỗi chuẩn.
fn err_resp(status: StatusCode, msg: &'static str) -> Response {
    (status, Json(json!({ "error": { "message": msg } }))).into_response()
}

/// Khởi tạo router cho module `calendar`.
pub fn create_routes(state: CalendarState) -> Router {
    Router::new()
        .route("/schedules", get(list_schedules).post(create_schedule))
        .route(
            "/schedules/{id}",
            get(get_schedule)
                .put(update_schedule)
                .delete(delete_schedule),
        )
        .route("/schedules/{id}/pause", post(pause_schedule))
        .route("/schedules/{id}/resume", post(resume_schedule))
        .route("/schedules/{id}/fire-now", post(fire_schedule_now))
        .with_state(state)
}

/// Tham số lọc danh sách lịch.
#[derive(Debug, Deserialize)]
pub struct ListSchedulesQuery {
    /// Lọc theo dự án.
    pub project_id: Option<String>,
    /// Lọc theo workspace.
    pub workspace_id: Option<String>,
}

/// `GET /schedules`: Lấy danh sách lịch định kỳ.
pub async fn list_schedules(
    State(state): State<CalendarState>,
    headers: HeaderMap,
    Query(query): Query<ListSchedulesQuery>,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có kết nối CSDL"),
    };
    let user = match session_user(pool, &headers).await {
        Some(u) => u,
        None => return err_resp(StatusCode::UNAUTHORIZED, "Chưa xác thực danh tính"),
    };

    let schedules = if let Some(prj_id) = &query.project_id {
        let ws_id = match project_workspace(pool, prj_id).await {
            Some(w) => w,
            None => return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy dự án"),
        };
        if !can_access_workspace(pool, &user, &ws_id).await {
            return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
        }

        sqlx::query_as::<_, ScheduleRecord>(
            "SELECT id, workspace_id, project_id, title, schedule_type, schedule_expr, timezone, \
             task_template, auto_dispatch, missed_policy, enabled, last_fire, next_fire, \
             created_at, updated_at \
             FROM m_calendar_schedule WHERE project_id = ? ORDER BY created_at DESC",
        )
        .bind(prj_id)
        .fetch_all(pool)
        .await
    } else if let Some(ws_id) = &query.workspace_id {
        if !can_access_workspace(pool, &user, ws_id).await {
            return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
        }

        sqlx::query_as::<_, ScheduleRecord>(
            "SELECT id, workspace_id, project_id, title, schedule_type, schedule_expr, timezone, \
             task_template, auto_dispatch, missed_policy, enabled, last_fire, next_fire, \
             created_at, updated_at \
             FROM m_calendar_schedule WHERE workspace_id = ? ORDER BY created_at DESC",
        )
        .bind(ws_id)
        .fetch_all(pool)
        .await
    } else {
        return err_resp(
            StatusCode::BAD_REQUEST,
            "Cần cung cấp project_id hoặc workspace_id",
        );
    };

    match schedules {
        Ok(list) => {
            let dtos: Vec<ScheduleDto> = list.iter().map(ScheduleRecord::to_dto).collect();
            Json(json!({ "schedules": dtos })).into_response()
        }
        Err(_) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Lỗi truy vấn lịch"),
    }
}

/// `POST /schedules`: Tạo mới một lịch định kỳ (chỉ Workspace Admin / System Admin).
pub async fn create_schedule(
    State(state): State<CalendarState>,
    headers: HeaderMap,
    Json(payload): Json<CreateScheduleRequest>,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có kết nối CSDL"),
    };
    let user = match session_user(pool, &headers).await {
        Some(u) => u,
        None => return err_resp(StatusCode::UNAUTHORIZED, "Chưa xác thực danh tính"),
    };

    let ws_id = match project_workspace(pool, &payload.project_id).await {
        Some(w) => w,
        None => return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy dự án"),
    };

    // Tiêu chuẩn phân quyền P2/P3: Tạo lịch tự động cần quyền Admin của workspace
    if !can_administer_workspace(pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Cần quyền quản trị viên workspace để tạo lịch",
        );
    }

    let tz_str = payload.timezone.as_deref().unwrap_or("UTC");
    let now = state.clock.now();

    let next_fire =
        match compute_next_fire(&payload.schedule_type, &payload.schedule_expr, tz_str, now) {
            Ok(nf) => nf,
            Err(err_msg) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": { "message": err_msg } })),
                )
                    .into_response();
            }
        };

    let sched_id = Uuid::new_v4().to_string();
    let template_json = match serde_json::to_string(&payload.task_template) {
        Ok(s) => s,
        Err(_) => return err_resp(StatusCode::BAD_REQUEST, "Mẫu nhiệm vụ không hợp lệ"),
    };

    let auto_dispatch_val = i64::from(payload.auto_dispatch.unwrap_or(false));
    let missed_policy_val = match payload.missed_policy.as_deref() {
        Some("run_once") => "run_once",
        _ => "skip",
    };

    let now_str = now.to_rfc3339();
    let next_fire_str = next_fire.to_rfc3339();

    let res = sqlx::query(
        r#"
        INSERT INTO m_calendar_schedule (
            id, workspace_id, project_id, title, schedule_type, schedule_expr,
            timezone, task_template, auto_dispatch, missed_policy, enabled,
            last_fire, next_fire, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, NULL, ?, ?, ?)
        "#,
    )
    .bind(&sched_id)
    .bind(&ws_id)
    .bind(&payload.project_id)
    .bind(&payload.title)
    .bind(&payload.schedule_type)
    .bind(&payload.schedule_expr)
    .bind(tz_str)
    .bind(&template_json)
    .bind(auto_dispatch_val)
    .bind(missed_policy_val)
    .bind(&next_fire_str)
    .bind(&now_str)
    .bind(&now_str)
    .execute(pool)
    .await;

    if res.is_err() {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Lỗi lưu lịch vào CSDL");
    }

    let record = ScheduleRecord {
        id: sched_id,
        workspace_id: ws_id,
        project_id: payload.project_id,
        title: payload.title,
        schedule_type: payload.schedule_type,
        schedule_expr: payload.schedule_expr,
        timezone: tz_str.to_string(),
        task_template: template_json,
        auto_dispatch: auto_dispatch_val,
        missed_policy: missed_policy_val.to_string(),
        enabled: 1,
        last_fire: None,
        next_fire: Some(next_fire_str),
        created_at: now_str.clone(),
        updated_at: now_str,
    };

    (
        StatusCode::CREATED,
        Json(json!({ "schedule": record.to_dto() })),
    )
        .into_response()
}

/// Lấy thông tin lịch và xác thực quyền truy cập.
async fn fetch_schedule_and_check_access(
    pool: &SqlitePool,
    user: &SessionUser,
    sched_id: &str,
    require_admin: bool,
) -> Result<ScheduleRecord, (StatusCode, &'static str)> {
    let sched: Option<ScheduleRecord> = sqlx::query_as(
        "SELECT id, workspace_id, project_id, title, schedule_type, schedule_expr, timezone, \
         task_template, auto_dispatch, missed_policy, enabled, last_fire, next_fire, \
         created_at, updated_at \
         FROM m_calendar_schedule WHERE id = ?",
    )
    .bind(sched_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi truy vấn lịch"))?;

    let sched = sched.ok_or((StatusCode::NOT_FOUND, "Không tìm thấy lịch"))?;

    if require_admin {
        if !can_administer_workspace(pool, user, &sched.workspace_id).await {
            return Err((StatusCode::FORBIDDEN, "Cần quyền quản trị viên workspace"));
        }
    } else if !can_access_workspace(pool, user, &sched.workspace_id).await {
        return Err((StatusCode::FORBIDDEN, "Không có quyền truy cập workspace"));
    }

    Ok(sched)
}

/// `GET /schedules/:id`: Xem chi tiết một lịch.
pub async fn get_schedule(
    State(state): State<CalendarState>,
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có kết nối CSDL"),
    };
    let user = match session_user(pool, &headers).await {
        Some(u) => u,
        None => return err_resp(StatusCode::UNAUTHORIZED, "Chưa xác thực danh tính"),
    };

    match fetch_schedule_and_check_access(pool, &user, &id, false).await {
        Ok(sched) => Json(json!({ "schedule": sched.to_dto() })).into_response(),
        Err((code, msg)) => err_resp(code, msg),
    }
}

/// `PUT /schedules/:id`: Cập nhật cấu hình lịch.
pub async fn update_schedule(
    State(state): State<CalendarState>,
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
    Json(payload): Json<UpdateScheduleRequest>,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có kết nối CSDL"),
    };
    let user = match session_user(pool, &headers).await {
        Some(u) => u,
        None => return err_resp(StatusCode::UNAUTHORIZED, "Chưa xác thực danh tính"),
    };

    let sched = match fetch_schedule_and_check_access(pool, &user, &id, true).await {
        Ok(s) => s,
        Err((code, msg)) => return err_resp(code, msg),
    };

    let title = payload.title.unwrap_or(sched.title);
    let sched_type = payload.schedule_type.unwrap_or(sched.schedule_type);
    let sched_expr = payload.schedule_expr.unwrap_or(sched.schedule_expr);
    let tz_str = payload.timezone.unwrap_or(sched.timezone);
    let auto_dispatch = payload.auto_dispatch.map_or(sched.auto_dispatch, i64::from);
    let missed_policy = payload.missed_policy.unwrap_or(sched.missed_policy);
    let enabled = payload.enabled.map_or(sched.enabled, i64::from);

    let template_json = if let Some(tmpl) = payload.task_template {
        serde_json::to_string(&tmpl).unwrap_or(sched.task_template)
    } else {
        sched.task_template
    };

    let now = state.clock.now();
    let next_fire = match compute_next_fire(&sched_type, &sched_expr, &tz_str, now) {
        Ok(nf) => nf.to_rfc3339(),
        Err(err_msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": { "message": err_msg } })),
            )
                .into_response();
        }
    };

    let now_str = now.to_rfc3339();
    let res = sqlx::query(
        r#"
        UPDATE m_calendar_schedule
        SET title = ?, schedule_type = ?, schedule_expr = ?, timezone = ?,
            task_template = ?, auto_dispatch = ?, missed_policy = ?, enabled = ?,
            next_fire = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&title)
    .bind(&sched_type)
    .bind(&sched_expr)
    .bind(&tz_str)
    .bind(&template_json)
    .bind(auto_dispatch)
    .bind(&missed_policy)
    .bind(enabled)
    .bind(&next_fire)
    .bind(&now_str)
    .bind(&id)
    .execute(pool)
    .await;

    if res.is_err() {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Lỗi cập nhật lịch");
    }

    let updated = ScheduleRecord {
        id,
        workspace_id: sched.workspace_id,
        project_id: sched.project_id,
        title,
        schedule_type: sched_type,
        schedule_expr: sched_expr,
        timezone: tz_str,
        task_template: template_json,
        auto_dispatch,
        missed_policy,
        enabled,
        last_fire: sched.last_fire,
        next_fire: Some(next_fire),
        created_at: sched.created_at,
        updated_at: now_str,
    };

    Json(json!({ "schedule": updated.to_dto() })).into_response()
}

/// `DELETE /schedules/:id`: Xóa một lịch định kỳ.
pub async fn delete_schedule(
    State(state): State<CalendarState>,
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có kết nối CSDL"),
    };
    let user = match session_user(pool, &headers).await {
        Some(u) => u,
        None => return err_resp(StatusCode::UNAUTHORIZED, "Chưa xác thực danh tính"),
    };

    if let Err((code, msg)) = fetch_schedule_and_check_access(pool, &user, &id, true).await {
        return err_resp(code, msg);
    }

    let res = sqlx::query("DELETE FROM m_calendar_schedule WHERE id = ?")
        .bind(&id)
        .execute(pool)
        .await;

    match res {
        Ok(_) => Json(json!({ "success": true })).into_response(),
        Err(_) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Lỗi xóa lịch"),
    }
}

/// `POST /schedules/:id/pause`: Tạm dừng kích hoạt lịch.
pub async fn pause_schedule(
    State(state): State<CalendarState>,
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có kết nối CSDL"),
    };
    let user = match session_user(pool, &headers).await {
        Some(u) => u,
        None => return err_resp(StatusCode::UNAUTHORIZED, "Chưa xác thực danh tính"),
    };

    if let Err((code, msg)) = fetch_schedule_and_check_access(pool, &user, &id, true).await {
        return err_resp(code, msg);
    }

    let now_str = Utc::now().to_rfc3339();
    let res =
        sqlx::query("UPDATE m_calendar_schedule SET enabled = 0, updated_at = ? WHERE id = ?")
            .bind(&now_str)
            .bind(&id)
            .execute(pool)
            .await;

    match res {
        Ok(_) => Json(json!({ "success": true, "enabled": false })).into_response(),
        Err(_) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Lỗi tạm dừng lịch"),
    }
}

/// `POST /schedules/:id/resume`: Khôi phục kích hoạt lịch.
pub async fn resume_schedule(
    State(state): State<CalendarState>,
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có kết nối CSDL"),
    };
    let user = match session_user(pool, &headers).await {
        Some(u) => u,
        None => return err_resp(StatusCode::UNAUTHORIZED, "Chưa xác thực danh tính"),
    };

    let sched = match fetch_schedule_and_check_access(pool, &user, &id, true).await {
        Ok(s) => s,
        Err((code, msg)) => return err_resp(code, msg),
    };

    let now = state.clock.now();
    let next_fire = match compute_next_fire(
        &sched.schedule_type,
        &sched.schedule_expr,
        &sched.timezone,
        now,
    ) {
        Ok(nf) => nf.to_rfc3339(),
        Err(err_msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": { "message": err_msg } })),
            )
                .into_response();
        }
    };

    let now_str = now.to_rfc3339();
    let res = sqlx::query(
        "UPDATE m_calendar_schedule SET enabled = 1, next_fire = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&next_fire)
    .bind(&now_str)
    .bind(&id)
    .execute(pool)
    .await;

    match res {
        Ok(_) => Json(json!({ "success": true, "enabled": true, "next_fire": next_fire }))
            .into_response(),
        Err(_) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Lỗi khôi phục lịch"),
    }
}

/// `POST /schedules/:id/fire-now`: Kích hoạt lịch ngay lập tức.
pub async fn fire_schedule_now(
    State(state): State<CalendarState>,
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có kết nối CSDL"),
    };
    let user = match session_user(pool, &headers).await {
        Some(u) => u,
        None => return err_resp(StatusCode::UNAUTHORIZED, "Chưa xác thực danh tính"),
    };

    let sched = match fetch_schedule_and_check_access(pool, &user, &id, true).await {
        Ok(s) => s,
        Err((code, msg)) => return err_resp(code, msg),
    };

    let now = state.clock.now();
    match fire_single_schedule(pool, &*state.clock, &sched, now, state.ctx.event_bus()).await {
        Ok(true) => Json(json!({ "success": true, "message": "Đã kích hoạt lịch thành công" }))
            .into_response(),
        Ok(false) => err_resp(
            StatusCode::CONFLICT,
            "Không thể kích hoạt (lịch đang xử lý hoặc không có next_fire)",
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": { "message": e } })),
        )
            .into_response(),
    }
}

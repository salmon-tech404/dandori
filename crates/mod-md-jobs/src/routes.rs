//! Định nghĩa HTTP API Router cho module `md-jobs`.

#![expect(
    clippy::too_many_lines,
    reason = "Xử lý các endpoint danh sách, đồng bộ, phê duyệt và kích hoạt job kèm phân quyền"
)]

use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use croner::Cron;
use dandori_module_api::{
    ModuleContext, SessionUser, can_access_workspace, can_administer_workspace, project_workspace,
    session_user,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::model::{MdJobDto, MdJobRecord};
use crate::scanner::sync_project_jobs;
use crate::worker::{Clock, RealClock, fire_single_md_job};

/// Trạng thái chia sẻ cho router `md-jobs`.
#[derive(Clone, Debug)]
pub struct MdJobsState {
    /// Ngữ cảnh runtime của module.
    pub ctx: ModuleContext,
    /// Đồng hồ thời gian.
    pub clock: Arc<dyn Clock>,
}

impl MdJobsState {
    /// Khởi tạo trạng thái mới.
    #[must_use]
    pub fn new(ctx: ModuleContext) -> Self {
        Self {
            ctx,
            clock: Arc::new(RealClock),
        }
    }

    /// Khởi tạo trạng thái với đồng hồ tùy chỉnh cho kiểm thử.
    #[must_use]
    pub fn with_clock(ctx: ModuleContext, clock: Arc<dyn Clock>) -> Self {
        Self { ctx, clock }
    }
}

fn err_resp(status: StatusCode, msg: &'static str) -> Response {
    (status, Json(json!({ "error": { "message": msg } }))).into_response()
}

/// Khởi tạo router cho module `md-jobs`.
pub fn create_routes(state: MdJobsState) -> Router {
    Router::new()
        .route("/jobs", get(list_jobs))
        .route("/sync", post(sync_jobs))
        .route("/jobs/{id}/approve", post(approve_job))
        .route("/jobs/{id}/reject", post(reject_job))
        .route("/jobs/{id}/fire-now", post(fire_job_now))
        .with_state(state)
}

/// Tham số lọc danh sách job.
#[derive(Debug, Deserialize)]
pub struct ListJobsQuery {
    /// Lọc theo định danh dự án.
    pub project_id: Option<String>,
}

/// `GET /jobs`: Lấy danh sách job trong dự án.
pub async fn list_jobs(
    State(state): State<MdJobsState>,
    headers: HeaderMap,
    Query(query): Query<ListJobsQuery>,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có kết nối CSDL"),
    };
    let user = match session_user(pool, &headers).await {
        Some(u) => u,
        None => return err_resp(StatusCode::UNAUTHORIZED, "Chưa xác thực danh tính"),
    };

    let prj_id = match query.project_id {
        Some(ref id) => id,
        None => return err_resp(StatusCode::BAD_REQUEST, "Thiếu tham số project_id"),
    };

    let ws_id = match project_workspace(pool, prj_id).await {
        Some(w) => w,
        None => return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy dự án"),
    };

    // Kiểm tra quyền xem workspace
    if !can_access_workspace(pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Không có quyền truy cập workspace này",
        );
    }

    let jobs_res: Result<Vec<MdJobRecord>, _> = sqlx::query_as(
        "SELECT id, workspace_id, project_id, file_path, job_name, schedule_cron, \
         agent_role, account_id, prompt, content_hash, approved_hash, status, \
         next_fire, last_fire, created_at, updated_at \
         FROM m_md_job WHERE project_id = ? ORDER BY created_at DESC",
    )
    .bind(prj_id)
    .fetch_all(pool)
    .await;

    match jobs_res {
        Ok(list) => {
            let dtos: Vec<MdJobDto> = list.iter().map(MdJobRecord::to_dto).collect();
            Json(json!({ "jobs": dtos })).into_response()
        }
        Err(_) => err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Lỗi truy vấn danh sách job",
        ),
    }
}

/// Tham số yêu cầu đồng bộ job.
#[derive(Debug, Deserialize)]
pub struct SyncJobsRequest {
    /// Định danh dự án cần quét đồng bộ.
    pub project_id: String,
}

/// `POST /sync`: Kích hoạt quét và đồng bộ tệp `.cron/jobs/*.md` theo yêu cầu.
pub async fn sync_jobs(
    State(state): State<MdJobsState>,
    headers: HeaderMap,
    Json(payload): Json<SyncJobsRequest>,
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

    if !can_access_workspace(pool, &user, &ws_id).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
    }

    // Lấy repo_path của dự án
    let prj_row = sqlx::query("SELECT repo_path FROM project WHERE id = ?")
        .bind(&payload.project_id)
        .fetch_optional(pool)
        .await;

    let repo_path: String = match prj_row {
        Ok(Some(row)) => {
            use sqlx::Row;
            row.get("repo_path")
        }
        _ => return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy dự án hoặc repo_path"),
    };

    match sync_project_jobs(pool, &ws_id, &payload.project_id, &repo_path).await {
        Ok(count) => Json(json!({ "success": true, "scanned_files": count })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": { "message": e } })),
        )
            .into_response(),
    }
}

/// Lấy thông tin job và kiểm tra quyền admin của workspace.
async fn fetch_job_and_check_admin(
    pool: &SqlitePool,
    user: &SessionUser,
    job_id: &str,
) -> Result<MdJobRecord, (StatusCode, &'static str)> {
    let job: Option<MdJobRecord> = sqlx::query_as(
        "SELECT id, workspace_id, project_id, file_path, job_name, schedule_cron, \
         agent_role, account_id, prompt, content_hash, approved_hash, status, \
         next_fire, last_fire, created_at, updated_at \
         FROM m_md_job WHERE id = ?",
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi truy vấn CSDL"))?;

    let job = match job {
        Some(j) => j,
        None => return Err((StatusCode::NOT_FOUND, "Không tìm thấy job")),
    };

    if !can_administer_workspace(pool, user, &job.workspace_id).await {
        return Err((
            StatusCode::FORBIDDEN,
            "Cần quyền Workspace Admin để thao tác",
        ));
    }

    Ok(job)
}

/// `POST /jobs/{id}/approve`: Chủ dự án phê duyệt công việc.
pub async fn approve_job(
    State(state): State<MdJobsState>,
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

    let job = match fetch_job_and_check_admin(pool, &user, &id).await {
        Ok(j) => j,
        Err((code, msg)) => return err_resp(code, msg),
    };

    if job.status == "syntax_error" {
        return err_resp(
            StatusCode::BAD_REQUEST,
            "Không thể duyệt job đang có lỗi cú pháp",
        );
    }

    // Tính next_fire từ cron
    let now = state.clock.now();
    let cron = match Cron::new(&job.schedule_cron).parse() {
        Ok(c) => c,
        Err(_) => return err_resp(StatusCode::BAD_REQUEST, "Biểu thức cron không hợp lệ"),
    };

    let next_fire = match cron.find_next_occurrence(&now, false) {
        Ok(nf) => nf.to_rfc3339(),
        Err(_) => {
            return err_resp(
                StatusCode::BAD_REQUEST,
                "Không thể tính thời điểm kích hoạt tiếp theo",
            );
        }
    };

    let now_str = now.to_rfc3339();
    let approved_hash = job.content_hash.clone();

    let res = sqlx::query(
        r#"
        UPDATE m_md_job
        SET approved_hash = ?, status = 'approved', next_fire = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&approved_hash)
    .bind(&next_fire)
    .bind(&now_str)
    .bind(&id)
    .execute(pool)
    .await;

    match res {
        Ok(_) => Json(json!({
            "success": true,
            "status": "approved",
            "approved_hash": approved_hash,
            "next_fire": next_fire
        }))
        .into_response(),
        Err(_) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Lỗi cập nhật duyệt job"),
    }
}

/// `POST /jobs/{id}/reject`: Từ chối hoặc hủy duyệt công việc.
pub async fn reject_job(
    State(state): State<MdJobsState>,
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

    if let Err((code, msg)) = fetch_job_and_check_admin(pool, &user, &id).await {
        return err_resp(code, msg);
    }

    let now_str = Utc::now().to_rfc3339();
    let res = sqlx::query(
        "UPDATE m_md_job SET status = 'rejected', next_fire = NULL, updated_at = ? WHERE id = ?",
    )
    .bind(&now_str)
    .bind(&id)
    .execute(pool)
    .await;

    match res {
        Ok(_) => Json(json!({ "success": true, "status": "rejected" })).into_response(),
        Err(_) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Lỗi từ chối job"),
    }
}

/// `POST /jobs/{id}/fire-now`: Kích hoạt thủ công ngay lập tức một job đã được duyệt.
pub async fn fire_job_now(
    State(state): State<MdJobsState>,
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

    let job = match fetch_job_and_check_admin(pool, &user, &id).await {
        Ok(j) => j,
        Err((code, msg)) => return err_resp(code, msg),
    };

    if job.status != "approved" {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Chỉ job đã được duyệt mới có thể kích hoạt",
        );
    }

    let now = state.clock.now();
    match fire_single_md_job(pool, &job, now, state.ctx.event_bus()).await {
        Ok(true) => Json(json!({ "success": true, "message": "Đã kích hoạt job thành công" }))
            .into_response(),
        Ok(false) => err_resp(
            StatusCode::CONFLICT,
            "Job đang được xử lý bởi lượt kích hoạt khác",
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": { "message": e } })),
        )
            .into_response(),
    }
}

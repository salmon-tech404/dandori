//! Các route API điều phối lần chạy của task (P1-01).

use std::str::FromStr;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use dandori_agent_port::AgentRole;
use dandori_types::{AgentAccountId, RunId, TaskId};
use serde::Deserialize;

use crate::error::ApiError;
use crate::middleware::AuthUser;
use crate::run_service::RunServiceError;
use crate::state::AppState;

/// Tham số body yêu cầu bắt đầu một lần chạy task.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct StartRunPayload {
    /// Vai trò của agent (mặc định là `worker`).
    pub role: Option<String>,
    /// Danh sách ID tài khoản bị loại trừ điều phối (nếu có).
    pub exclude_accounts: Option<Vec<String>>,
}

/// Tham số phân trang danh sách sự kiện lần chạy.
#[derive(Debug, Clone, Deserialize)]
pub struct ListEventsQuery {
    /// Số lượng tối đa bản ghi trả về (mặc định 50, tối đa 200).
    pub limit: Option<i64>,
    /// Vị trí dịch chuyển offset (mặc định 0).
    pub offset: Option<i64>,
}

/// Khởi động một lần chạy mới cho task.
pub async fn start_run_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(task_id_str): Path<String>,
    payload: Option<Json<StartRunPayload>>,
) -> Result<impl IntoResponse, ApiError> {
    let task_id = TaskId::from_str(&task_id_str)
        .map_err(|e| ApiError::BadRequest(format!("TaskId không hợp lệ: {e}")))?;

    let payload = payload.map(|Json(p)| p).unwrap_or_default();
    let role = match payload.role.as_deref() {
        Some("lead") => AgentRole::Lead,
        Some("worker") | None => AgentRole::Worker,
        Some(custom) => AgentRole::Custom(custom.to_string()),
    };

    let mut exclude_accounts = Vec::new();
    if let Some(list) = payload.exclude_accounts {
        for acc_str in list {
            if let Ok(acc_id) = AgentAccountId::from_str(&acc_str) {
                exclude_accounts.push(acc_id);
            }
        }
    }

    match state
        .run_service
        .start_run(task_id, role, user.id, exclude_accounts)
        .await
    {
        Ok(run) => Ok((
            StatusCode::CREATED,
            Json(serde_json::json!({
                "run_id": run.id,
                "status": run.status,
                "agent": run.agent,
                "account": run.account
            })),
        )),
        Err(e) => Err(map_start_run_error(e)),
    }
}

/// Ánh xạ lỗi bắt đầu lần chạy sang lỗi API (dùng chung cho `POST /tasks/{id}/runs` và `dispatch`).
#[must_use]
pub(crate) fn map_start_run_error(err: RunServiceError) -> ApiError {
    match err {
        RunServiceError::WorkdirProviderMissing => ApiError::WorkdirProviderMissing,
        RunServiceError::NoAvailableAccount => ApiError::NoAvailableAccount,
        RunServiceError::ConcurrencyLimitExceeded(max) => ApiError::ConcurrencyLimitReached(
            format!("Vượt quá số lần chạy đồng thời tối đa ({max})"),
        ),
        RunServiceError::TaskNotFound(id) => ApiError::NotFound(format!("task: {id}")),
        RunServiceError::Forbidden(msg) => ApiError::Forbidden(msg),
        RunServiceError::TaskAlreadyRunning(_) => ApiError::BadRequest("Task đang chạy".into()),
        RunServiceError::TaskAlreadyCompleted(_) => {
            ApiError::BadRequest("Task đã hoàn thành".into())
        }
        RunServiceError::InvalidTaskTransition(s) => {
            ApiError::BadRequest(format!("Không thể chạy task ở trạng thái: {s}"))
        }
        RunServiceError::WorkdirError(e) => {
            ApiError::Internal(format!("Lỗi thư mục làm việc: {e}"))
        }
        RunServiceError::AdapterNotFound(prov) => {
            ApiError::BadRequest(format!("Không tìm thấy adapter cho nhà cung cấp: {prov}"))
        }
        other => ApiError::Internal(other.to_string()),
    }
}

/// Hủy bỏ một lần chạy đang thực thi.
pub async fn cancel_run_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(run_id_str): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let run_id = RunId::from_str(&run_id_str)
        .map_err(|e| ApiError::BadRequest(format!("RunId không hợp lệ: {e}")))?;

    match state.run_service.cancel_run(run_id, user.id).await {
        Ok(()) => Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "cancelling",
                "run_id": run_id
            })),
        )),
        Err(RunServiceError::RunNotFound(id)) => Err(ApiError::NotFound(format!("run: {id}"))),
        Err(RunServiceError::Forbidden(msg)) => Err(ApiError::Forbidden(msg)),
        Err(other) => Err(ApiError::Internal(other.to_string())),
    }
}

/// Lấy thông tin chi tiết một lần chạy.
pub async fn get_run_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(run_id_str): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let run_id = RunId::from_str(&run_id_str)
        .map_err(|e| ApiError::BadRequest(format!("RunId không hợp lệ: {e}")))?;

    match state.run_service.get_run(run_id, user.id).await {
        Ok(run) => Ok((StatusCode::OK, Json(serde_json::json!({ "run": run })))),
        Err(RunServiceError::RunNotFound(id)) => Err(ApiError::NotFound(format!("run: {id}"))),
        Err(RunServiceError::Forbidden(msg)) => Err(ApiError::Forbidden(msg)),
        Err(other) => Err(ApiError::Internal(other.to_string())),
    }
}

/// Lấy danh sách sự kiện phân trang của một lần chạy.
pub async fn list_run_events_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(run_id_str): Path<String>,
    Query(query): Query<ListEventsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let run_id = RunId::from_str(&run_id_str)
        .map_err(|e| ApiError::BadRequest(format!("RunId không hợp lệ: {e}")))?;

    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let offset = query.offset.unwrap_or(0).max(0);

    match state
        .run_service
        .list_run_events(run_id, user.id, limit, offset)
        .await
    {
        Ok((events, total)) => Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "events": events,
                "total": total,
                "limit": limit,
                "offset": offset
            })),
        )),
        Err(RunServiceError::RunNotFound(id)) => Err(ApiError::NotFound(format!("run: {id}"))),
        Err(RunServiceError::Forbidden(msg)) => Err(ApiError::Forbidden(msg)),
        Err(other) => Err(ApiError::Internal(other.to_string())),
    }
}

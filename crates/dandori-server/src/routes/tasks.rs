//! Các route xử lý nhiệm vụ (Task) và máy trạng thái Kanban (T11).

use std::str::FromStr;

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use chrono::Utc;
use dandori_db::{ProjectRepo, TaskRepo};
use dandori_types::{
    Envelope, EventPayload, EventScope, Project, ProjectId, Task, TaskId, TaskStatus,
};
use serde::Deserialize;
use serde_json::json;

use crate::error::ApiError;
use crate::middleware::{AuthUser, RequestId};
use crate::routes::workspaces::check_workspace_access;
use crate::state::AppState;

/// Tham số phân trang cho danh sách task.
#[derive(Debug, Deserialize)]
pub struct TaskPaginationQuery {
    /// Số trang hiện tại (bắt đầu từ 1).
    pub page: Option<i64>,
    /// Số phần tử trên mỗi trang (mặc định 20, tối đa 100).
    pub per_page: Option<i64>,
}

/// Yêu cầu tạo mới nhiệm vụ.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTaskRequest {
    /// Tiêu đề nhiệm vụ.
    pub title: String,
    /// Mô tả chi tiết yêu cầu.
    pub description: String,
}

/// Yêu cầu cập nhật thông tin nhiệm vụ.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateTaskRequest {
    /// Tiêu đề mới của nhiệm vụ.
    pub title: String,
    /// Mô tả mới của nhiệm vụ.
    pub description: String,
}

/// Yêu cầu chuyển trạng thái nhiệm vụ.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateTaskStatusRequest {
    /// Trạng thái đích muốn chuyển tới.
    pub status: String,
}

async fn get_task_and_verify_access(
    state: &AppState,
    auth_user: &AuthUser,
    task_id: &TaskId,
) -> Result<(Task, Project), ApiError> {
    let task_repo = TaskRepo::new(&state.db);
    let task = task_repo
        .get_task_by_id(task_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Nhiệm vụ".to_string()))?;

    let proj_repo = ProjectRepo::new(&state.db);
    let project = proj_repo
        .get_project_by_id(&task.project_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Dự án sở hữu nhiệm vụ".to_string()))?;

    check_workspace_access(state, auth_user, &project.workspace_id, false).await?;
    Ok((task, project))
}

async fn find_project_and_check_access(
    state: &AppState,
    auth_user: &AuthUser,
    proj_id: &ProjectId,
) -> Result<Project, ApiError> {
    let proj_repo = ProjectRepo::new(&state.db);
    let project = proj_repo
        .get_project_by_id(proj_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Dự án".to_string()))?;

    check_workspace_access(state, auth_user, &project.workspace_id, false).await?;
    Ok(project)
}

/// `POST /api/v1/projects/:id/tasks` - Tạo nhiệm vụ mới trong dự án.
pub async fn create_task(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Path(proj_str): Path<String>,
    Json(payload): Json<CreateTaskRequest>,
) -> Response {
    let proj_id = match ProjectId::from_str(&proj_str) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID dự án không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    let project = match find_project_and_check_access(&state, &auth_user, &proj_id).await {
        Ok(p) => p,
        Err(err) => return err.into_response_with_id(&req_id.0),
    };

    let title = payload.title.trim();
    if title.is_empty() {
        return ApiError::BadRequest("Tiêu đề nhiệm vụ không được để trống".to_string())
            .into_response_with_id(&req_id.0);
    }

    let task_id = TaskId::new();
    let now = Utc::now();
    let task = Task {
        id: task_id,
        project_id: proj_id,
        title: title.to_string(),
        description: payload.description,
        status: TaskStatus::Backlog.as_str().to_string(),
        lead_agent: String::new(),
        worker_agent: String::new(),
        workdir_path: None,
        created_at: now,
        updated_at: now,
    };

    let task_repo = TaskRepo::new(&state.db);
    if let Err(e) = task_repo.create_task(&task).await {
        return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0);
    }

    state.event_bus.publish(Envelope::new(
        now,
        EventScope::task(project.workspace_id, proj_id, task_id),
        EventPayload::TaskCreated {
            task_id,
            title: title.to_string(),
        },
    ));

    Json(json!({ "task": task })).into_response()
}

/// `GET /api/v1/projects/:id/tasks` - Danh sách nhiệm vụ phân trang.
pub async fn list_tasks(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Path(proj_str): Path<String>,
    Query(query): Query<TaskPaginationQuery>,
) -> Response {
    let proj_id = match ProjectId::from_str(&proj_str) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID dự án không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    if let Err(err) = find_project_and_check_access(&state, &auth_user, &proj_id).await {
        return err.into_response_with_id(&req_id.0);
    }

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let task_repo = TaskRepo::new(&state.db);
    match task_repo
        .list_tasks_paginated(&proj_id, per_page, offset)
        .await
    {
        Ok((tasks, total)) => Json(json!({
            "tasks": tasks,
            "total": total,
            "page": page,
            "per_page": per_page
        }))
        .into_response(),
        Err(e) => ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    }
}

/// `GET /api/v1/tasks/:id` - Lấy thông tin chi tiết nhiệm vụ.
pub async fn get_task(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Path(task_str): Path<String>,
) -> Response {
    let task_id = match TaskId::from_str(&task_str) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID nhiệm vụ không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    match get_task_and_verify_access(&state, &auth_user, &task_id).await {
        Ok((task, _)) => Json(json!({ "task": task })).into_response(),
        Err(err) => err.into_response_with_id(&req_id.0),
    }
}

/// `PATCH /api/v1/tasks/:id` - Cập nhật tiêu đề và mô tả nhiệm vụ.
pub async fn update_task(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Path(task_str): Path<String>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Response {
    let task_id = match TaskId::from_str(&task_str) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID nhiệm vụ không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    let (task, _) = match get_task_and_verify_access(&state, &auth_user, &task_id).await {
        Ok(res) => res,
        Err(err) => return err.into_response_with_id(&req_id.0),
    };

    let title = payload.title.trim();
    if title.is_empty() {
        return ApiError::BadRequest("Tiêu đề nhiệm vụ không được để trống".to_string())
            .into_response_with_id(&req_id.0);
    }

    let task_repo = TaskRepo::new(&state.db);
    if let Err(e) = task_repo
        .update_task(&task.project_id, &task_id, title, &payload.description)
        .await
    {
        return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0);
    }

    match task_repo.get_task_by_id(&task_id).await {
        Ok(Some(updated)) => Json(json!({ "task": updated })).into_response(),
        Ok(None) => ApiError::NotFound("Nhiệm vụ".to_string()).into_response_with_id(&req_id.0),
        Err(e) => ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    }
}

async fn execute_task_transition(
    state: &AppState,
    task: &Task,
    project: &Project,
    target_status: TaskStatus,
) -> Result<Task, ApiError> {
    let current_status = TaskStatus::from_str(&task.status).map_err(ApiError::Internal)?;

    if !current_status.can_transition_to(target_status) {
        return Err(ApiError::BadRequest(format!(
            "Không thể chuyển trạng thái từ '{}' sang '{}' (chuyển đổi không hợp lệ)",
            current_status.as_str(),
            target_status.as_str()
        )));
    }

    let task_repo = TaskRepo::new(&state.db);
    task_repo
        .update_task_status(&task.project_id, &task.id, target_status.as_str())
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    state.event_bus.publish(Envelope::new(
        Utc::now(),
        EventScope::task(project.workspace_id, task.project_id, task.id),
        EventPayload::TaskStatusChanged {
            task_id: task.id,
            old_status: current_status.as_str().to_string(),
            new_status: target_status.as_str().to_string(),
        },
    ));

    task_repo
        .get_task_by_id(&task.id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Nhiệm vụ".to_string()))
}

/// `POST /api/v1/tasks/:id/status` - Chuyển trạng thái theo bảng máy trạng thái (T11 AC3).
pub async fn update_task_status(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Path(task_str): Path<String>,
    Json(payload): Json<UpdateTaskStatusRequest>,
) -> Response {
    let task_id = match TaskId::from_str(&task_str) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID nhiệm vụ không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    let (task, project) = match get_task_and_verify_access(&state, &auth_user, &task_id).await {
        Ok(res) => res,
        Err(err) => return err.into_response_with_id(&req_id.0),
    };

    let target_status = match TaskStatus::from_str(&payload.status) {
        Ok(st) => st,
        Err(err_msg) => {
            return ApiError::BadRequest(err_msg).into_response_with_id(&req_id.0);
        }
    };

    match execute_task_transition(&state, &task, &project, target_status).await {
        Ok(updated) => Json(json!({ "task": updated })).into_response(),
        Err(err) => err.into_response_with_id(&req_id.0),
    }
}

/// `DELETE /api/v1/tasks/:id` - Xóa một nhiệm vụ.
pub async fn delete_task(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Path(task_str): Path<String>,
) -> Response {
    let task_id = match TaskId::from_str(&task_str) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID nhiệm vụ không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    let (task, _) = match get_task_and_verify_access(&state, &auth_user, &task_id).await {
        Ok(res) => res,
        Err(err) => return err.into_response_with_id(&req_id.0),
    };

    let task_repo = TaskRepo::new(&state.db);
    if let Err(e) = task_repo.delete_task(&task.project_id, &task_id).await {
        return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0);
    }

    Json(json!({ "status": "deleted" })).into_response()
}

//! Các route xử lý dự án (Project) và kiểm tra an toàn đường dẫn git (T11).

use std::path::Path as FsPath;
use std::str::FromStr;

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use chrono::Utc;
use dandori_db::ProjectRepo;
use dandori_types::{Project, ProjectId, WorkspaceId};
use serde::Deserialize;
use serde_json::json;

use crate::error::ApiError;
use crate::middleware::{AuthUser, RequestId};
use crate::routes::workspaces::check_workspace_access;
use crate::state::AppState;

/// Yêu cầu tạo mới dự án.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateProjectRequest {
    /// Tên dự án.
    pub name: String,
    /// Đường dẫn thư mục kho lưu trữ git trên máy cục bộ.
    pub repo_path: String,
    /// Nhánh mặc định (ví dụ `main` hoặc `master`).
    pub default_branch: String,
}

/// Yêu cầu cập nhật thông tin dự án.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateProjectRequest {
    /// Tên dự án.
    pub name: String,
    /// Nhánh mặc định.
    pub default_branch: String,
}

/// Kiểm tra tính hợp lệ và an toàn của đường dẫn repository (T11 AC2).
pub fn validate_repo_path(repo_path: &str) -> Result<(), String> {
    let trimmed = repo_path.trim();
    if trimmed.is_empty() {
        return Err("Đường dẫn repo không được để trống".to_string());
    }

    // Chống tấn công duyệt thư mục (path traversal)
    if trimmed.contains("..") {
        return Err("Đường dẫn repo không được chứa ký tự chuyển thư mục '..'".to_string());
    }

    let p = FsPath::new(trimmed);
    if !p.exists() {
        return Err(format!("Đường dẫn thư mục '{trimmed}' không tồn tại"));
    }

    if !p.is_dir() {
        return Err(format!("Đường dẫn '{trimmed}' không phải là một thư mục"));
    }

    let git_dir = p.join(".git");
    if !git_dir.exists() {
        return Err(format!(
            "Thư mục '{trimmed}' không phải là một kho lưu trữ Git hợp lệ (thiếu thư mục .git)"
        ));
    }

    Ok(())
}

/// `POST /api/v1/workspaces/:id/projects` - Tạo dự án mới trong workspace.
pub async fn create_project(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Path(ws_str): Path<String>,
    Json(payload): Json<CreateProjectRequest>,
) -> Response {
    let ws_id = match WorkspaceId::from_str(&ws_str) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID không gian làm việc không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    if let Err(err) = check_workspace_access(&state, &auth_user, &ws_id, false).await {
        return err.into_response_with_id(&req_id.0);
    }

    let name = payload.name.trim();
    if name.is_empty() {
        return ApiError::BadRequest("Tên dự án không được để trống".to_string())
            .into_response_with_id(&req_id.0);
    }

    let default_branch = payload.default_branch.trim();
    if default_branch.is_empty() {
        return ApiError::BadRequest("Nhánh mặc định không được để trống".to_string())
            .into_response_with_id(&req_id.0);
    }
    if dandori_git::validate_ref_name(default_branch).is_err() {
        return ApiError::BadRequest("Tên nhánh mặc định không hợp lệ".to_string())
            .into_response_with_id(&req_id.0);
    }

    if let Err(validation_err) = validate_repo_path(&payload.repo_path) {
        return ApiError::BadRequest(validation_err).into_response_with_id(&req_id.0);
    }

    let project = Project {
        id: ProjectId::new(),
        workspace_id: ws_id,
        name: name.to_string(),
        repo_path: payload.repo_path.trim().to_string(),
        default_branch: default_branch.to_string(),
        created_at: Utc::now(),
    };

    let repo = ProjectRepo::new(&state.db);
    if let Err(e) = repo.create_project(&project).await {
        return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0);
    }

    Json(json!({ "project": project })).into_response()
}

/// `GET /api/v1/workspaces/:id/projects` - Danh sách dự án trong workspace.
pub async fn list_projects(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Path(ws_str): Path<String>,
) -> Response {
    let ws_id = match WorkspaceId::from_str(&ws_str) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID không gian làm việc không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    if let Err(err) = check_workspace_access(&state, &auth_user, &ws_id, false).await {
        return err.into_response_with_id(&req_id.0);
    }

    let repo = ProjectRepo::new(&state.db);
    match repo.list_projects(&ws_id).await {
        Ok(projects) => Json(json!({ "projects": projects })).into_response(),
        Err(e) => ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    }
}

/// `GET /api/v1/projects/:id` - Lấy thông tin chi tiết một dự án.
pub async fn get_project(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Path(proj_str): Path<String>,
) -> Response {
    let proj_id = match ProjectId::from_str(&proj_str) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID dự án không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    let repo = ProjectRepo::new(&state.db);
    let project = match repo.get_project_by_id(&proj_id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return ApiError::NotFound("Dự án".to_string()).into_response_with_id(&req_id.0);
        }
        Err(e) => return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    };

    if let Err(err) = check_workspace_access(&state, &auth_user, &project.workspace_id, false).await
    {
        return err.into_response_with_id(&req_id.0);
    }

    Json(json!({ "project": project })).into_response()
}

/// `PATCH /api/v1/projects/:id` - Cập nhật tên và nhánh mặc định của dự án.
pub async fn update_project(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Path(proj_str): Path<String>,
    Json(payload): Json<UpdateProjectRequest>,
) -> Response {
    let proj_id = match ProjectId::from_str(&proj_str) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID dự án không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    let repo = ProjectRepo::new(&state.db);
    let project = match repo.get_project_by_id(&proj_id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return ApiError::NotFound("Dự án".to_string()).into_response_with_id(&req_id.0);
        }
        Err(e) => return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    };

    if let Err(err) = check_workspace_access(&state, &auth_user, &project.workspace_id, false).await
    {
        return err.into_response_with_id(&req_id.0);
    }

    let name = payload.name.trim();
    let default_branch = payload.default_branch.trim();
    if name.is_empty() || default_branch.is_empty() {
        return ApiError::BadRequest("Tên dự án và nhánh mặc định không được để trống".to_string())
            .into_response_with_id(&req_id.0);
    }
    if dandori_git::validate_ref_name(default_branch).is_err() {
        return ApiError::BadRequest("Tên nhánh mặc định không hợp lệ".to_string())
            .into_response_with_id(&req_id.0);
    }

    if let Err(e) = repo
        .update_project(&project.workspace_id, &proj_id, name, default_branch)
        .await
    {
        return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0);
    }

    match repo.get_project_by_id(&proj_id).await {
        Ok(Some(updated)) => Json(json!({ "project": updated })).into_response(),
        Ok(None) => ApiError::NotFound("Dự án".to_string()).into_response_with_id(&req_id.0),
        Err(e) => ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    }
}

/// `DELETE /api/v1/projects/:id` - Xóa một dự án.
pub async fn delete_project(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Path(proj_str): Path<String>,
) -> Response {
    let proj_id = match ProjectId::from_str(&proj_str) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID dự án không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    let repo = ProjectRepo::new(&state.db);
    let project = match repo.get_project_by_id(&proj_id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return ApiError::NotFound("Dự án".to_string()).into_response_with_id(&req_id.0);
        }
        Err(e) => return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    };

    if let Err(err) = check_workspace_access(&state, &auth_user, &project.workspace_id, false).await
    {
        return err.into_response_with_id(&req_id.0);
    }

    if let Err(e) = repo.delete_project(&project.workspace_id, &proj_id).await {
        return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0);
    }

    Json(json!({ "status": "deleted" })).into_response()
}

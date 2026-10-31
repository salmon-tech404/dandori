//! Các route xử lý không gian làm việc (Workspace) và thành viên (T11).

use std::str::FromStr;

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use chrono::Utc;
use dandori_db::WorkspaceRepo;
use dandori_types::{UserId, Workspace, WorkspaceId, WorkspaceMember};
use serde::Deserialize;
use serde_json::json;

use crate::error::ApiError;
use crate::middleware::{AuthUser, RequestId};
use crate::state::AppState;

/// Yêu cầu tạo mới không gian làm việc.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateWorkspaceRequest {
    /// Tên không gian làm việc.
    pub name: String,
}

/// Yêu cầu cập nhật không gian làm việc.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateWorkspaceRequest {
    /// Tên mới của không gian làm việc.
    pub name: String,
}

/// Yêu cầu thêm thành viên vào không gian làm việc.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AddMemberRequest {
    /// ID của người dùng cần thêm.
    pub user_id: String,
    /// Vai trò (`admin` hoặc `member`).
    pub role: String,
}

/// Kiểm tra quyền truy cập vào workspace (SEC-BOLA-002).
pub async fn check_workspace_access(
    state: &AppState,
    auth_user: &AuthUser,
    ws_id: &WorkspaceId,
    require_admin: bool,
) -> Result<String, ApiError> {
    let repo = WorkspaceRepo::new(&state.db);
    let member = repo
        .get_member(ws_id, &auth_user.id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if auth_user.is_admin() {
        return Ok("admin".to_string());
    }

    match member {
        Some(m) => {
            if require_admin && m.role != "admin" {
                return Err(ApiError::Forbidden(
                    "Thao tác yêu cầu quyền quản trị viên không gian làm việc (Workspace Admin)"
                        .to_string(),
                ));
            }
            Ok(m.role)
        }
        None => Err(ApiError::Forbidden(
            "Bạn không phải là thành viên của không gian làm việc này".to_string(),
        )),
    }
}

/// `POST /api/v1/workspaces` - Tạo không gian làm việc mới.
pub async fn create_workspace(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Json(payload): Json<CreateWorkspaceRequest>,
) -> Response {
    let name = payload.name.trim();
    if name.is_empty() {
        return ApiError::BadRequest("Tên không gian làm việc không được rỗng".to_string())
            .into_response_with_id(&req_id.0);
    }

    let ws = Workspace {
        id: WorkspaceId::new(),
        name: name.to_string(),
        created_at: Utc::now(),
    };

    let repo = WorkspaceRepo::new(&state.db);
    if let Err(e) = repo.create_workspace(&ws).await {
        return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0);
    }

    // Người tạo tự động trở thành admin của workspace
    let member = WorkspaceMember {
        workspace_id: ws.id,
        user_id: auth_user.id,
        role: "admin".to_string(),
    };
    if let Err(e) = repo.add_member(&member).await {
        return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0);
    }

    Json(json!({ "workspace": ws })).into_response()
}

/// `GET /api/v1/workspaces` - Danh sách không gian làm việc của người dùng.
pub async fn list_workspaces(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
) -> Response {
    let repo = WorkspaceRepo::new(&state.db);
    let workspaces = match repo.list_user_workspaces(&auth_user.id).await {
        Ok(ws) => ws,
        Err(e) => return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    };

    Json(json!({ "workspaces": workspaces })).into_response()
}

/// `GET /api/v1/workspaces/:id` - Lấy thông tin một không gian làm việc.
pub async fn get_workspace(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Path(id_str): Path<String>,
) -> Response {
    let ws_id = match WorkspaceId::from_str(&id_str) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID không gian làm việc không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    if let Err(err) = check_workspace_access(&state, &auth_user, &ws_id, false).await {
        return err.into_response_with_id(&req_id.0);
    }

    let repo = WorkspaceRepo::new(&state.db);
    match repo.get_workspace(&ws_id).await {
        Ok(Some(ws)) => Json(json!({ "workspace": ws })).into_response(),
        Ok(None) => {
            ApiError::NotFound("Không gian làm việc".to_string()).into_response_with_id(&req_id.0)
        }
        Err(e) => ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    }
}

/// `PATCH /api/v1/workspaces/:id` - Cập nhật tên không gian làm việc.
pub async fn update_workspace(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Path(id_str): Path<String>,
    Json(payload): Json<UpdateWorkspaceRequest>,
) -> Response {
    let ws_id = match WorkspaceId::from_str(&id_str) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID không gian làm việc không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    if let Err(err) = check_workspace_access(&state, &auth_user, &ws_id, true).await {
        return err.into_response_with_id(&req_id.0);
    }

    let name = payload.name.trim();
    if name.is_empty() {
        return ApiError::BadRequest("Tên không gian làm việc không được rỗng".to_string())
            .into_response_with_id(&req_id.0);
    }

    let repo = WorkspaceRepo::new(&state.db);
    if let Err(e) = repo.update_workspace_name(&ws_id, name).await {
        return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0);
    }

    match repo.get_workspace(&ws_id).await {
        Ok(Some(ws)) => Json(json!({ "workspace": ws })).into_response(),
        Ok(None) => {
            ApiError::NotFound("Không gian làm việc".to_string()).into_response_with_id(&req_id.0)
        }
        Err(e) => ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    }
}

/// `DELETE /api/v1/workspaces/:id` - Xóa không gian làm việc.
pub async fn delete_workspace(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Path(id_str): Path<String>,
) -> Response {
    let ws_id = match WorkspaceId::from_str(&id_str) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID không gian làm việc không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    if let Err(err) = check_workspace_access(&state, &auth_user, &ws_id, true).await {
        return err.into_response_with_id(&req_id.0);
    }

    let repo = WorkspaceRepo::new(&state.db);
    if let Err(e) = repo.delete_workspace(&ws_id).await {
        return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0);
    }

    Json(json!({ "status": "deleted" })).into_response()
}

/// `GET /api/v1/workspaces/:id/members` - Danh sách thành viên.
pub async fn list_members(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Path(id_str): Path<String>,
) -> Response {
    let ws_id = match WorkspaceId::from_str(&id_str) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID không gian làm việc không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    if let Err(err) = check_workspace_access(&state, &auth_user, &ws_id, false).await {
        return err.into_response_with_id(&req_id.0);
    }

    let repo = WorkspaceRepo::new(&state.db);
    match repo.list_members(&ws_id).await {
        Ok(members) => Json(json!({ "members": members })).into_response(),
        Err(e) => ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    }
}

/// `POST /api/v1/workspaces/:id/members` - Thêm thành viên mới.
pub async fn add_member(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Path(id_str): Path<String>,
    Json(payload): Json<AddMemberRequest>,
) -> Response {
    let ws_id = match WorkspaceId::from_str(&id_str) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID không gian làm việc không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    if let Err(err) = check_workspace_access(&state, &auth_user, &ws_id, true).await {
        return err.into_response_with_id(&req_id.0);
    }

    let target_uid = match UserId::from_str(&payload.user_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID người dùng không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    let role = payload.role.trim();
    if role != "admin" && role != "member" {
        return ApiError::BadRequest("Vai trò phải là admin hoặc member".to_string())
            .into_response_with_id(&req_id.0);
    }

    let member = WorkspaceMember {
        workspace_id: ws_id,
        user_id: target_uid,
        role: role.to_string(),
    };

    let repo = WorkspaceRepo::new(&state.db);
    if let Err(e) = repo.add_member(&member).await {
        return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0);
    }

    Json(json!({ "member": member })).into_response()
}

/// `DELETE /api/v1/workspaces/:id/members/:user_id` - Xóa thành viên.
pub async fn remove_member(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Path((ws_str, user_str)): Path<(String, String)>,
) -> Response {
    let ws_id = match WorkspaceId::from_str(&ws_str) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID không gian làm việc không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    if let Err(err) = check_workspace_access(&state, &auth_user, &ws_id, true).await {
        return err.into_response_with_id(&req_id.0);
    }

    let target_uid = match UserId::from_str(&user_str) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::BadRequest("ID người dùng không hợp lệ".to_string())
                .into_response_with_id(&req_id.0);
        }
    };

    let repo = WorkspaceRepo::new(&state.db);
    if let Err(e) = repo.remove_member(&ws_id, &target_uid).await {
        return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0);
    }

    Json(json!({ "status": "removed" })).into_response()
}

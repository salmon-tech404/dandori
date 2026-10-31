//! Các route quản lý tính năng (Feature Registry) và phân quyền 3 cấp (T11 AC5).

use std::str::FromStr;

use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use dandori_db::WorkspaceRepo;
use dandori_db::repo::FeatureRepo;
use dandori_registry::types::{RegistryRole, RegistryScope};
use dandori_types::WorkspaceId;
use serde::Deserialize;
use serde_json::json;

use crate::error::ApiError;
use crate::middleware::{AuthUser, RequestId};
use crate::state::AppState;

/// Tham số truy vấn trạng thái hiệu lực của module.
#[derive(Debug, Deserialize)]
pub struct EffectiveQuery {
    /// ID module cần truy vấn.
    pub module_id: Option<String>,
    /// ID workspace ngữ cảnh (nếu có).
    pub workspace: Option<String>,
    /// ID project ngữ cảnh (nếu có).
    pub project: Option<String>,
}

/// Yêu cầu lên kế hoạch kích hoạt module.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanEnableRequest {
    /// ID module muốn bật.
    pub module_id: String,
    /// Cấp phạm vi (`system`, `workspace`, hoặc `project`).
    pub scope: String,
    /// ID của phạm vi (ví dụ ID workspace hoặc ID project).
    pub target_id: String,
    /// ID workspace cha (nếu scope là project).
    pub workspace_id: Option<String>,
}

/// Yêu cầu áp dụng thay đổi trạng thái module.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplyFeatureRequest {
    /// ID module.
    pub module_id: String,
    /// Cấp phạm vi (`system`, `workspace`, hoặc `project`).
    pub scope: String,
    /// ID của phạm vi.
    pub target_id: String,
    /// ID workspace cha (nếu scope là project).
    pub workspace_id: Option<String>,
    /// Bật (`true`) hay tắt (`false`).
    pub enable: bool,
    /// Lý do thực hiện (phục vụ audit).
    pub reason: Option<String>,
    /// Xác nhận cảnh báo rủi ro.
    pub risk_acknowledged: Option<bool>,
}

/// Yêu cầu lên kế hoạch vô hiệu hóa module.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanDisableRequest {
    /// ID module muốn tắt.
    pub module_id: String,
    /// Cấp phạm vi.
    pub scope: String,
    /// ID của phạm vi.
    pub target_id: String,
    /// ID workspace cha (nếu scope là project).
    pub workspace_id: Option<String>,
}

fn parse_registry_scope(
    scope_str: &str,
    target_id: &str,
    workspace_id: Option<&str>,
) -> Result<RegistryScope, String> {
    match scope_str {
        "system" => Ok(RegistryScope::System),
        "workspace" => Ok(RegistryScope::Workspace(target_id.to_string())),
        "project" => {
            let ws_id = workspace_id.unwrap_or(target_id).to_string();
            Ok(RegistryScope::Project {
                workspace_id: ws_id,
                project_id: target_id.to_string(),
            })
        }
        other => Err(format!(
            "Phạm vi '{other}' không hợp lệ (phải là system, workspace hoặc project)"
        )),
    }
}

async fn resolve_user_registry_role(
    state: &AppState,
    auth_user: &AuthUser,
    scope: &RegistryScope,
) -> RegistryRole {
    if auth_user.is_admin() {
        return RegistryRole::SystemOwner;
    }

    match scope {
        RegistryScope::System => RegistryRole::Member,
        RegistryScope::Workspace(ws_id) => {
            let Ok(ws_uuid) = WorkspaceId::from_str(ws_id) else {
                return RegistryRole::Member;
            };
            let repo = WorkspaceRepo::new(&state.db);
            match repo.get_member(&ws_uuid, &auth_user.id).await {
                Ok(Some(m)) if m.role == "admin" => RegistryRole::WorkspaceAdmin,
                Ok(Some(_)) => RegistryRole::Member,
                _ => RegistryRole::Member,
            }
        }
        RegistryScope::Project { workspace_id, .. } => {
            let Ok(ws_uuid) = WorkspaceId::from_str(workspace_id) else {
                return RegistryRole::Member;
            };
            let repo = WorkspaceRepo::new(&state.db);
            match repo.get_member(&ws_uuid, &auth_user.id).await {
                Ok(Some(m)) if m.role == "admin" => RegistryRole::WorkspaceAdmin,
                Ok(Some(_)) => RegistryRole::ProjectOwner,
                _ => RegistryRole::Member,
            }
        }
    }
}

/// `GET /api/v1/features/effective` - Truy vấn trạng thái hiệu lực của module.
pub async fn get_effective_state(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    _auth_user: AuthUser,
    Query(query): Query<EffectiveQuery>,
) -> Response {
    let module_id = query
        .module_id
        .unwrap_or_else(|| "sample-hello".to_string());
    let ws_id = query.workspace.as_deref().unwrap_or("default");
    let proj_id = query.project.as_deref().unwrap_or("default");

    let registry = state.lifecycle_manager.registry();
    match registry.effective_state(&module_id, ws_id, proj_id).await {
        Ok(eff) => Json(json!({ "effective": eff })).into_response(),
        Err(e) => ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    }
}

/// `GET /api/v1/features/modules` - Danh sách module trong hệ thống.
pub async fn list_modules(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    _auth_user: AuthUser,
) -> Response {
    let repo = FeatureRepo::new(&state.db);
    match repo.list_modules().await {
        Ok(modules) => Json(json!({ "modules": modules })).into_response(),
        Err(e) => ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    }
}

/// `GET /api/v1/features/packs` - Danh sách gói mẫu tính năng.
pub async fn list_packs(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    _auth_user: AuthUser,
) -> Response {
    let repo = FeatureRepo::new(&state.db);
    match repo.list_packs().await {
        Ok(packs) => Json(json!({ "packs": packs })).into_response(),
        Err(e) => ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    }
}

/// `POST /api/v1/features/plan-enable` - Lên kế hoạch kích hoạt module.
pub async fn plan_enable(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Json(payload): Json<PlanEnableRequest>,
) -> Response {
    let scope = match parse_registry_scope(
        &payload.scope,
        &payload.target_id,
        payload.workspace_id.as_deref(),
    ) {
        Ok(s) => s,
        Err(err) => return ApiError::BadRequest(err).into_response_with_id(&req_id.0),
    };

    let role = resolve_user_registry_role(&state, &auth_user, &scope).await;
    if !role.can_manage_scope(&scope) {
        return ApiError::Forbidden(format!(
            "Vai trò '{}' không có quyền thay đổi cấu hình tại phạm vi '{}'",
            role.as_str(),
            scope.scope_kind()
        ))
        .into_response_with_id(&req_id.0);
    }

    let registry = state.lifecycle_manager.registry();
    match registry.plan_enable(&payload.module_id, &scope).await {
        Ok(plan) => Json(json!({ "plan": plan })).into_response(),
        Err(e) => ApiError::BadRequest(e.to_string()).into_response_with_id(&req_id.0),
    }
}

/// `POST /api/v1/features/apply` - Áp dụng thay đổi trạng thái module (bật/tắt thực tế).
pub async fn apply_feature(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Json(payload): Json<ApplyFeatureRequest>,
) -> Response {
    let scope = match parse_registry_scope(
        &payload.scope,
        &payload.target_id,
        payload.workspace_id.as_deref(),
    ) {
        Ok(s) => s,
        Err(err) => return ApiError::BadRequest(err).into_response_with_id(&req_id.0),
    };

    let role = resolve_user_registry_role(&state, &auth_user, &scope).await;
    if !role.can_manage_scope(&scope) {
        return ApiError::Forbidden(format!(
            "Vai trò '{}' không có quyền thay đổi cấu hình tại phạm vi '{}'",
            role.as_str(),
            scope.scope_kind()
        ))
        .into_response_with_id(&req_id.0);
    }

    let force = payload.risk_acknowledged.unwrap_or(false);
    let user_id_str = auth_user.id.to_string();

    let result = if payload.enable {
        state
            .lifecycle_manager
            .enable(&payload.module_id, &scope, role, &user_id_str, force)
            .await
    } else {
        state
            .lifecycle_manager
            .disable(&payload.module_id, &scope, role, &user_id_str, force)
            .await
    };

    match result {
        Ok(_) => Json(json!({ "status": "applied", "enabled": payload.enable })).into_response(),
        Err(e) => ApiError::BadRequest(e.to_string()).into_response_with_id(&req_id.0),
    }
}

/// `POST /api/v1/features/plan-disable` - Lên kế hoạch vô hiệu hóa module.
pub async fn plan_disable(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Json(payload): Json<PlanDisableRequest>,
) -> Response {
    let scope = match parse_registry_scope(
        &payload.scope,
        &payload.target_id,
        payload.workspace_id.as_deref(),
    ) {
        Ok(s) => s,
        Err(err) => return ApiError::BadRequest(err).into_response_with_id(&req_id.0),
    };

    let role = resolve_user_registry_role(&state, &auth_user, &scope).await;
    if !role.can_manage_scope(&scope) {
        return ApiError::Forbidden(format!(
            "Vai trò '{}' không có quyền thay đổi cấu hình tại phạm vi '{}'",
            role.as_str(),
            scope.scope_kind()
        ))
        .into_response_with_id(&req_id.0);
    }

    let registry = state.lifecycle_manager.registry();
    match registry.plan_disable(&payload.module_id, &scope).await {
        Ok(plan) => Json(json!({ "plan": plan })).into_response(),
        Err(e) => ApiError::BadRequest(e.to_string()).into_response_with_id(&req_id.0),
    }
}

/// `GET /api/v1/features/audit` - Danh mục lịch sử kiểm toán thay đổi cấu hình.
pub async fn get_audit_log(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
) -> Response {
    if !auth_user.is_admin() {
        return ApiError::Forbidden("Chỉ quản trị viên mới có quyền xem audit log".to_string())
            .into_response_with_id(&req_id.0);
    }

    let repo = FeatureRepo::new(&state.db);
    match repo.list_all_audits(50).await {
        Ok(logs) => Json(json!({ "audit": logs })).into_response(),
        Err(e) => ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    }
}

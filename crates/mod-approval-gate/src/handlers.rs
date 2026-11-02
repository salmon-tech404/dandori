//! Các bộ xử lý HTTP request cho module `approval-gate`.

use axum::Json;
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use dandori_module_api::{
    ModuleContext, SessionUser, can_access_workspace, can_administer_workspace, session_user,
};
use dandori_types::{
    Envelope, EventPayload, EventScope, ProjectId, TaskId, TaskStatus, WorkspaceId,
};
use serde_json::json;
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::model::{
    ApprovalPolicy, ApprovalRequest, DecideRequest, ListRequestsQuery, UpdatePolicyRequest,
};

/// Trạng thái chia sẻ của module `approval-gate`.
#[derive(Clone, Debug)]
pub struct ApprovalGateState {
    /// Ngữ cảnh runtime của module.
    pub ctx: ModuleContext,
}

fn err_resp(status: StatusCode, msg: &'static str) -> Response {
    (status, Json(json!({ "error": msg }))).into_response()
}

/// `GET /requests`: Danh sách các yêu cầu phê duyệt.
pub async fn list_requests(
    State(state): State<ApprovalGateState>,
    headers: HeaderMap,
    Query(query): Query<ListRequestsQuery>,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có cơ sở dữ liệu"),
    };
    let user = match session_user(pool, &headers).await {
        Some(u) => u,
        None => return err_resp(StatusCode::UNAUTHORIZED, "Chưa xác thực"),
    };

    let ws_id = query.workspace_id.as_deref().unwrap_or_default();
    if !ws_id.is_empty() && !can_access_workspace(pool, &user, ws_id).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
    }

    let mut sql = "SELECT id, workspace_id, project_id, task_id, kind, from_status, target_status, \
                   requester_id, status, reason, decider_id, created_at, decided_at, expires_at, rejection_reason \
                   FROM m_approval_gate_request WHERE 1=1".to_string();

    if let Some(ref w) = query.workspace_id {
        sql.push_str(&format!(" AND workspace_id = '{w}'"));
    }
    if let Some(ref p) = query.project_id {
        sql.push_str(&format!(" AND project_id = '{p}'"));
    }
    if let Some(ref t) = query.task_id {
        sql.push_str(&format!(" AND task_id = '{t}'"));
    }
    if let Some(ref s) = query.status {
        sql.push_str(&format!(" AND status = '{s}'"));
    }
    sql.push_str(" ORDER BY created_at DESC LIMIT 100");

    let requests: Vec<ApprovalRequest> = match sqlx::query_as(&sql).fetch_all(pool).await {
        Ok(rows) => rows,
        Err(_) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Lỗi truy vấn dữ liệu"),
    };

    let mut allowed = Vec::new();
    for req in requests {
        if can_access_workspace(pool, &user, &req.workspace_id).await {
            allowed.push(req);
        }
    }

    Json(json!({ "requests": allowed })).into_response()
}

/// `GET /requests/:id`: Chi tiết một yêu cầu phê duyệt.
pub async fn get_request(
    State(state): State<ApprovalGateState>,
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có cơ sở dữ liệu"),
    };
    let user = match session_user(pool, &headers).await {
        Some(u) => u,
        None => return err_resp(StatusCode::UNAUTHORIZED, "Chưa xác thực"),
    };

    let req: Option<ApprovalRequest> = match sqlx::query_as(
        "SELECT id, workspace_id, project_id, task_id, kind, from_status, target_status, \
         requester_id, status, reason, decider_id, created_at, decided_at, expires_at, rejection_reason \
         FROM m_approval_gate_request WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(pool)
    .await
    {
        Ok(r) => r,
        Err(_) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Lỗi truy vấn dữ liệu"),
    };

    let req = match req {
        Some(r) => r,
        None => return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy yêu cầu"),
    };

    if !can_access_workspace(pool, &user, &req.workspace_id).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập");
    }

    Json(json!({ "request": req })).into_response()
}

async fn fetch_policy(pool: &SqlitePool, project_id: &str) -> ApprovalPolicy {
    let row: Option<(bool, bool, i64, String)> = sqlx::query_as(
        "SELECT require_in_review_to_merge, disallow_self_approval, expiry_hours, workspace_id \
         FROM m_approval_gate_policy WHERE project_id = ?",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    match row {
        Some((require, disallow, expiry, ws_id)) => ApprovalPolicy {
            workspace_id: ws_id,
            project_id: project_id.to_string(),
            require_in_review_to_merge: require,
            disallow_self_approval: disallow,
            expiry_hours: expiry,
        },
        None => ApprovalPolicy::default(),
    }
}

async fn validate_decision_request(
    pool: &SqlitePool,
    headers: &HeaderMap,
    id: &str,
) -> Result<(SessionUser, ApprovalRequest), (StatusCode, &'static str)> {
    let user = session_user(pool, headers)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Chưa xác thực"))?;

    let req: Option<ApprovalRequest> = sqlx::query_as(
        "SELECT id, workspace_id, project_id, task_id, kind, from_status, target_status, \
         requester_id, status, reason, decider_id, created_at, decided_at, expires_at, rejection_reason \
         FROM m_approval_gate_request WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi truy vấn dữ liệu"))?;

    let req = req.ok_or((StatusCode::NOT_FOUND, "Không tìm thấy yêu cầu"))?;

    if !can_access_workspace(pool, &user, &req.workspace_id).await {
        return Err((StatusCode::FORBIDDEN, "Không có quyền truy cập"));
    }
    // Duyệt là đặc quyền của chủ dự án/quản trị (đặc tả P2-03), không phải của mọi thành viên.
    if !can_administer_workspace(pool, &user, &req.workspace_id).await {
        return Err((
            StatusCode::FORBIDDEN,
            "Chỉ quản trị workspace được duyệt hoặc từ chối",
        ));
    }

    let policy = fetch_policy(pool, &req.project_id).await;
    if policy.disallow_self_approval && req.requester_id == user.user_id {
        return Err((
            StatusCode::BAD_REQUEST,
            "Người yêu cầu không được tự phê duyệt",
        ));
    }

    Ok((user, req))
}

fn publish_decision_events(
    ctx: &ModuleContext,
    req: &ApprovalRequest,
    new_status: &str,
    approved: bool,
    decided_by: &str,
) {
    let ws_id = WorkspaceId::from_str(&req.workspace_id)
        .unwrap_or_else(|_| WorkspaceId::from(uuid::Uuid::nil()));
    let prj_id =
        ProjectId::from_str(&req.project_id).unwrap_or_else(|_| ProjectId::from(uuid::Uuid::nil()));
    let t_id = TaskId::from_str(&req.task_id).unwrap_or_else(|_| TaskId::from(uuid::Uuid::nil()));

    ctx.event_bus().publish(Envelope::new(
        Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::TaskStatusChanged {
            task_id: t_id,
            old_status: TaskStatus::NeedsHuman.as_str().to_string(),
            new_status: new_status.to_string(),
        },
    ));

    ctx.event_bus().publish(Envelope::new(
        Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::ApprovalDecided {
            gate_id: "approval-gate".to_string(),
            task_id: t_id,
            approved,
            decided_by: decided_by.to_string(),
        },
    ));
}

/// `POST /requests/:id/approve`: Chấp thuận yêu cầu phê duyệt.
pub async fn approve_request(
    State(state): State<ApprovalGateState>,
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
    Json(payload): Json<DecideRequest>,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có cơ sở dữ liệu"),
    };
    let (user, req) = match validate_decision_request(pool, &headers, &id).await {
        Ok(pair) => pair,
        Err((code, msg)) => return err_resp(code, msg),
    };

    let now_str = Utc::now().to_rfc3339();
    let res = match sqlx::query(
        "UPDATE m_approval_gate_request \
         SET status = 'approved', decider_id = ?, decided_at = ?, reason = coalesce(?, reason) \
         WHERE id = ? AND status = 'pending' AND (expires_at IS NULL OR expires_at > ?)",
    )
    .bind(&user.user_id)
    .bind(&now_str)
    .bind(&payload.reason)
    .bind(&id)
    .bind(&now_str)
    .execute(pool)
    .await
    {
        Ok(r) => r,
        Err(_) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Lỗi cập nhật yêu cầu"),
    };

    if res.rows_affected() == 0 {
        return err_resp(
            StatusCode::CONFLICT,
            "Yêu cầu đã được quyết định hoặc đã hết hạn",
        );
    }

    let _ = sqlx::query("UPDATE task SET status = ?, updated_at = ? WHERE id = ?")
        .bind(&req.target_status)
        .bind(&now_str)
        .bind(&req.task_id)
        .execute(pool)
        .await;

    publish_decision_events(&state.ctx, &req, &req.target_status, true, &user.user_id);

    Json(json!({ "status": "approved", "id": id })).into_response()
}

/// `POST /requests/:id/reject`: Từ chối yêu cầu phê duyệt.
pub async fn reject_request(
    State(state): State<ApprovalGateState>,
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
    Json(payload): Json<DecideRequest>,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có cơ sở dữ liệu"),
    };
    let (user, req) = match validate_decision_request(pool, &headers, &id).await {
        Ok(pair) => pair,
        Err((code, msg)) => return err_resp(code, msg),
    };

    let now_str = Utc::now().to_rfc3339();
    let res = match sqlx::query(
        "UPDATE m_approval_gate_request \
         SET status = 'rejected', decider_id = ?, decided_at = ?, rejection_reason = ? \
         WHERE id = ? AND status = 'pending' AND (expires_at IS NULL OR expires_at > ?)",
    )
    .bind(&user.user_id)
    .bind(&now_str)
    .bind(&payload.reason)
    .bind(&id)
    .bind(&now_str)
    .execute(pool)
    .await
    {
        Ok(r) => r,
        Err(_) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Lỗi cập nhật yêu cầu"),
    };

    if res.rows_affected() == 0 {
        return err_resp(
            StatusCode::CONFLICT,
            "Yêu cầu đã được quyết định hoặc đã hết hạn",
        );
    }

    let _ = sqlx::query("UPDATE task SET status = ?, updated_at = ? WHERE id = ?")
        .bind(&req.from_status)
        .bind(&now_str)
        .bind(&req.task_id)
        .execute(pool)
        .await;

    publish_decision_events(&state.ctx, &req, &req.from_status, false, &user.user_id);

    Json(json!({ "status": "rejected", "id": id })).into_response()
}

async fn resolve_project_workspace(
    pool: &SqlitePool,
    headers: &HeaderMap,
    project_id: &str,
) -> Result<(SessionUser, String), (StatusCode, &'static str)> {
    let user = session_user(pool, headers)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Chưa xác thực"))?;

    let ws_row: Option<(String,)> = sqlx::query_as("SELECT workspace_id FROM project WHERE id = ?")
        .bind(project_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi truy vấn dự án"))?;

    let ws_id = ws_row
        .map(|(w,)| w)
        .ok_or((StatusCode::NOT_FOUND, "Không tìm thấy dự án"))?;

    if !can_access_workspace(pool, &user, &ws_id).await {
        return Err((StatusCode::FORBIDDEN, "Không có quyền truy cập"));
    }

    Ok((user, ws_id))
}

/// `GET /policy/:project_id`: Lấy chính sách phê duyệt của dự án.
pub async fn get_project_policy(
    State(state): State<ApprovalGateState>,
    AxumPath(project_id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có cơ sở dữ liệu"),
    };
    if let Err((code, msg)) = resolve_project_workspace(pool, &headers, &project_id).await {
        return err_resp(code, msg);
    }

    let policy = fetch_policy(pool, &project_id).await;
    Json(json!({ "policy": policy })).into_response()
}

/// `PUT /policy/:project_id`: Cập nhật chính sách phê duyệt của dự án.
pub async fn update_project_policy(
    State(state): State<ApprovalGateState>,
    AxumPath(project_id): AxumPath<String>,
    headers: HeaderMap,
    Json(payload): Json<UpdatePolicyRequest>,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có cơ sở dữ liệu"),
    };
    let (user, ws_id) = match resolve_project_workspace(pool, &headers, &project_id).await {
        Ok(res) => res,
        Err((code, msg)) => return err_resp(code, msg),
    };
    // Đổi chính sách duyệt (tắt yêu cầu duyệt, cho tự duyệt) là quyền quản trị: nếu không, mọi thành viên
    // tự gỡ được cổng duyệt.
    if !can_administer_workspace(pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Chỉ quản trị workspace được đổi chính sách duyệt",
        );
    }

    let current = fetch_policy(pool, &project_id).await;
    let require = payload
        .require_in_review_to_merge
        .unwrap_or(current.require_in_review_to_merge);
    let disallow = payload
        .disallow_self_approval
        .unwrap_or(current.disallow_self_approval);
    let expiry = payload.expiry_hours.unwrap_or(current.expiry_hours);

    let res = sqlx::query(
        "INSERT INTO m_approval_gate_policy (workspace_id, project_id, require_in_review_to_merge, disallow_self_approval, expiry_hours) \
         VALUES (?, ?, ?, ?, ?) \
         ON CONFLICT (workspace_id, project_id) DO UPDATE SET \
         require_in_review_to_merge = excluded.require_in_review_to_merge, \
         disallow_self_approval = excluded.disallow_self_approval, \
         expiry_hours = excluded.expiry_hours",
    )
    .bind(&ws_id)
    .bind(&project_id)
    .bind(require)
    .bind(disallow)
    .bind(expiry)
    .execute(pool)
    .await;

    if res.is_err() {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Lỗi lưu chính sách");
    }

    let updated = ApprovalPolicy {
        workspace_id: ws_id,
        project_id,
        require_in_review_to_merge: require,
        disallow_self_approval: disallow,
        expiry_hours: expiry,
    };

    Json(json!({ "policy": updated })).into_response()
}

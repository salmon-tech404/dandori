//! Định nghĩa các route HTTP cho module `approval-gate`.

use axum::Router;
use axum::routing::{get, post};

use crate::handlers::{
    ApprovalGateState, approve_request, get_project_policy, get_request, list_requests,
    reject_request, update_project_policy,
};

/// Khởi tạo router cho module `approval-gate`.
pub fn create_routes(state: ApprovalGateState) -> Router {
    Router::new()
        .route("/requests", get(list_requests))
        .route("/requests/{id}", get(get_request))
        .route("/requests/{id}/approve", post(approve_request))
        .route("/requests/{id}/reject", post(reject_request))
        .route(
            "/policy/{project_id}",
            get(get_project_policy).put(update_project_policy),
        )
        .with_state(state)
}

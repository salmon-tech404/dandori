//! Định tuyến các endpoint cho module `integrations` (P2-06).

use axum::Router;
use axum::routing::{delete, get, post};

use crate::handlers::{
    IntegrationsState, create_connection, delete_connection, import_linear_issues,
    list_connections, list_linear_issues, test_connection_handler, test_preflight_connection,
};

/// Tạo Axum Router cho module `integrations`.
pub fn create_routes(state: IntegrationsState) -> Router {
    Router::new()
        .route(
            "/workspaces/{ws_id}/connections",
            get(list_connections).post(create_connection),
        )
        .route(
            "/workspaces/{ws_id}/connections/test",
            post(test_preflight_connection),
        )
        .route("/connections/{conn_id}/test", post(test_connection_handler))
        .route("/connections/{conn_id}", delete(delete_connection))
        .route(
            "/connections/{conn_id}/linear/issues",
            get(list_linear_issues),
        )
        .route(
            "/connections/{conn_id}/linear/import",
            post(import_linear_issues),
        )
        .with_state(state)
}

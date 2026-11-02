//! Định nghĩa các route HTTP cho module `diff-preview`.

use axum::Router;
use axum::routing::{get, post};

use crate::handlers::{
    DiffPreviewState, get_diff, get_preview_status, start_preview, stop_preview,
};

/// Khởi tạo router cho module `diff-preview`.
pub fn create_routes(state: DiffPreviewState) -> Router {
    Router::new()
        .route("/tasks/{id}/diff", get(get_diff))
        .route("/tasks/{id}/preview/start", post(start_preview))
        .route("/tasks/{id}/preview/status", get(get_preview_status))
        .route("/tasks/{id}/preview/stop", post(stop_preview))
        .with_state(state)
}

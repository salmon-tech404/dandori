//! Định nghĩa các route HTTP cho module `test-agent`.

use axum::Router;
use axum::routing::get;

use crate::handlers::{TestAgentState, get_config, get_result, list_results, update_config};

/// Khởi tạo router cho module `test-agent`.
pub fn create_routes(state: TestAgentState) -> Router {
    Router::new()
        .route(
            "/projects/{project_id}/config",
            get(get_config).put(update_config),
        )
        .route("/tasks/{task_id}/results", get(list_results))
        .route("/runs/{run_id}/result", get(get_result))
        .with_state(state)
}

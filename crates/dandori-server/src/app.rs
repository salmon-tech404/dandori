//! Cấu hình và lắp ráp Router chính cho ứng dụng Dandori Server.

use axum::Router;
use axum::routing::{delete, get, post};

use crate::middleware::{check_origin, set_request_id};
use crate::routes::{auth, features, health, projects, tasks, workspaces};
use crate::state::AppState;

fn make_workspace_router() -> Router<AppState> {
    Router::new()
        .route(
            "/workspaces",
            get(workspaces::list_workspaces).post(workspaces::create_workspace),
        )
        .route(
            "/workspaces/{id}",
            get(workspaces::get_workspace)
                .patch(workspaces::update_workspace)
                .delete(workspaces::delete_workspace),
        )
        .route(
            "/workspaces/{id}/members",
            get(workspaces::list_members).post(workspaces::add_member),
        )
        .route(
            "/workspaces/{ws_id}/members/{user_id}",
            delete(workspaces::remove_member),
        )
        .route(
            "/workspaces/{id}/projects",
            get(projects::list_projects).post(projects::create_project),
        )
}

fn make_entity_router() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{id}",
            get(projects::get_project)
                .patch(projects::update_project)
                .delete(projects::delete_project),
        )
        .route(
            "/projects/{id}/tasks",
            get(tasks::list_tasks).post(tasks::create_task),
        )
        .route(
            "/tasks/{id}",
            get(tasks::get_task)
                .patch(tasks::update_task)
                .delete(tasks::delete_task),
        )
        .route("/tasks/{id}/status", post(tasks::update_task_status))
}

fn make_features_router() -> Router<AppState> {
    Router::new()
        .route("/features/effective", get(features::get_effective_state))
        .route("/features/modules", get(features::list_modules))
        .route("/features/packs", get(features::list_packs))
        .route("/features/plan-enable", post(features::plan_enable))
        .route("/features/apply", post(features::apply_feature))
        .route("/features/plan-disable", post(features::plan_disable))
        .route("/features/audit", get(features::get_audit_log))
}

/// Xây dựng Axum Router hoàn chỉnh với đầy đủ route lõi, module routes và middleware.
pub fn create_app(state: AppState) -> Router {
    let module_router = state.lifecycle_manager.create_router();

    let api_router = Router::new()
        .route("/health", get(health::health_check))
        .route("/auth/setup-owner", post(auth::setup_owner))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/users", post(auth::create_user))
        .route("/me", get(auth::get_current_user))
        .merge(make_workspace_router())
        .merge(make_entity_router())
        .merge(make_features_router())
        .with_state(state);

    Router::new()
        .nest("/api/v1/m", module_router)
        .nest("/api/v1", api_router)
        .layer(axum::middleware::from_fn(check_origin))
        .layer(axum::middleware::from_fn(set_request_id))
}

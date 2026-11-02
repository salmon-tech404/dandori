//! Khai báo router cho module `run-history`.

use axum::Router;
use axum::routing::get;
use dandori_module_api::ModuleContext;

use crate::handlers::{export_run, get_note, get_run, get_run_events, list_runs, update_note};

/// Khởi tạo Router cho module `run-history`.
pub fn create_router(ctx: ModuleContext) -> Router {
    Router::new()
        .route("/runs", get(list_runs))
        .route("/runs/{id}", get(get_run))
        .route("/runs/{id}/events", get(get_run_events))
        .route("/runs/{id}/note", get(get_note).put(update_note))
        .route("/runs/{id}/export", get(export_run))
        .with_state(ctx)
}

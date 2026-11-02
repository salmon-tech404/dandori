//! Module `live-status`: Theo dõi hoạt động agent và luồng log SSE theo thời gian thực (P1-06).

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use dandori_module_api::{
    BoxFuture, Manifest, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup, ModuleRisk,
    ModuleScope, ModuleSubscription, UiContribution, UiDeclaration,
};
use dandori_types::Envelope;

pub mod filter;
pub mod handlers;
pub mod state;

pub use filter::{allowed_event_kinds, is_allowed_kind};
pub use state::LiveStatusState;

fn get_manifest() -> Manifest {
    Manifest {
        id: "live-status".to_string(),
        group: ModuleGroup::Dashboard,
        name: "Xem thời gian thực".to_string(),
        description: "Xem agent đang làm gì theo thời gian thực qua luồng SSE".to_string(),
        version: "0.1.0".to_string(),
        requires: vec![],
        scopes: vec![ModuleScope::Workspace, ModuleScope::Project],
        effort: ModuleEffort::Medium,
        risk: ModuleRisk::Low,
        risk_note: None,
        core: false,
        requires_worker_restart: false,
        settings: std::collections::HashMap::new(),
        ui: UiDeclaration {
            contributes: vec![
                UiContribution {
                    id: "live-status-dashboard-panel".to_string(),
                    slot: "dashboard.panel".to_string(),
                    label: "Agent đang làm việc".to_string(),
                    order: 10,
                },
                UiContribution {
                    id: "live-status-task-tab".to_string(),
                    slot: "board.task-panel.tab".to_string(),
                    label: "Trực tiếp".to_string(),
                    order: 20,
                },
            ],
        },
    }
}

/// Module hiện thực tính năng xem trực tiếp qua SSE.
#[derive(Debug, Clone)]
pub struct LiveStatusModule {
    state: LiveStatusState,
}

impl Default for LiveStatusModule {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveStatusModule {
    /// Khởi tạo instance mới của `LiveStatusModule`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: LiveStatusState::new(),
        }
    }

    /// Trạng thái quản lý của module.
    #[must_use]
    pub fn state(&self) -> &LiveStatusState {
        &self.state
    }
}

impl Module for LiveStatusModule {
    fn manifest(&self) -> &Manifest {
        static MANIFEST: std::sync::OnceLock<Manifest> = std::sync::OnceLock::new();
        MANIFEST.get_or_init(get_manifest)
    }

    fn migrations(&self) -> &'static [Migration] {
        &[]
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        let app_state = handlers::AppState {
            module_state: self.state.clone(),
            event_bus: ctx.event_bus().clone(),
            pool: ctx.pool().cloned(),
        };

        Router::new()
            .route("/stream", get(handlers::stream_handler))
            .route("/status", get(handlers::status_handler))
            .with_state(app_state)
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<dandori_module_api::WorkerSpec> {
        vec![]
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        let recent_events = self.state.recent_events().clone();
        vec![ModuleSubscription {
            name: "live_status_recorder",
            filter: dandori_module_api::EventFilter::all(),
            handler: Arc::new(move |envelope: Envelope| {
                let buffer = recent_events.clone();
                Box::pin(async move {
                    if is_allowed_kind(&envelope.kind) {
                        buffer.record(envelope);
                    }
                }) as BoxFuture<'static, ()>
            }),
        }]
    }
}

//! Module `tech-profile`: Hồ sơ kỹ thuật dự án mà mọi agent đều đọc (P1-08).

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, put};
use dandori_module_api::{
    Manifest, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup, ModuleRisk, ModuleScope,
    ModuleSubscription, PromptContributor, UiContribution, UiDeclaration, WorkerSpec,
};

pub mod contributor;
pub mod handlers;
pub mod validation;

pub use contributor::TechProfileContributor;

fn get_manifest() -> Manifest {
    Manifest {
        id: "tech-profile".to_string(),
        group: ModuleGroup::Knowledge,
        name: "Hồ sơ kỹ thuật".to_string(),
        description: "Hồ sơ kỹ thuật dự án mà mọi agent đều đọc khi chạy".to_string(),
        version: "0.1.0".to_string(),
        requires: vec!["lead-worker".to_string()],
        scopes: vec![ModuleScope::Workspace, ModuleScope::Project],
        effort: ModuleEffort::Medium,
        risk: ModuleRisk::Low,
        risk_note: None,
        core: false,
        requires_worker_restart: false,
        settings: std::collections::HashMap::new(),
        ui: UiDeclaration {
            contributes: vec![UiContribution {
                id: "tech-profile-tab".to_string(),
                slot: "knowledge.tab".to_string(),
                label: "Hồ sơ kỹ thuật".to_string(),
                order: 20,
            }],
        },
    }
}

const MIGRATIONS: &[Migration] = &[Migration {
    id: "0001_tech_profile",
    sql: include_str!("../migrations/0001_tech_profile.sql"),
}];

/// Cấu trúc đại diện cho module `tech-profile`.
#[derive(Debug, Clone, Default)]
pub struct TechProfileModule;

impl TechProfileModule {
    /// Khởi tạo module TechProfileModule mới.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for TechProfileModule {
    fn manifest(&self) -> &Manifest {
        static MANIFEST: std::sync::OnceLock<Manifest> = std::sync::OnceLock::new();
        MANIFEST.get_or_init(get_manifest)
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        let app_state = handlers::AppState {
            pool: ctx.pool().cloned(),
        };

        Router::new()
            .route(
                "/projects/{project_id}/entries",
                get(handlers::list_entries_handler).post(handlers::create_entry_handler),
            )
            .route(
                "/entries/{id}",
                put(handlers::update_entry_handler).delete(handlers::delete_entry_handler),
            )
            .route(
                "/projects/{project_id}/preview",
                get(handlers::preview_profile_handler),
            )
            .with_state(app_state)
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        vec![]
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        vec![]
    }

    fn prompt_contributor(&self, ctx: &ModuleContext) -> Option<Arc<dyn PromptContributor>> {
        Some(Arc::new(TechProfileContributor::new(ctx.pool().cloned())))
    }
}

//! Module `prompt-library`: Thư viện prompt gắn theo vai trò agent và kỹ năng (P1-07).

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use dandori_module_api::{
    Manifest, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup, ModuleRisk, ModuleScope,
    ModuleSubscription, PromptContributor, UiContribution, UiDeclaration, WorkerSpec,
};

pub mod contributor;
pub mod handlers;
pub mod template;

pub use contributor::PromptLibraryContributor;

fn get_manifest() -> Manifest {
    Manifest {
        id: "prompt-library".to_string(),
        group: ModuleGroup::Knowledge,
        name: "Thư viện prompt".to_string(),
        description: "Thư viện prompt gắn theo vai trò agent và kỹ năng".to_string(),
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
                id: "prompt-library-tab".to_string(),
                slot: "knowledge.tab".to_string(),
                label: "Prompt".to_string(),
                order: 10,
            }],
        },
    }
}

const MIGRATIONS: &[Migration] = &[Migration {
    id: "0001_prompt_library",
    sql: include_str!("../migrations/0001_prompt_library.sql"),
}];

/// Cấu trúc đại diện cho module `prompt-library`.
#[derive(Debug, Clone, Default)]
pub struct PromptLibraryModule;

impl PromptLibraryModule {
    /// Khởi tạo module PromptLibraryModule mới.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for PromptLibraryModule {
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
                "/prompts",
                get(handlers::list_prompts_handler).post(handlers::create_prompt_handler),
            )
            .route(
                "/prompts/{id}",
                get(handlers::get_prompt_handler).put(handlers::update_prompt_handler),
            )
            .route(
                "/prompts/{id}/versions",
                get(handlers::list_versions_handler),
            )
            .route(
                "/prompts/{id}/restore/{version}",
                post(handlers::restore_version_handler),
            )
            .route("/prompts/test-render", post(handlers::test_render_handler))
            .with_state(app_state)
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        vec![]
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        vec![]
    }

    fn prompt_contributor(&self, ctx: &ModuleContext) -> Option<Arc<dyn PromptContributor>> {
        Some(Arc::new(PromptLibraryContributor::new(ctx.pool().cloned())))
    }
}

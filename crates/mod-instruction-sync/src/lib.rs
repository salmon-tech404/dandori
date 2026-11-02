//! Crate `mod-instruction-sync`: Module đồng bộ tệp hướng dẫn CLI (P3-09).

pub mod decorator;
pub mod generator;
pub mod model;
pub mod routes;

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use axum::Router;
use dandori_module_api::{
    Manifest, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup, ModuleRisk, ModuleScope,
    ModuleSubscription, UiContribution, UiDeclaration, WorkdirDecorator, WorkerSpec,
};

pub use decorator::{ALLOWED_FILENAMES, InstructionSyncDecorator};
pub use model::{InstructionSyncConfig, PreviewPayload, PreviewResult, UpdateConfigPayload};
pub use routes::{InstructionSyncState, instruction_sync_router};

fn default_ui() -> UiDeclaration {
    UiDeclaration {
        contributes: vec![UiContribution {
            id: "instruction-sync-tab".to_string(),
            slot: "knowledge.tab".to_string(),
            label: "Đồng bộ hướng dẫn".to_string(),
            order: 40,
        }],
    }
}

fn get_manifest() -> Manifest {
    Manifest {
        id: "instruction-sync".to_string(),
        group: ModuleGroup::Knowledge,
        name: "Đồng bộ tệp hướng dẫn CLI".to_string(),
        description: "Đồng bộ tệp hướng dẫn AGENTS.md, CLAUDE.md từ hồ sơ kỹ thuật và kỹ năng"
            .to_string(),
        version: "0.1.0".to_string(),
        requires: vec!["tech-profile".to_string(), "skills-md".to_string()],
        scopes: vec![ModuleScope::Workspace, ModuleScope::Project],
        effort: ModuleEffort::Medium,
        risk: ModuleRisk::Medium,
        risk_note: Some("Ghi tệp vào worktree; tự động loại trừ git info/exclude".to_string()),
        core: false,
        requires_worker_restart: false,
        settings: HashMap::new(),
        ui: default_ui(),
        allowed_workdir_files: vec![
            "AGENTS.md".to_string(),
            "CLAUDE.md".to_string(),
            ".cursorrules".to_string(),
        ],
    }
}

const MIGRATIONS: &[Migration] = &[Migration {
    id: "0001_instruction_sync_schema",
    sql: include_str!("../migrations/0001_instruction_sync_schema.sql"),
}];

#[derive(Debug, Default)]
pub struct InstructionSyncModule;

impl InstructionSyncModule {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for InstructionSyncModule {
    fn manifest(&self) -> &Manifest {
        static MANIFEST: OnceLock<Manifest> = OnceLock::new();
        MANIFEST.get_or_init(get_manifest)
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        instruction_sync_router(InstructionSyncState::new(ctx.clone()))
    }

    fn workdir_decorator(&self, ctx: &ModuleContext) -> Option<Arc<dyn WorkdirDecorator>> {
        let pool = ctx.pool()?.clone();
        Some(Arc::new(InstructionSyncDecorator::new(pool)))
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        Vec::new()
    }
}

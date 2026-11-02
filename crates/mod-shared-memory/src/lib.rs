//! Crate `mod-shared-memory`: Module bộ nhớ dùng chung chống nhiễm độc bộ nhớ và lọc private key (P3-10).

pub mod contributor;
pub mod model;
pub mod parser;
pub mod redaction;
pub mod routes;

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use axum::Router;
use dandori_module_api::{
    Manifest, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup, ModuleRisk, ModuleScope,
    ModuleSubscription, PromptContributor, SettingSchema, UiContribution, UiDeclaration,
    WorkerSpec,
};

pub use contributor::{
    SHARED_MEMORY_BEGIN_MARKER, SHARED_MEMORY_END_MARKER, SharedMemoryContributor,
};
pub use model::{
    BatchUpdateStatusPayload, CreateMemoryPayload, MemoryEntry, ProposeMemoryPayload,
    UpdateMemoryStatusPayload, is_valid_category, is_valid_status,
};
pub use parser::{ParsedMemoryBlock, parse_agent_memories};
pub use redaction::{contains_secrets, redact_secrets};
pub use routes::{ListMemoryParams, SharedMemoryState, shared_memory_router};

fn default_settings() -> HashMap<String, SettingSchema> {
    let mut s = HashMap::new();
    s.insert(
        "max_items_in_prompt".to_string(),
        SettingSchema {
            type_name: "number".to_string(),
            default: serde_json::json!(10),
            enum_values: None,
            min: Some(1.0),
            max: Some(50.0),
        },
    );
    s.insert(
        "auto_approve_agent_memories".to_string(),
        SettingSchema {
            type_name: "boolean".to_string(),
            default: serde_json::json!(false),
            enum_values: None,
            min: None,
            max: None,
        },
    );
    s
}

fn default_ui() -> UiDeclaration {
    UiDeclaration {
        contributes: vec![UiContribution {
            id: "shared-memory-tab".to_string(),
            slot: "knowledge.tab".to_string(),
            label: "Bộ nhớ chung".to_string(),
            order: 50,
        }],
    }
}

fn get_manifest() -> Manifest {
    Manifest {
        id: "shared-memory".to_string(),
        group: ModuleGroup::Knowledge,
        name: "Bộ nhớ dùng chung".to_string(),
        description:
            "Bộ nhớ chia sẻ giữa các agent, chống nhiễm độc bộ nhớ và lọc khóa riêng đa dòng (B-10)"
                .to_string(),
        version: "0.1.0".to_string(),
        requires: vec!["prompt-library".to_string()],
        scopes: vec![ModuleScope::Workspace, ModuleScope::Project],
        effort: ModuleEffort::Medium,
        risk: ModuleRisk::Medium,
        risk_note: Some(
            "Bộ nhớ dùng chung: Mục agent luôn pending; lọc private key đa dòng (B-10)".to_string(),
        ),
        core: false,
        requires_worker_restart: false,
        settings: default_settings(),
        ui: default_ui(),
        allowed_workdir_files: Vec::new(),
    }
}

const MIGRATIONS: &[Migration] = &[Migration {
    id: "0001_shared_memory_schema",
    sql: include_str!("../migrations/0001_shared_memory_schema.sql"),
}];

#[derive(Debug, Default)]
pub struct SharedMemoryModule;

impl SharedMemoryModule {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for SharedMemoryModule {
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
        shared_memory_router(SharedMemoryState::new(ctx.clone()))
    }

    fn prompt_contributor(&self, ctx: &ModuleContext) -> Option<Arc<dyn PromptContributor>> {
        Some(Arc::new(SharedMemoryContributor::new(ctx.pool().cloned())))
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        Vec::new()
    }
}

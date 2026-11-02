//! Crate `mod-skills-md`: Module quản lý kỹ năng Agent theo chuẩn `SKILL.md` (P3-08).

pub mod archive;
pub mod contributor;
pub mod db;
pub mod frontmatter;
pub mod model;
pub mod routes;

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use axum::Router;
use dandori_module_api::{
    Manifest, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup, ModuleRisk, ModuleScope,
    ModuleSubscription, PromptContributor, UiContribution, UiDeclaration, WorkerSpec,
};

pub use contributor::SkillsMdContributor;
pub use model::{
    CreateBindingPayload, ImportDirPayload, ImportZipPayload, ParsedSkill, SkillBinding,
    SkillDetail, SkillStatus, SkillSummary,
};
pub use routes::{SkillsMdState, skills_md_router};

fn default_ui() -> UiDeclaration {
    UiDeclaration {
        contributes: vec![UiContribution {
            id: "skills-md-knowledge-tab".to_string(),
            slot: "knowledge.tab".to_string(),
            label: "Kỹ năng (SKILL.md)".to_string(),
            order: 30,
        }],
    }
}

fn get_manifest() -> Manifest {
    Manifest {
        id: "skills-md".to_string(),
        group: ModuleGroup::Knowledge,
        name: "Kỹ năng Agent (SKILL.md)".to_string(),
        description:
            "Quản lý kỹ năng theo định dạng SKILL.md, nhập an toàn và đóng góp prompt tiết kiệm"
                .to_string(),
        version: "0.1.0".to_string(),
        requires: vec!["prompt-library".to_string()],
        scopes: vec![ModuleScope::Workspace, ModuleScope::Project],
        effort: ModuleEffort::Medium,
        risk: ModuleRisk::Low,
        risk_note: None,
        core: false,
        requires_worker_restart: false,
        settings: HashMap::new(),
        ui: default_ui(),
        allowed_workdir_files: Vec::new(),
    }
}

const MIGRATIONS: &[Migration] = &[Migration {
    id: "0001_skills_md_schema",
    sql: include_str!("../migrations/0001_skills_md_schema.sql"),
}];

/// Module quản lý kỹ năng Agent theo định dạng SKILL.md.
#[derive(Debug, Default)]
pub struct SkillsMdModule;

impl SkillsMdModule {
    /// Khởi tạo instance mới của `SkillsMdModule`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for SkillsMdModule {
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
        skills_md_router(SkillsMdState::new(ctx.clone()))
    }

    fn prompt_contributor(&self, ctx: &ModuleContext) -> Option<Arc<dyn PromptContributor>> {
        Some(Arc::new(SkillsMdContributor::new(ctx.pool().cloned())))
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        Vec::new()
    }
}

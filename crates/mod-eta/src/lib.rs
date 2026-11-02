//! Crate `mod-eta`: Module dự tính thời gian hoàn thành (ETA) dựa trên thống kê thực tế và tự đo sai số (P3-07).

pub mod calculator;
pub mod db;
pub mod model;
pub mod routes;

use std::collections::HashMap;
use std::sync::OnceLock;

use axum::Router;
use dandori_module_api::{
    Manifest, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup, ModuleRisk, ModuleScope,
    ModuleSubscription, UiContribution, UiDeclaration, WorkerSpec,
};

pub use calculator::{calculate_median, calculate_p80, evaluate_eta};
pub use model::{EtaAccuracyStats, EtaConfidence, EtaPredictionRecord, EtaResult};
pub use routes::{EtaState, eta_router};

fn default_ui() -> UiDeclaration {
    UiDeclaration {
        contributes: vec![
            UiContribution {
                id: "eta-card-badge".to_string(),
                slot: "board.card.badge".to_string(),
                label: "Dự tính hoàn thành".to_string(),
                order: 50,
            },
            UiContribution {
                id: "eta-task-panel-section".to_string(),
                slot: "board.task-panel.section".to_string(),
                label: "Ước lượng thời gian (ETA)".to_string(),
                order: 50,
            },
        ],
    }
}

fn get_manifest() -> Manifest {
    Manifest {
        id: "eta".to_string(),
        group: ModuleGroup::Dashboard,
        name: "Dự tính thời gian (ETA)".to_string(),
        description:
            "Dự tính khoảng thời gian hoàn tất tác vụ theo thống kê thực tế và tự đo sai số"
                .to_string(),
        version: "0.1.0".to_string(),
        requires: vec!["run-history".to_string()],
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
    id: "0001_eta_schema",
    sql: include_str!("../migrations/0001_eta_schema.sql"),
}];

/// Module dự tính thời gian hoàn thành tác vụ.
#[derive(Debug, Default)]
pub struct EtaModule;

impl EtaModule {
    /// Khởi tạo instance mới của `EtaModule`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for EtaModule {
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
        eta_router(EtaState::new(ctx.clone()))
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        Vec::new()
    }
}

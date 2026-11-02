//! Module `run-history`: Xem lại lịch sử từng lần chạy, phát lại log và ghi chú (P2-01).

use axum::Router;
use dandori_module_api::{
    BoxFuture, Manifest, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup, ModuleRisk,
    ModuleScope, ModuleSubscription, SettingSchema, UiContribution, UiDeclaration, WorkerSpec,
};
use std::collections::HashMap;
use std::sync::OnceLock;

pub mod handlers;
pub mod model;
pub mod routes;
pub mod worker;

pub use model::{
    ExportQuery, ListEventsQuery, ListRunsQuery, RunDetail, RunEventItem, RunHistoryNote,
    RunSummary, UpdateNotePayload,
};

fn get_manifest() -> Manifest {
    let mut settings = HashMap::new();
    settings.insert(
        "retentionDays".to_string(),
        SettingSchema {
            type_name: "number".to_string(),
            default: serde_json::json!(90),
            enum_values: None,
            min: Some(7.0),
            max: None,
        },
    );

    Manifest {
        id: "run-history".to_string(),
        group: ModuleGroup::Dashboard,
        name: "Lịch sử lần chạy".to_string(),
        description: "Xem lại chi tiết, phát lại log và ghi chú cho từng lần chạy".to_string(),
        version: "0.1.0".to_string(),
        requires: Vec::new(),
        scopes: vec![ModuleScope::Workspace, ModuleScope::Project],
        effort: ModuleEffort::Medium,
        risk: ModuleRisk::Low,
        risk_note: None,
        core: false,
        requires_worker_restart: false,
        settings,
        ui: UiDeclaration {
            contributes: vec![UiContribution {
                id: "run-history-tab".to_string(),
                slot: "board.task-panel.tab".to_string(),
                label: "Lịch sử".to_string(),
                order: 30,
            }],
        },
    }
}

const MIGRATIONS: &[Migration] = &[Migration {
    id: "0001_run_history",
    sql: include_str!("../migrations/0001_run_history.sql"),
}];

/// Cấu trúc đại diện cho module `run-history`.
#[derive(Debug, Clone, Default)]
pub struct RunHistoryModule;

impl RunHistoryModule {
    /// Khởi tạo module RunHistoryModule mới.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for RunHistoryModule {
    fn manifest(&self) -> &Manifest {
        static MANIFEST: OnceLock<Manifest> = OnceLock::new();
        MANIFEST.get_or_init(get_manifest)
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        routes::create_router(ctx.clone())
    }

    fn workers(&self, ctx: &ModuleContext) -> Vec<WorkerSpec> {
        let ctx = ctx.clone();
        vec![WorkerSpec::new(
            "run-history-retention",
            move |cancel_token| {
                let ctx = ctx.clone();
                Box::pin(async move {
                    worker::run_retention_worker(ctx, cancel_token).await;
                }) as BoxFuture<'static, ()>
            },
        )]
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }
}

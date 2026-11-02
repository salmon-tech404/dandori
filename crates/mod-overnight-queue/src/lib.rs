//! Module `overnight-queue`: Hàng đợi chạy qua đêm tuần tự an toàn, giới hạn song song và dừng an toàn (P3-04).

#![warn(missing_docs)]

pub mod model;
pub mod routes;
pub mod worker;

use axum::Router;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use dandori_module_api::{
    BoxFuture, Manifest, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup, ModuleRisk,
    ModuleScope, ModuleSubscription, SettingSchema, TransitionGuard, UiContribution, UiDeclaration,
    WorkerSpec,
};

pub use model::{MorningSummaryDto, OvernightQueueConfig, OvernightQueueItem, QueueStatus};
pub use routes::{OvernightQueueState, overnight_queue_router};
pub use worker::{
    Clock, MockClock, OvernightQueueWorkerConfig, RealClock, run_overnight_queue_worker,
};

fn default_settings() -> HashMap<String, SettingSchema> {
    let mut settings = HashMap::new();
    settings.insert(
        "defaultStartTime".to_string(),
        SettingSchema {
            type_name: "string".to_string(),
            default: serde_json::json!("23:00"),
            enum_values: None,
            min: None,
            max: None,
        },
    );
    settings.insert(
        "defaultMaxConcurrency".to_string(),
        SettingSchema {
            type_name: "number".to_string(),
            default: serde_json::json!(1),
            enum_values: None,
            min: Some(1.0),
            max: Some(10.0),
        },
    );
    settings
}

fn default_ui() -> UiDeclaration {
    UiDeclaration {
        contributes: vec![
            UiContribution {
                id: "overnight-queue-banner".to_string(),
                slot: "calendar.panel".to_string(),
                label: "Hàng đợi qua đêm".to_string(),
                order: 40,
            },
            UiContribution {
                id: "overnight-queue-toolbar".to_string(),
                slot: "board.toolbar".to_string(),
                label: "Qua đêm".to_string(),
                order: 45,
            },
        ],
    }
}

fn get_manifest() -> Manifest {
    Manifest {
        id: "overnight-queue".to_string(),
        group: ModuleGroup::Scheduling,
        name: "Hàng đợi qua đêm (Overnight Queue)".to_string(),
        description:
            "Quản lý hàng đợi chạy tuần tự qua đêm an toàn với giới hạn song song và dừng an toàn"
                .to_string(),
        version: "0.1.0".to_string(),
        requires: vec!["calendar".to_string()],
        scopes: vec![ModuleScope::Workspace],
        effort: ModuleEffort::Medium,
        risk: ModuleRisk::Low,
        risk_note: None,
        core: false,
        requires_worker_restart: false,
        settings: default_settings(),
        ui: default_ui(),
        allowed_workdir_files: Vec::new(),
    }
}

const MIGRATIONS: &[Migration] = &[Migration {
    id: "0001_overnight_queue",
    sql: include_str!("../migrations/0001_overnight_queue.sql"),
}];

/// Module `overnight-queue` tích hợp vào hệ thống Dandori.
#[derive(Debug, Clone, Default)]
pub struct OvernightQueueModule;

impl OvernightQueueModule {
    /// Khởi tạo instance mới của `OvernightQueueModule`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for OvernightQueueModule {
    fn manifest(&self) -> &Manifest {
        static MANIFEST: OnceLock<Manifest> = OnceLock::new();
        MANIFEST.get_or_init(get_manifest)
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        let state = OvernightQueueState::new(ctx.clone());
        overnight_queue_router(state)
    }

    fn workers(&self, ctx: &ModuleContext) -> Vec<WorkerSpec> {
        let pool = ctx.pool().cloned();
        let event_bus = ctx.event_bus().clone();
        let runner = ctx.task_runner().cloned();

        vec![WorkerSpec::new(
            "overnight-queue-worker",
            move |cancel_token| {
                let pool = pool.clone();
                let bus = event_bus.clone();
                let r = runner.clone();
                Box::pin(async move {
                    if let Some(p) = pool {
                        run_overnight_queue_worker(
                            p,
                            Arc::new(RealClock),
                            cancel_token,
                            bus,
                            r,
                            OvernightQueueWorkerConfig::default(),
                        )
                        .await;
                    }
                }) as BoxFuture<'static, ()>
            },
        )]
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }

    fn transition_guard(&self, _ctx: &ModuleContext) -> Option<Arc<dyn TransitionGuard>> {
        None
    }
}

/// Hàm xuất tạo module phục vụ nạp động / registry.
#[must_use]
pub fn create_module() -> Box<dyn Module> {
    Box::new(OvernightQueueModule::new())
}

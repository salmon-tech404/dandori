//! Module `approval-gate`: Cổng duyệt của con người; chặn chuyển trạng thái nhạy cảm (P2-03).

#![warn(missing_docs)]

pub mod guard;
pub mod handlers;
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
use guard::ApprovalGateGuard;
use handlers::ApprovalGateState;

fn default_settings() -> HashMap<String, SettingSchema> {
    let mut settings = HashMap::new();
    settings.insert(
        "require_in_review_to_merge".to_string(),
        SettingSchema {
            type_name: "boolean".to_string(),
            default: serde_json::json!(true),
            enum_values: None,
            min: None,
            max: None,
        },
    );
    settings.insert(
        "disallow_self_approval".to_string(),
        SettingSchema {
            type_name: "boolean".to_string(),
            default: serde_json::json!(true),
            enum_values: None,
            min: None,
            max: None,
        },
    );
    settings.insert(
        "expiry_hours".to_string(),
        SettingSchema {
            type_name: "number".to_string(),
            default: serde_json::json!(24),
            enum_values: None,
            min: Some(1.0),
            max: Some(720.0),
        },
    );
    settings
}

fn default_ui() -> UiDeclaration {
    UiDeclaration {
        contributes: vec![
            UiContribution {
                id: "approval-gate-chip".to_string(),
                slot: "board.header.chip".to_string(),
                label: "Cần bạn".to_string(),
                order: 20,
            },
            UiContribution {
                id: "approval-gate-tab".to_string(),
                slot: "board.task-panel.tab".to_string(),
                label: "Duyệt".to_string(),
                order: 40,
            },
            UiContribution {
                id: "approval-gate-kpi".to_string(),
                slot: "dashboard.kpi".to_string(),
                label: "Cần bạn".to_string(),
                order: 20,
            },
        ],
    }
}

fn get_manifest() -> Manifest {
    Manifest {
        id: "approval-gate".to_string(),
        group: ModuleGroup::Orchestration,
        name: "Cổng duyệt".to_string(),
        description:
            "Cổng duyệt của con người; chặn chuyển trạng thái nhạy cảm cho đến khi được duyệt"
                .to_string(),
        version: "0.1.0".to_string(),
        requires: Vec::new(),
        scopes: vec![
            ModuleScope::System,
            ModuleScope::Workspace,
            ModuleScope::Project,
        ],
        effort: ModuleEffort::Medium,
        risk: ModuleRisk::Low,
        risk_note: None,
        core: false,
        requires_worker_restart: false,
        settings: default_settings(),
        ui: default_ui(),
    }
}

const MIGRATIONS: &[Migration] = &[Migration {
    id: "0001_approval_gate",
    sql: include_str!("../migrations/0001_approval_gate.sql"),
}];

/// Module `approval-gate` tích hợp vào hệ thống Dandori.
#[derive(Debug, Clone, Default)]
pub struct ApprovalGateModule;

impl ApprovalGateModule {
    /// Khởi tạo instance mới của `ApprovalGateModule`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for ApprovalGateModule {
    fn manifest(&self) -> &Manifest {
        static MANIFEST: OnceLock<Manifest> = OnceLock::new();
        MANIFEST.get_or_init(get_manifest)
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        let state = ApprovalGateState { ctx: ctx.clone() };
        routes::create_routes(state)
    }

    fn workers(&self, ctx: &ModuleContext) -> Vec<WorkerSpec> {
        let pool = ctx.pool().cloned();
        let event_bus = ctx.event_bus().clone();
        vec![WorkerSpec::new(
            "approval-gate-expiry-worker",
            move |cancel_token| {
                let pool = pool.clone();
                let bus = event_bus.clone();
                Box::pin(async move {
                    if let Some(p) = pool {
                        worker::run_expiry_worker(p, bus, cancel_token).await;
                    }
                }) as BoxFuture<'static, ()>
            },
        )]
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }

    fn transition_guard(&self, ctx: &ModuleContext) -> Option<Arc<dyn TransitionGuard>> {
        Some(Arc::new(ApprovalGateGuard::new(
            ctx.pool().cloned(),
            ctx.event_bus().clone(),
        )))
    }
}

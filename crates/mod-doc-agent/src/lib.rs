//! Module `doc-agent`: Agent viết tài liệu tự chạy khi task hoàn thành (P2-05).

#![warn(missing_docs)]

pub mod handlers;
pub mod model;
pub mod pattern;
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
use handlers::DocAgentState;

fn default_settings() -> HashMap<String, SettingSchema> {
    let mut settings = HashMap::new();
    settings.insert(
        "enabled".to_string(),
        SettingSchema {
            type_name: "boolean".to_string(),
            default: serde_json::json!(true),
            enum_values: None,
            min: None,
            max: None,
        },
    );
    settings.insert(
        "prompt_template".to_string(),
        SettingSchema {
            type_name: "string".to_string(),
            default: serde_json::json!(""),
            enum_values: None,
            min: None,
            max: None,
        },
    );
    settings.insert(
        "timeout_seconds".to_string(),
        SettingSchema {
            type_name: "number".to_string(),
            default: serde_json::json!(600),
            enum_values: None,
            min: Some(30.0),
            max: Some(3600.0),
        },
    );
    settings
}

fn default_ui() -> UiDeclaration {
    UiDeclaration {
        contributes: vec![
            UiContribution {
                id: "doc-agent-task-section".to_string(),
                slot: "board.task-panel.section".to_string(),
                label: "Tài liệu".to_string(),
                order: 35,
            },
            UiContribution {
                id: "doc-agent-project-settings".to_string(),
                slot: "settings.section".to_string(),
                label: "Cấu hình Doc Agent".to_string(),
                order: 45,
            },
        ],
    }
}

fn get_manifest() -> Manifest {
    Manifest {
        id: "doc-agent".to_string(),
        group: ModuleGroup::Orchestration,
        name: "Agent viết tài liệu".to_string(),
        description: "Agent viết tài liệu tự chạy khi task hoàn thành (ready_to_merge)".to_string(),
        version: "0.1.0".to_string(),
        requires: vec!["lead-worker".to_string()],
        scopes: vec![ModuleScope::Workspace, ModuleScope::Project],
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
    id: "0001_doc_agent",
    sql: include_str!("../migrations/0001_doc_agent.sql"),
}];

/// Module `doc-agent` tích hợp vào hệ thống Dandori.
#[derive(Debug, Clone, Default)]
pub struct DocAgentModule;

impl DocAgentModule {
    /// Khởi tạo instance mới của `DocAgentModule`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for DocAgentModule {
    fn manifest(&self) -> &Manifest {
        static MANIFEST: OnceLock<Manifest> = OnceLock::new();
        MANIFEST.get_or_init(get_manifest)
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        let state = DocAgentState { ctx: ctx.clone() };
        routes::create_routes(state)
    }

    fn workers(&self, ctx: &ModuleContext) -> Vec<WorkerSpec> {
        let pool = ctx.pool().cloned();
        let event_bus = ctx.event_bus().clone();
        let runner = ctx.task_runner().cloned();

        vec![WorkerSpec::new(
            "doc-agent-runner-worker",
            move |cancel_token| {
                let pool = pool.clone();
                let bus = event_bus.clone();
                let runner = runner.clone();
                Box::pin(async move {
                    if let Some(p) = pool {
                        worker::run_doc_agent_worker(p, bus, runner, cancel_token).await;
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

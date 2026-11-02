//! Module `test-agent`: Agent test chuyên trách tự chạy sau khi worker xong (P2-04).

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
use handlers::TestAgentState;

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
        "suggested_command".to_string(),
        SettingSchema {
            type_name: "string".to_string(),
            default: serde_json::json!(""),
            enum_values: None,
            min: None,
            max: None,
        },
    );
    settings.insert(
        "max_test_runs_per_worker".to_string(),
        SettingSchema {
            type_name: "number".to_string(),
            default: serde_json::json!(1),
            enum_values: None,
            min: Some(1.0),
            max: Some(5.0),
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
                id: "test-agent-task-section".to_string(),
                slot: "board.task-panel.section".to_string(),
                label: "Kiểm thử".to_string(),
                order: 30,
            },
            UiContribution {
                id: "test-agent-card-badge".to_string(),
                slot: "board.card.badge".to_string(),
                label: "Kết quả test".to_string(),
                order: 20,
            },
            UiContribution {
                id: "test-agent-project-settings".to_string(),
                slot: "settings.section".to_string(),
                label: "Cấu hình Test Agent".to_string(),
                order: 40,
            },
        ],
    }
}

fn get_manifest() -> Manifest {
    Manifest {
        id: "test-agent".to_string(),
        group: ModuleGroup::Orchestration,
        name: "Agent test chuyên trách".to_string(),
        description: "Agent test chuyên trách tự chạy sau khi worker xong".to_string(),
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
    id: "0001_test_agent",
    sql: include_str!("../migrations/0001_test_agent.sql"),
}];

/// Module `test-agent` tích hợp vào hệ thống Dandori.
#[derive(Debug, Clone, Default)]
pub struct TestAgentModule;

impl TestAgentModule {
    /// Khởi tạo instance mới của `TestAgentModule`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for TestAgentModule {
    fn manifest(&self) -> &Manifest {
        static MANIFEST: OnceLock<Manifest> = OnceLock::new();
        MANIFEST.get_or_init(get_manifest)
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        let state = TestAgentState { ctx: ctx.clone() };
        routes::create_routes(state)
    }

    fn workers(&self, ctx: &ModuleContext) -> Vec<WorkerSpec> {
        let pool = ctx.pool().cloned();
        let event_bus = ctx.event_bus().clone();
        let runner = ctx.task_runner().cloned();

        vec![WorkerSpec::new(
            "test-agent-runner-worker",
            move |cancel_token| {
                let pool = pool.clone();
                let bus = event_bus.clone();
                let runner = runner.clone();
                Box::pin(async move {
                    if let Some(p) = pool {
                        worker::run_test_agent_worker(p, bus, runner, cancel_token).await;
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

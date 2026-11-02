//! Module `auto-lessons`: Tự động trích xuất và đề xuất bài học vào bộ nhớ chung sau khi task hoàn thành (P3-11).

#![warn(missing_docs)]

pub mod model;
pub mod redaction;
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
use routes::{AutoLessonsState, create_routes};

fn default_settings() -> HashMap<String, SettingSchema> {
    let mut settings = HashMap::new();
    settings.insert(
        "trigger_on_done".to_string(),
        SettingSchema {
            type_name: "boolean".to_string(),
            default: serde_json::json!(true),
            enum_values: None,
            min: None,
            max: None,
        },
    );
    settings.insert(
        "trigger_on_failed".to_string(),
        SettingSchema {
            type_name: "boolean".to_string(),
            default: serde_json::json!(true),
            enum_values: None,
            min: None,
            max: None,
        },
    );
    settings.insert(
        "max_token_budget".to_string(),
        SettingSchema {
            type_name: "number".to_string(),
            default: serde_json::json!(1000),
            enum_values: None,
            min: Some(100.0),
            max: Some(10000.0),
        },
    );
    settings.insert(
        "min_content_length".to_string(),
        SettingSchema {
            type_name: "number".to_string(),
            default: serde_json::json!(10),
            enum_values: None,
            min: Some(1.0),
            max: Some(500.0),
        },
    );
    settings
}

fn default_ui() -> UiDeclaration {
    UiDeclaration {
        contributes: vec![UiContribution {
            id: "auto-lessons-settings".to_string(),
            slot: "settings.section".to_string(),
            label: "Tự động rút bài học".to_string(),
            order: 55,
        }],
    }
}

fn get_manifest() -> Manifest {
    Manifest {
        id: "auto-lessons".to_string(),
        group: ModuleGroup::Knowledge,
        name: "Tự động rút bài học".to_string(),
        description:
            "Tự động trích xuất và đề xuất bài học vào bộ nhớ chung sau khi task hoàn thành"
                .to_string(),
        version: "0.1.0".to_string(),
        requires: vec!["shared-memory".to_string()],
        scopes: vec![ModuleScope::Workspace, ModuleScope::Project],
        effort: ModuleEffort::Light,
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
    id: "0001_auto_lessons_schema",
    sql: include_str!("../migrations/0001_auto_lessons_schema.sql"),
}];

/// Module `auto-lessons` tích hợp vào hệ thống Dandori.
#[derive(Debug, Clone, Default)]
pub struct AutoLessonsModule;

impl AutoLessonsModule {
    /// Khởi tạo instance mới của `AutoLessonsModule`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for AutoLessonsModule {
    fn manifest(&self) -> &Manifest {
        static MANIFEST: OnceLock<Manifest> = OnceLock::new();
        MANIFEST.get_or_init(get_manifest)
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        let state = AutoLessonsState { ctx: ctx.clone() };
        create_routes(state)
    }

    fn workers(&self, ctx: &ModuleContext) -> Vec<WorkerSpec> {
        let pool = ctx.pool().cloned();
        let event_bus = ctx.event_bus().clone();
        let runner = ctx.task_runner().cloned();

        vec![WorkerSpec::new(
            "auto-lessons-worker",
            move |cancel_token| {
                let pool = pool.clone();
                let bus = event_bus.clone();
                let runner = runner.clone();
                Box::pin(async move {
                    if let Some(p) = pool {
                        worker::run_auto_lessons_worker(p, bus, runner, cancel_token).await;
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

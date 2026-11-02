//! Module `notifications`: Gửi thông báo sự kiện qua Telegram, Discord và Slack (P3-12).

#![warn(missing_docs)]

pub mod model;
pub mod routes;
pub mod sender;
pub mod vault;
pub mod worker;

use axum::Router;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use dandori_module_api::{
    BoxFuture, Manifest, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup, ModuleRisk,
    ModuleScope, ModuleSubscription, SettingSchema, TransitionGuard, UiContribution, UiDeclaration,
    WorkerSpec,
};
use routes::NotificationsState;

fn default_settings() -> HashMap<String, SettingSchema> {
    let mut settings = HashMap::new();
    settings.insert(
        "rate_limit_per_minute".to_string(),
        SettingSchema {
            type_name: "number".to_string(),
            default: serde_json::json!(10),
            enum_values: None,
            min: Some(1.0),
            max: Some(100.0),
        },
    );
    settings.insert(
        "dedup_window_seconds".to_string(),
        SettingSchema {
            type_name: "number".to_string(),
            default: serde_json::json!(300),
            enum_values: None,
            min: Some(10.0),
            max: Some(3600.0),
        },
    );
    settings
}

fn default_ui() -> UiDeclaration {
    UiDeclaration {
        contributes: vec![UiContribution {
            id: "notifications-settings".to_string(),
            slot: "settings.section".to_string(),
            label: "Kênh thông báo".to_string(),
            order: 60,
        }],
    }
}

fn get_manifest() -> Manifest {
    Manifest {
        id: "notifications".to_string(),
        group: ModuleGroup::Integrations,
        name: "Kênh thông báo".to_string(),
        description:
            "Gửi thông báo sự kiện qua Telegram, Discord và Slack với bảo vệ SSRF và kho khóa"
                .to_string(),
        version: "0.1.0".to_string(),
        requires: Vec::new(),
        scopes: vec![ModuleScope::Workspace],
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
    id: "0001_notifications_schema",
    sql: include_str!("../migrations/0001_notifications_schema.sql"),
}];

/// Module `notifications` trong hệ thống Dandori.
#[derive(Debug, Clone, Default)]
pub struct NotificationsModule;

impl NotificationsModule {
    /// Khởi tạo instance mới của `NotificationsModule`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for NotificationsModule {
    fn manifest(&self) -> &Manifest {
        static MANIFEST: OnceLock<Manifest> = OnceLock::new();
        MANIFEST.get_or_init(get_manifest)
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        if let Some(pool) = ctx.pool() {
            let state = NotificationsState { pool: pool.clone() };
            routes::router(state)
        } else {
            Router::new()
        }
    }

    fn workers(&self, ctx: &ModuleContext) -> Vec<WorkerSpec> {
        let pool = ctx.pool().cloned();
        let event_bus = ctx.event_bus().clone();

        vec![WorkerSpec::new(
            "notifications-worker",
            move |cancel_token| {
                let pool = pool.clone();
                let bus = event_bus.clone();
                Box::pin(async move {
                    if let Some(p) = pool {
                        worker::run_notifications_worker(p, bus, cancel_token).await;
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

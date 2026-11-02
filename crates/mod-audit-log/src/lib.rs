//! Module `audit-log`: nhật ký kiểm toán bất biến chỉ thêm với chuỗi băm mật mã (P3-14).

#![warn(missing_docs)]

pub mod hash_chain;
pub mod model;
pub mod routes;
pub mod worker;

use axum::Router;
use dandori_module_api::{
    BoxFuture, Manifest, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup, ModuleRisk,
    ModuleScope, ModuleSubscription, SettingSchema, UiContribution, UiDeclaration, WorkerSpec,
};
use routes::AuditLogState;
use std::collections::HashMap;
use std::sync::OnceLock;

fn default_settings() -> HashMap<String, SettingSchema> {
    let mut settings = HashMap::new();
    settings.insert(
        "retention_days".to_string(),
        SettingSchema {
            type_name: "number".to_string(),
            default: serde_json::json!(90),
            enum_values: None,
            min: Some(1.0),
            max: Some(3650.0),
        },
    );
    settings
}

fn default_ui() -> UiDeclaration {
    UiDeclaration {
        contributes: vec![UiContribution {
            id: "audit-log-settings".to_string(),
            slot: "settings.section".to_string(),
            label: "Nhật ký kiểm toán".to_string(),
            order: 80,
        }],
    }
}

fn get_manifest() -> Manifest {
    Manifest {
        id: "audit-log".to_string(),
        group: ModuleGroup::Safety,
        name: "Nhật ký kiểm toán".to_string(),
        description: "Nhật ký kiểm toán có tính toàn vẹn với chuỗi băm mật mã".to_string(),
        version: "0.1.0".to_string(),
        requires: Vec::new(),
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
    id: "0001_audit_log_schema",
    sql: include_str!("../migrations/0001_audit_log_schema.sql"),
}];

/// Module audit-log ghi nhận nhật ký kiểm toán bất biến và xác minh chuỗi băm.
#[derive(Default, Debug, Clone)]
pub struct AuditLogModule;

impl AuditLogModule {
    /// Khởi tạo instance mới của AuditLogModule.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for AuditLogModule {
    fn manifest(&self) -> &Manifest {
        static MANIFEST: OnceLock<Manifest> = OnceLock::new();
        MANIFEST.get_or_init(get_manifest)
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        if let Some(pool) = ctx.pool() {
            routes::router(AuditLogState { pool: pool.clone() })
        } else {
            Router::new()
        }
    }

    fn workers(&self, ctx: &ModuleContext) -> Vec<WorkerSpec> {
        let pool = ctx.pool().cloned();
        let event_bus = ctx.event_bus().clone();

        vec![WorkerSpec::new("audit-log-worker", move |cancel_token| {
            let pool = pool.clone();
            let bus = event_bus.clone();
            Box::pin(async move {
                if let Some(p) = pool {
                    worker::run_audit_worker(p, bus, cancel_token).await;
                }
            }) as BoxFuture<'static, ()>
        })]
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }
}

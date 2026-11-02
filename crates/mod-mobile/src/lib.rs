//! Module kết nối thiết bị di động an toàn và giao diện thu gọn (P3-13).

#![warn(missing_docs)]

pub mod auth;
pub mod model;
pub mod routes;

use axum::Router;
use dandori_module_api::{
    Manifest, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup, ModuleRisk, ModuleScope,
    ModuleSubscription, SettingSchema, UiContribution, UiDeclaration, WorkerSpec,
};
use routes::MobileState;
use std::collections::HashMap;
use std::sync::OnceLock;

fn default_settings() -> HashMap<String, SettingSchema> {
    let mut settings = HashMap::new();
    settings.insert(
        "token_ttl_hours".to_string(),
        SettingSchema {
            type_name: "number".to_string(),
            default: serde_json::json!(720),
            enum_values: None,
            min: Some(1.0),
            max: Some(8760.0),
        },
    );
    settings
}

fn default_ui() -> UiDeclaration {
    UiDeclaration {
        contributes: vec![UiContribution {
            id: "mobile-settings".to_string(),
            slot: "settings.section".to_string(),
            label: "Thiết bị di động".to_string(),
            order: 70,
        }],
    }
}

fn get_manifest() -> Manifest {
    Manifest {
        id: "mobile".to_string(),
        group: ModuleGroup::Dashboard,
        name: "Truy cập di động".to_string(),
        description: "Theo dõi tiến trình và phê duyệt từ thiết bị di động an toàn".to_string(),
        version: "0.1.0".to_string(),
        requires: Vec::new(),
        scopes: vec![ModuleScope::Workspace],
        effort: ModuleEffort::Medium,
        risk: ModuleRisk::Medium,
        risk_note: Some(
            "Cần cấu hình đúng danh sách Origin/Host cho phép để phòng chống rò rỉ".to_string(),
        ),
        core: false,
        requires_worker_restart: false,
        settings: default_settings(),
        ui: default_ui(),
        allowed_workdir_files: Vec::new(),
    }
}

const MIGRATIONS: &[Migration] = &[Migration {
    id: "0001_mobile_schema",
    sql: include_str!("../migrations/0001_mobile_schema.sql"),
}];

/// Module mobile quản lý thiết bị di động và phục vụ trang /m an toàn.
#[derive(Default, Debug, Clone)]
pub struct MobileModule;

impl MobileModule {
    /// Khởi tạo instance mới của MobileModule.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for MobileModule {
    fn manifest(&self) -> &Manifest {
        static MANIFEST: OnceLock<Manifest> = OnceLock::new();
        MANIFEST.get_or_init(get_manifest)
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        if let Some(pool) = ctx.pool() {
            routes::router(MobileState { pool: pool.clone() })
        } else {
            Router::new()
        }
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        Vec::new()
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }
}

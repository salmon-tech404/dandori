//! Module `quiet-hours`: Khung giờ cấm chạy tác vụ tự động và bảo vệ người dùng (P3-03).

#![warn(missing_docs)]

pub mod guard;
pub mod model;
pub mod routes;
pub mod window;

use axum::Router;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use dandori_module_api::{
    Manifest, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup, ModuleRisk, ModuleScope,
    ModuleSubscription, SettingSchema, TransitionGuard, UiContribution, UiDeclaration, WorkerSpec,
};

pub use guard::{Clock, MockClock, QuietHoursGuard, RealClock};
pub use model::{CreateQuietWindowDto, QuietStatusDto, QuietWindow, UpdateQuietWindowDto};
pub use routes::{QuietHoursState, quiet_hours_router};

fn default_settings() -> HashMap<String, SettingSchema> {
    let mut settings = HashMap::new();
    settings.insert(
        "defaultTimezone".to_string(),
        SettingSchema {
            type_name: "string".to_string(),
            default: serde_json::json!("UTC"),
            enum_values: None,
            min: None,
            max: None,
        },
    );
    settings
}

fn default_ui() -> UiDeclaration {
    UiDeclaration {
        contributes: vec![
            UiContribution {
                id: "quiet-hours-toolbar".to_string(),
                slot: "calendar.toolbar".to_string(),
                label: "Giờ cấm".to_string(),
                order: 30,
            },
            UiContribution {
                id: "quiet-hours-settings".to_string(),
                slot: "settings.panel".to_string(),
                label: "Cấu hình giờ cấm".to_string(),
                order: 35,
            },
        ],
    }
}

fn get_manifest() -> Manifest {
    Manifest {
        id: "quiet-hours".to_string(),
        group: ModuleGroup::Scheduling,
        name: "Khung giờ cấm chạy (Quiet Hours)".to_string(),
        description: "Quản lý khung giờ cấm chạy cho các tác vụ tự động và bảo vệ người dùng"
            .to_string(),
        version: "0.1.0".to_string(),
        requires: vec!["calendar".to_string()],
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
    id: "0001_quiet_hours",
    sql: include_str!("../migrations/0001_quiet_hours.sql"),
}];

/// Module `quiet-hours` tích hợp vào Dandori.
#[derive(Debug, Clone, Default)]
pub struct QuietHoursModule;

impl QuietHoursModule {
    /// Khởi tạo instance mới của `QuietHoursModule`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for QuietHoursModule {
    fn manifest(&self) -> &Manifest {
        static MANIFEST: OnceLock<Manifest> = OnceLock::new();
        MANIFEST.get_or_init(get_manifest)
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        let state = QuietHoursState::new(ctx.clone());
        quiet_hours_router(state)
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        Vec::new()
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }

    fn transition_guard(&self, ctx: &ModuleContext) -> Option<Arc<dyn TransitionGuard>> {
        let pool = ctx.pool()?.clone();
        Some(Arc::new(QuietHoursGuard::new(pool, Arc::new(RealClock))))
    }
}

/// Hàm xuất tạo module phục vụ nạp động / registry.
#[must_use]
pub fn create_module() -> Box<dyn Module> {
    Box::new(QuietHoursModule::new())
}

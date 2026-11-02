//! Module `diff-preview`: Xem diff trong giao diện và mở bản xem trước (P2-02).

#![warn(missing_docs)]

pub mod handlers;
pub mod manager;
pub mod model;
pub mod routes;
pub mod worker;

use axum::Router;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use dandori_module_api::{
    BoxFuture, Manifest, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup, ModuleRisk,
    ModuleScope, ModuleSubscription, SettingSchema, UiContribution, UiDeclaration, WorkerSpec,
};
use handlers::DiffPreviewState;
use manager::PreviewManager;

fn get_manifest() -> Manifest {
    let mut settings = HashMap::new();
    settings.insert(
        "preview_command".to_string(),
        SettingSchema {
            type_name: "string".to_string(),
            default: serde_json::json!("[]"),
            enum_values: None,
            min: None,
            max: None,
        },
    );
    settings.insert(
        "preview_port".to_string(),
        SettingSchema {
            type_name: "number".to_string(),
            default: serde_json::json!(3000),
            enum_values: None,
            min: Some(1024.0),
            max: Some(65535.0),
        },
    );

    Manifest {
        id: "diff-preview".to_string(),
        group: ModuleGroup::Dashboard,
        name: "Diff & Xem trước".to_string(),
        description: "Xem diff thay đổi trong giao diện và mở bản xem trước kết quả".to_string(),
        version: "0.1.0".to_string(),
        requires: vec!["worktree".to_string()],
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
        settings,
        ui: UiDeclaration {
            contributes: vec![UiContribution {
                id: "diff-preview-tab".to_string(),
                slot: "board.task-panel.tab".to_string(),
                label: "Diff".to_string(),
                order: 30,
            }],
        },
    }
}

const MIGRATIONS: &[Migration] = &[Migration {
    id: "0001_diff_preview",
    sql: include_str!("../migrations/0001_diff_preview.sql"),
}];

/// Module `diff-preview` tích hợp vào hệ thống Dandori.
#[derive(Debug, Clone)]
pub struct DiffPreviewModule {
    manager: Arc<PreviewManager>,
}

impl Default for DiffPreviewModule {
    fn default() -> Self {
        Self::new()
    }
}

impl DiffPreviewModule {
    /// Khởi tạo instance mới của `DiffPreviewModule`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            manager: Arc::new(PreviewManager::new()),
        }
    }

    /// Lấy tham chiếu tới trình quản lý phiên xem trước.
    #[must_use]
    pub fn manager(&self) -> Arc<PreviewManager> {
        Arc::clone(&self.manager)
    }
}

impl Module for DiffPreviewModule {
    fn manifest(&self) -> &Manifest {
        static MANIFEST: OnceLock<Manifest> = OnceLock::new();
        MANIFEST.get_or_init(get_manifest)
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        let state = DiffPreviewState {
            ctx: ctx.clone(),
            manager: Arc::clone(&self.manager),
        };
        routes::create_routes(state)
    }

    fn workers(&self, ctx: &ModuleContext) -> Vec<WorkerSpec> {
        let bus = ctx.event_bus().clone();
        let mgr = Arc::clone(&self.manager);
        vec![WorkerSpec::new(
            "diff-preview-worker",
            move |cancel_token| {
                let bus = bus.clone();
                let mgr = mgr.clone();
                Box::pin(async move {
                    worker::run_preview_worker(bus, mgr, cancel_token).await;
                }) as BoxFuture<'static, ()>
            },
        )]
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }
}

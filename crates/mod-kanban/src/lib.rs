//! Module `kanban`: Bảng nhiệm vụ dạng Kanban theo trạng thái (P1-05).

use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use dandori_module_api::{
    Manifest, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup, ModuleRisk, ModuleScope,
    ModuleSubscription, UiContribution, UiDeclaration, WorkerSpec,
};
use serde_json::json;

fn get_manifest() -> Manifest {
    Manifest {
        id: "kanban".to_string(),
        group: ModuleGroup::Dashboard,
        name: "Bảng Kanban".to_string(),
        description: "Bảng nhiệm vụ theo trạng thái".to_string(),
        version: "0.1.0".to_string(),
        requires: vec![],
        scopes: vec![ModuleScope::Workspace, ModuleScope::Project],
        effort: ModuleEffort::Medium,
        risk: ModuleRisk::Low,
        risk_note: None,
        core: false,
        requires_worker_restart: false,
        settings: std::collections::HashMap::new(),
        ui: UiDeclaration {
            contributes: vec![
                UiContribution {
                    id: "kanban-board".to_string(),
                    slot: "board.columns".to_string(),
                    label: "Bảng Kanban".to_string(),
                    order: 10,
                },
                UiContribution {
                    id: "kanban-header-chip".to_string(),
                    slot: "board.header.chip".to_string(),
                    label: "Chỉ số trạng thái".to_string(),
                    order: 10,
                },
            ],
        },
    }
}

/// Cấu trúc đại diện cho module `kanban`.
#[derive(Debug, Clone, Default)]
pub struct KanbanModule;

impl KanbanModule {
    /// Khởi tạo module Kanban mới.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

async fn handle_kanban_status() -> impl IntoResponse {
    Json(json!({
        "module": "kanban",
        "status": "active",
        "columns": [
            { "id": "backlog", "label": "Backlog" },
            { "id": "running", "label": "Đang chạy" },
            { "id": "needs_human", "label": "Cần bạn" },
            { "id": "in_review", "label": "Đang review" },
            { "id": "ready_to_merge", "label": "Sẵn sàng merge" }
        ]
    }))
}

impl Module for KanbanModule {
    fn manifest(&self) -> &Manifest {
        static MANIFEST: std::sync::OnceLock<Manifest> = std::sync::OnceLock::new();
        MANIFEST.get_or_init(get_manifest)
    }

    fn migrations(&self) -> &'static [Migration] {
        &[]
    }

    fn routes(&self, _ctx: &ModuleContext) -> Router {
        Router::new().route("/status", get(handle_kanban_status))
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        vec![]
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        vec![]
    }
}

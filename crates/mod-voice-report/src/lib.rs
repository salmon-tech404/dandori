//! Module đọc báo cáo bằng giọng nói qua Web Speech API của trình duyệt.

#![warn(missing_docs)]

use axum::Router;
use std::collections::HashMap;
use std::sync::OnceLock;

use dandori_module_api::{
    Manifest, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup, ModuleRisk, ModuleScope,
    ModuleSubscription, UiContribution, UiDeclaration, WorkerSpec,
};

fn default_ui() -> UiDeclaration {
    UiDeclaration {
        contributes: vec![UiContribution {
            id: "voice-report-control".to_string(),
            slot: "dashboard.panel".to_string(),
            label: "Giọng đọc báo cáo".to_string(),
            order: 45,
        }],
    }
}

fn get_manifest() -> &'static Manifest {
    static MANIFEST: OnceLock<Manifest> = OnceLock::new();
    MANIFEST.get_or_init(|| Manifest {
        id: "voice-report".to_string(),
        group: ModuleGroup::Dashboard,
        name: "Giọng đọc báo cáo (Voice Report)".to_string(),
        description: "Đọc báo cáo tổng kết bằng Web Speech API có sẵn của trình duyệt".to_string(),
        version: "0.1.0".to_string(),
        requires: vec!["reports".to_string()],
        scopes: vec![ModuleScope::Workspace],
        effort: ModuleEffort::Light,
        risk: ModuleRisk::Low,
        risk_note: None,
        core: false,
        requires_worker_restart: false,
        settings: HashMap::new(),
        ui: default_ui(),
        allowed_workdir_files: Vec::new(),
    })
}

/// Trình triển khai module `voice-report`.
#[derive(Debug, Default, Clone)]
pub struct VoiceReportModule;

impl VoiceReportModule {
    /// Khởi tạo module `voice-report` mới.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for VoiceReportModule {
    fn manifest(&self) -> &Manifest {
        get_manifest()
    }

    fn routes(&self, _ctx: &ModuleContext) -> Router {
        Router::new()
    }

    fn migrations(&self) -> &'static [Migration] {
        &[]
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        Vec::new()
    }
}

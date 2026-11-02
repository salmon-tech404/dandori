//! Module `reports`: Báo cáo tổng kết ngày và tuần mang tính xác định (P3-05).

#![warn(missing_docs)]

pub mod generator;
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

pub use generator::{
    calculate_median, calculate_period_bounds, export_report_to_markdown, generate_report_data,
    parse_timezone, redact_secrets,
};
pub use model::{
    AccountSwitchStats, CostStats, DurationStats, ProviderAccountUsage, ReportConfig, ReportCounts,
    ReportData, ReportSnapshot, TopTaskItem, UpdateReportConfigRequest,
};
pub use routes::{ReportsState, reports_router};
pub use worker::{Clock, MockClock, RealClock, ReportsWorkerConfig, run_reports_worker};

fn default_settings() -> HashMap<String, SettingSchema> {
    let mut settings = HashMap::new();
    settings.insert(
        "autoGenerate".to_string(),
        SettingSchema {
            type_name: "boolean".to_string(),
            default: serde_json::json!(false),
            enum_values: None,
            min: None,
            max: None,
        },
    );
    settings.insert(
        "timezone".to_string(),
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
        contributes: vec![UiContribution {
            id: "reports-panel".to_string(),
            slot: "dashboard.panel".to_string(),
            label: "Báo cáo tổng kết".to_string(),
            order: 40,
        }],
    }
}

fn get_manifest() -> Manifest {
    Manifest {
        id: "reports".to_string(),
        group: ModuleGroup::Dashboard,
        name: "Báo cáo tổng kết (Reports)".to_string(),
        description: "Báo cáo tổng kết ngày và tuần xác định, không số liệu bịa, xuất Markdown"
            .to_string(),
        version: "0.1.0".to_string(),
        requires: vec!["run-history".to_string()],
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
    id: "0001_reports",
    sql: include_str!("../migrations/0001_reports.sql"),
}];

/// Module báo cáo tổng kết ngày và tuần.
#[derive(Clone, Debug, Default)]
pub struct ReportsModule {
    clock: Option<Arc<dyn Clock>>,
}

impl ReportsModule {
    /// Khởi tạo module với đồng hồ mặc định.
    #[must_use]
    pub fn new() -> Self {
        Self { clock: None }
    }

    /// Khởi tạo module với đồng hồ tùy chỉnh cho kiểm thử.
    #[must_use]
    pub fn with_clock(clock: Arc<dyn Clock>) -> Self {
        Self { clock: Some(clock) }
    }
}

impl Module for ReportsModule {
    fn manifest(&self) -> &Manifest {
        static MANIFEST: OnceLock<Manifest> = OnceLock::new();
        MANIFEST.get_or_init(get_manifest)
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        let clock = self.clock.clone().unwrap_or_else(|| Arc::new(RealClock));
        let state = ReportsState::with_clock(ctx.clone(), clock);
        reports_router(state)
    }

    fn workers(&self, ctx: &ModuleContext) -> Vec<WorkerSpec> {
        let pool = ctx.pool().cloned();
        let clock = self.clock.clone().unwrap_or_else(|| Arc::new(RealClock));
        let event_bus = ctx.event_bus().clone();
        let config = ReportsWorkerConfig::default();

        vec![WorkerSpec::new("reports-worker", move |cancel_token| {
            let pool = pool.clone();
            let clock = clock.clone();
            let bus = event_bus.clone();
            Box::pin(async move {
                if let Some(p) = pool {
                    run_reports_worker(p, clock, cancel_token, bus, config).await;
                }
            }) as BoxFuture<'static, ()>
        })]
    }
}

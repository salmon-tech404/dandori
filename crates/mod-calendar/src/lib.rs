//! Module `calendar`: Lên lịch task định kỳ theo cron hoặc khoảng đều.

pub mod model;
pub mod routes;
pub mod schedule;
pub mod worker;

use std::sync::{Arc, LazyLock};

use axum::Router;
use dandori_module_api::{
    Manifest, Migration, Module, ModuleContext, ModuleSubscription, WorkerSpec,
};
use tokio_util::sync::CancellationToken;

use crate::routes::{CalendarState, create_routes};
use crate::worker::{CalendarWorkerConfig, Clock, RealClock, run_calendar_worker};

#[expect(
    clippy::expect_used,
    reason = "embedded static module.json validated at build/test time"
)]
static MANIFEST: LazyLock<Manifest> = LazyLock::new(|| {
    let json = include_str!("../module.json");
    serde_json::from_str(json).expect("module.json của mod-calendar phải hợp lệ")
});

static MIGRATIONS: [Migration; 1] = [Migration {
    id: "0001_calendar",
    sql: include_str!("../migrations/0001_calendar.sql"),
}];

/// Cấu trúc chính của module `calendar`.
#[derive(Clone, Debug)]
pub struct CalendarModule {
    clock: Arc<dyn Clock>,
}

impl Default for CalendarModule {
    fn default() -> Self {
        Self {
            clock: Arc::new(RealClock),
        }
    }
}

impl CalendarModule {
    /// Khởi tạo module với đồng hồ thời gian thực.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Khởi tạo module với đồng hồ tùy chỉnh phục vụ kiểm thử.
    #[must_use]
    pub fn with_clock(clock: Arc<dyn Clock>) -> Self {
        Self { clock }
    }
}

impl Module for CalendarModule {
    fn manifest(&self) -> &Manifest {
        &MANIFEST
    }

    fn migrations(&self) -> &'static [Migration] {
        &MIGRATIONS
    }

    fn routes(&self, context: &ModuleContext) -> Router {
        let state = CalendarState::with_clock(context.clone(), Arc::clone(&self.clock));
        create_routes(state)
    }

    fn workers(&self, ctx: &ModuleContext) -> Vec<WorkerSpec> {
        let pool = ctx.pool().cloned();
        let event_bus = Arc::clone(ctx.event_bus());
        let clock = Arc::clone(&self.clock);

        vec![WorkerSpec::new(
            "calendar_scheduler",
            move |cancel_token: CancellationToken| {
                let pool = pool.clone();
                let event_bus = Arc::clone(&event_bus);
                let clock = Arc::clone(&clock);
                Box::pin(async move {
                    if let Some(p) = pool {
                        run_calendar_worker(
                            p,
                            clock,
                            cancel_token,
                            event_bus,
                            CalendarWorkerConfig::default(),
                        )
                        .await;
                    }
                })
            },
        )]
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }
}

/// Factory function tạo module instance cho registry.
#[must_use]
pub fn create_module() -> Arc<dyn Module> {
    Arc::new(CalendarModule::new())
}

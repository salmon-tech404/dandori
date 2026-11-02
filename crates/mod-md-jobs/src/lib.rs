//! Module `md-jobs`: Tự động phát hiện và quản lý công việc định kỳ từ Markdown trong repo.

pub mod frontmatter;
pub mod model;
pub mod routes;
pub mod scanner;
pub mod worker;

use axum::Router;
use dandori_module_api::{
    Manifest, Migration, Module, ModuleContext, ModuleSubscription, WorkerSpec,
};
use std::sync::{Arc, LazyLock};
use tokio_util::sync::CancellationToken;

use crate::routes::{MdJobsState, create_routes};
use crate::worker::{Clock, MdJobsWorkerConfig, RealClock, run_md_jobs_worker};

#[expect(
    clippy::expect_used,
    reason = "embedded static module.json validated at build/test time"
)]
static MANIFEST: LazyLock<Manifest> = LazyLock::new(|| {
    let json = include_str!("../module.json");
    serde_json::from_str(json).expect("module.json của mod-md-jobs phải hợp lệ")
});

static MIGRATIONS: [Migration; 1] = [Migration {
    id: "0001_md_jobs",
    sql: include_str!("../migrations/0001_md_jobs.sql"),
}];

/// Cấu trúc chính của module `md-jobs`.
#[derive(Clone, Debug)]
pub struct MdJobsModule {
    clock: Arc<dyn Clock>,
}

impl Default for MdJobsModule {
    fn default() -> Self {
        Self {
            clock: Arc::new(RealClock),
        }
    }
}

impl MdJobsModule {
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

impl Module for MdJobsModule {
    fn manifest(&self) -> &Manifest {
        &MANIFEST
    }

    fn migrations(&self) -> &'static [Migration] {
        &MIGRATIONS
    }

    fn routes(&self, context: &ModuleContext) -> Router {
        let state = MdJobsState::with_clock(context.clone(), Arc::clone(&self.clock));
        create_routes(state)
    }

    fn workers(&self, ctx: &ModuleContext) -> Vec<WorkerSpec> {
        let pool = ctx.pool().cloned();
        let event_bus = Arc::clone(ctx.event_bus());
        let clock = Arc::clone(&self.clock);

        vec![WorkerSpec::new(
            "md_jobs_scheduler",
            move |cancel_token: CancellationToken| {
                let pool = pool.clone();
                let event_bus = Arc::clone(&event_bus);
                let clock = Arc::clone(&clock);
                Box::pin(async move {
                    if let Some(p) = pool {
                        run_md_jobs_worker(
                            p,
                            clock,
                            cancel_token,
                            event_bus,
                            MdJobsWorkerConfig::default(),
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
    Arc::new(MdJobsModule::new())
}

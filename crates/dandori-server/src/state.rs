//! Trạng thái dùng chung của máy chủ HTTP Dandori.

use crate::config::ServerConfig;
use crate::run_service::RunService;
use dandori_events::EventBus;
use dandori_registry::lifecycle::FeatureLifecycleManager;
use sqlx::SqlitePool;
use std::sync::Arc;

/// Trạng thái ứng dụng dùng chung giữa các handler của Axum.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Kết nối tới cơ sở dữ liệu SQLite lõi.
    pub db: SqlitePool,
    /// Bus sự kiện trong tiến trình.
    pub event_bus: Arc<EventBus>,
    /// Quản lý vòng đời thực thi các module.
    pub lifecycle_manager: Arc<FeatureLifecycleManager>,
    /// Cấu hình máy chủ.
    pub config: ServerConfig,
    /// Dịch vụ quản lý lần chạy của agent (RunService).
    pub run_service: Arc<RunService>,
}

impl AppState {
    /// Khởi tạo AppState mới với RunService mặc định.
    #[must_use]
    pub fn new(
        db: SqlitePool,
        event_bus: Arc<EventBus>,
        lifecycle_manager: Arc<FeatureLifecycleManager>,
        config: ServerConfig,
    ) -> Self {
        let run_service = Arc::new(RunService::new(
            db.clone(),
            Arc::clone(&event_bus),
            Arc::clone(&lifecycle_manager),
            crate::run_service::DEFAULT_MAX_CONCURRENT_RUNS,
        ));
        let runner: Arc<dyn dandori_module_api::TaskRunner> = run_service.clone();
        lifecycle_manager.set_task_runner(runner);
        Self {
            db,
            event_bus,
            lifecycle_manager,
            config,
            run_service,
        }
    }

    /// Khởi tạo AppState mới với RunService tùy biến.
    #[must_use]
    pub fn new_with_run_service(
        db: SqlitePool,
        event_bus: Arc<EventBus>,
        lifecycle_manager: Arc<FeatureLifecycleManager>,
        config: ServerConfig,
        run_service: Arc<RunService>,
    ) -> Self {
        let runner: Arc<dyn dandori_module_api::TaskRunner> = run_service.clone();
        lifecycle_manager.set_task_runner(runner);
        Self {
            db,
            event_bus,
            lifecycle_manager,
            config,
            run_service,
        }
    }
}

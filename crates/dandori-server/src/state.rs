//! Trạng thái dùng chung của máy chủ HTTP Dandori.

use crate::config::ServerConfig;
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
}

impl AppState {
    /// Khởi tạo AppState mới.
    #[must_use]
    pub fn new(
        db: SqlitePool,
        event_bus: Arc<EventBus>,
        lifecycle_manager: Arc<FeatureLifecycleManager>,
        config: ServerConfig,
    ) -> Self {
        Self {
            db,
            event_bus,
            lifecycle_manager,
            config,
        }
    }
}

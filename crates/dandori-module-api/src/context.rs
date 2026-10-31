//! Ngữ cảnh thực thi hẹp do hệ thống lõi cung cấp cho module (01_KIEN_TRUC.md mục 4.2).

use dandori_events::EventBus;
use sqlx::SqlitePool;
use std::sync::Arc;

/// Ngữ cảnh cung cấp cho module khi khởi tạo route, worker hoặc xử lý sự kiện.
///
/// Hợp đồng bảo mật:
/// Không lộ pool `SqlitePool` thô của các bảng lõi cho module, ngăn chặn vi phạm phân quyền dữ liệu.
#[derive(Clone, Debug)]
pub struct ModuleContext {
    module_id: String,
    event_bus: Arc<EventBus>,
    config: serde_json::Value,
    pool: Option<SqlitePool>,
}

impl ModuleContext {
    /// Khởi tạo một ModuleContext mới.
    pub fn new(
        module_id: impl Into<String>,
        event_bus: Arc<EventBus>,
        config: serde_json::Value,
    ) -> Self {
        Self {
            module_id: module_id.into(),
            event_bus,
            config,
            pool: None,
        }
    }

    /// Gán pool cơ sở dữ liệu để module tương tác với bảng riêng `m_<id>_*`.
    #[must_use]
    pub fn with_pool(mut self, pool: SqlitePool) -> Self {
        self.pool = Some(pool);
        self
    }

    /// Khởi tạo một ModuleContext phục vụ kiểm thử (sử dụng EventBus dung lượng 32).
    pub fn for_test(module_id: impl Into<String>) -> Self {
        Self {
            module_id: module_id.into(),
            event_bus: Arc::new(EventBus::new(32)),
            config: serde_json::json!({}),
            pool: None,
        }
    }

    /// Lấy ID của module sở hữu ngữ cảnh này.
    pub fn module_id(&self) -> &str {
        &self.module_id
    }

    /// Tham chiếu tới EventBus của hệ thống.
    pub fn event_bus(&self) -> &Arc<EventBus> {
        &self.event_bus
    }

    /// Lấy giá trị cấu hình hiệu lực của module.
    pub fn config(&self) -> &serde_json::Value {
        &self.config
    }

    /// Tham chiếu tới pool cơ sở dữ liệu nếu có.
    pub fn pool(&self) -> Option<&SqlitePool> {
        self.pool.as_ref()
    }
}

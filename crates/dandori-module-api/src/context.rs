//! Ngữ cảnh thực thi hẹp do hệ thống lõi cung cấp cho module (01_KIEN_TRUC.md mục 4.2).

use dandori_events::EventBus;
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;

use crate::extension::TaskRunner;

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
    data_dir: Option<PathBuf>,
    task_runner: Option<Arc<dyn TaskRunner>>,
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
            data_dir: None,
            task_runner: None,
        }
    }

    /// Gán pool cơ sở dữ liệu để module tương tác với bảng riêng `m_<id>_*`.
    #[must_use]
    pub fn with_pool(mut self, pool: SqlitePool) -> Self {
        self.pool = Some(pool);
        self
    }

    /// Thiết lập thư mục dữ liệu cho ngữ cảnh module.
    #[must_use]
    pub fn with_data_dir(mut self, data_dir: PathBuf) -> Self {
        self.data_dir = Some(data_dir);
        self
    }

    /// Gán bộ điều phối thực thi nhiệm vụ TaskRunner.
    #[must_use]
    pub fn with_task_runner(mut self, runner: Arc<dyn TaskRunner>) -> Self {
        self.task_runner = Some(runner);
        self
    }

    /// Lấy tham chiếu tới bộ điều phối thực thi nhiệm vụ nếu có.
    #[must_use]
    pub fn task_runner(&self) -> Option<&Arc<dyn TaskRunner>> {
        self.task_runner.as_ref()
    }

    /// Khởi tạo một ModuleContext phục vụ kiểm thử (sử dụng EventBus dung lượng 32).
    pub fn for_test(module_id: impl Into<String>) -> Self {
        Self {
            module_id: module_id.into(),
            event_bus: Arc::new(EventBus::new(32)),
            config: serde_json::json!({}),
            pool: None,
            data_dir: None,
            task_runner: None,
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

    /// Lấy thư mục dữ liệu dùng cho module.
    pub fn data_dir(&self) -> PathBuf {
        if let Some(ref dir) = self.data_dir {
            return dir.clone();
        }
        std::env::var("DANDORI_DATA_DIR")
            .map(PathBuf::from)
            .or_else(|_| {
                #[cfg(target_os = "windows")]
                {
                    std::env::var("APPDATA").map(|appdata| PathBuf::from(appdata).join("dandori"))
                }
                #[cfg(not(target_os = "windows"))]
                {
                    std::env::var("HOME")
                        .map(|home| PathBuf::from(home).join(".local/share/dandori"))
                }
            })
            .unwrap_or_else(|_| PathBuf::from("./data"))
    }
}

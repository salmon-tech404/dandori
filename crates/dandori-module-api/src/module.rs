//! Hợp đồng trait Module và các kiểu cấu thành (01_KIEN_TRUC.md mục 4.2).

use crate::context::ModuleContext;
use crate::manifest::Manifest;
use dandori_events::EventFilter;
use dandori_types::Envelope;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// Kiểu Future trả về cho các tác vụ bất đồng bộ của worker và handler sự kiện.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Khai báo migration SQL của module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    /// Định danh phiên bản migration (ví dụ "0001_initial").
    pub id: &'static str,
    /// Nội dung câu lệnh DDL SQL.
    pub sql: &'static str,
}

/// Khai báo một tiến trình nền (worker) do module quản lý.
pub struct WorkerSpec {
    /// Tên mô tả của worker.
    pub name: &'static str,
    /// Hàm thực thi worker nhận token hủy để dừng êm.
    pub run: Arc<dyn Fn(CancellationToken) -> BoxFuture<'static, ()> + Send + Sync>,
}

impl WorkerSpec {
    /// Khởi tạo một WorkerSpec mới.
    pub fn new<F>(name: &'static str, run: F) -> Self
    where
        F: Fn(CancellationToken) -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        Self {
            name,
            run: Arc::new(run),
        }
    }
}

impl fmt::Debug for WorkerSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkerSpec")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

/// Khai báo bộ lắng nghe sự kiện của module trên EventBus.
pub struct ModuleSubscription {
    /// Tên bộ lắng nghe sự kiện.
    pub name: &'static str,
    /// Bộ lọc sự kiện cần nhận.
    pub filter: EventFilter,
    /// Hàm xử lý sự kiện khi có sự kiện phù hợp.
    pub handler: Arc<dyn Fn(Envelope) -> BoxFuture<'static, ()> + Send + Sync>,
}

impl ModuleSubscription {
    /// Khởi tạo một ModuleSubscription mới.
    pub fn new<F>(name: &'static str, filter: EventFilter, handler: F) -> Self
    where
        F: Fn(Envelope) -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        Self {
            name,
            filter,
            handler: Arc::new(handler),
        }
    }
}

impl fmt::Debug for ModuleSubscription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModuleSubscription")
            .field("name", &self.name)
            .field("filter", &self.filter)
            .finish_non_exhaustive()
    }
}

/// Hợp đồng trait chính mà mọi module của Dandori phải triển khai (01_KIEN_TRUC.md mục 4.2).
pub trait Module: fmt::Debug + Send + Sync {
    /// Lấy manifest khai báo thông tin tĩnh của module.
    fn manifest(&self) -> &Manifest;

    /// Danh sách migration SQL tạo bảng riêng của module (`m_<id>_*`).
    fn migrations(&self) -> &'static [Migration];

    /// Khởi tạo các router HTTP của module dưới tiền tố `/api/v1/m/<id>/`.
    fn routes(&self, ctx: &ModuleContext) -> axum::Router;

    /// Danh sách worker chạy nền của module khi module được kích hoạt.
    fn workers(&self, ctx: &ModuleContext) -> Vec<WorkerSpec>;

    /// Danh sách bộ nghe sự kiện do module đăng ký.
    fn subscriptions(&self) -> Vec<ModuleSubscription>;
}

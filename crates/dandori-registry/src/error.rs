//! Các kiểu lỗi của Feature Registry (01_KIEN_TRUC.md mục 5 & 10).

use thiserror::Error;

/// Các lỗi phát sinh trong quá trình quản lý Feature Registry.
#[derive(Debug, Error)]
pub enum RegistryError {
    /// Cấm tắt module lõi của hệ thống.
    #[error("Module lõi '{0}' không thể bị vô hiệu hóa")]
    CoreModuleCannotBeDisabled(String),

    /// Cấp trên đã khóa cài đặt của module.
    #[error(
        "Module '{module_id}' đã bị khóa tại phạm vi '{locked_at_scope}', cấp dưới không thể thay đổi"
    )]
    ScopeLocked {
        /// Cấp phạm vi khóa cài đặt.
        locked_at_scope: String,
        /// ID của module bị khóa.
        module_id: String,
    },

    /// Người dùng không đủ quyền hạn trên phạm vi yêu cầu.
    #[error(
        "Không có quyền thực hiện: yêu cầu vai trò {required_role}, vai trò hiện tại là {actual_role}"
    )]
    PermissionDenied {
        /// Vai trò tối thiểu yêu cầu.
        required_role: String,
        /// Vai trò hiện tại của người dùng.
        actual_role: String,
    },

    /// Module không tồn tại trong danh mục.
    #[error("Không tìm thấy module: {0}")]
    ModuleNotFound(String),

    /// Gói mẫu tính năng không tồn tại.
    #[error("Không tìm thấy gói mẫu: {0}")]
    PackNotFound(String),

    /// Phát hiện chu trình phụ thuộc giữa các module.
    #[error("Phát hiện chu trình phụ thuộc: {0}")]
    DependencyCycle(String),

    /// Cần xác nhận rủi ro để bật module.
    #[error("Cần xác nhận rủi ro trước khi kích hoạt module '{0}'")]
    MissingRiskAcknowledgment(String),

    /// Không có lịch sử audit nào để hoàn tác.
    #[error("Không có lịch sử thay đổi để hoàn tác cho module: {0}")]
    NoAuditHistoryToUndo(String),

    /// Migration của module thất bại.
    #[error("Migration của module thất bại: {0}")]
    MigrationFailed(String),

    /// Không thể tắt module vì có module phụ thuộc đang hoạt động.
    #[error("Các module phụ thuộc đang hoạt động: {0:?}")]
    DependentModulesActive(Vec<String>),

    /// Lỗi cơ sở dữ liệu.
    #[error("Lỗi cơ sở dữ liệu: {0}")]
    Db(#[from] dandori_db::DbError),

    /// Lỗi truy vấn SQLx trực tiếp.
    #[error("Lỗi thao tác SQL: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// Lỗi tuần tự hóa JSON.
    #[error("Lỗi tuần tự hóa dữ liệu JSON: {0}")]
    Json(#[from] serde_json::Error),
}

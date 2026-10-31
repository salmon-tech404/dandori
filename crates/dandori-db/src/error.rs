//! Định nghĩa các loại lỗi cho crate dandori-db (RS-ERR-001).

/// Lỗi phát sinh trong các thao tác cơ sở dữ liệu SQLite của Dandori.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// Lỗi thực thi truy vấn SQLite từ sqlx.
    #[error("Lỗi SQLite: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// Lỗi áp dụng migration từ sqlx::migrate.
    #[error("Lỗi migration SQLite: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    /// Không tìm thấy bản ghi dữ liệu yêu cầu.
    #[error("Không tìm thấy thực thể {entity} với định danh {id}")]
    NotFound {
        /// Tên thực thể.
        entity: &'static str,
        /// Định danh thực thể.
        id: String,
    },

    /// Thao tác vi phạm quy tắc nghiệp vụ hoặc phạm vi sở hữu.
    #[error("Vi phạm ràng buộc dữ liệu hoặc phân quyền: {0}")]
    ConstraintViolation(String),
}

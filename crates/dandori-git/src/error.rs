//! Các kiểu lỗi phát sinh từ thao tác git.

use thiserror::Error;

/// Các loại lỗi có thể xảy ra khi thao tác với git repo và worktree.
#[derive(Debug, Error)]
pub enum GitError {
    /// Lệnh git trả về mã lỗi khác 0 kèm thông báo stderr.
    #[error("lệnh git thất bại (mã thoát {code}): {stderr}")]
    CommandFailed {
        /// Mã thoát của tiến trình git.
        code: i32,
        /// Dữ liệu lỗi in ra từ stderr.
        stderr: String,
    },

    /// Lỗi khi khởi chạy tiến trình qua `dandori-process`.
    #[error("lỗi tiến trình con: {0}")]
    Process(#[from] dandori_process::ProcessError),

    /// Kho git chưa có commit nào nên không thể tạo branch/worktree.
    #[error("kho git chưa có commit nào")]
    NoCommits,

    /// Tên nhánh không hợp lệ hoặc chứa ký tự bị cấm.
    #[error("tên nhánh không hợp lệ: {0}")]
    InvalidBranchName(String),

    /// Thư mục worktree có các thay đổi chưa được commit hoặc untracked.
    #[error("thư mục worktree có thay đổi chưa commit")]
    WorktreeDirty,

    /// Đường dẫn không hợp lệ hoặc nằm ngoài phạm vi cho phép.
    #[error("đường dẫn không hợp lệ hoặc nằm ngoài phạm vi cho phép: {0}")]
    InvalidPath(String),

    /// Lỗi I/O hệ thống tệp.
    #[error("lỗi I/O: {0}")]
    Io(#[from] std::io::Error),
}

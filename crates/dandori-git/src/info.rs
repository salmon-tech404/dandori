//! Thông tin chi tiết về worktree trong git.

use std::path::PathBuf;

/// Thông tin về một worktree trong repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeInfo {
    /// Đường dẫn tuyệt đối của worktree.
    pub path: PathBuf,
    /// Commit SHA đang được trỏ bởi HEAD của worktree.
    pub head: String,
    /// Tên nhánh gắn liền với worktree (nếu có).
    pub branch: Option<String>,
}

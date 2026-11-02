//! Cấu trúc dữ liệu và xử lý diff cho GitClient.

use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

use crate::error::GitError;

/// Thống kê dòng thay đổi của một tệp trong diff.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiffFileStat {
    /// Đường dẫn tệp mới (tương đối so với worktree).
    pub path: String,
    /// Đường dẫn tệp cũ nếu là đổi tên.
    pub old_path: Option<String>,
    /// Trạng thái thay đổi: "added", "modified", "deleted", "renamed", "binary".
    pub status: String,
    /// Số dòng thêm mới.
    pub insertions: u32,
    /// Số dòng bị xóa.
    pub deletions: u32,
    /// Có phải tệp nhị phân hay không.
    pub is_binary: bool,
}

/// Dữ liệu diff chi tiết của một tệp.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiffFile {
    /// Đường dẫn tệp mới (tương đối so với worktree).
    pub path: String,
    /// Đường dẫn tệp cũ nếu là đổi tên.
    pub old_path: Option<String>,
    /// Trạng thái thay đổi: "added", "modified", "deleted", "renamed".
    pub status: String,
    /// Số dòng thêm mới.
    pub insertions: u32,
    /// Số dòng bị xóa.
    pub deletions: u32,
    /// Có phải tệp nhị phân hay không.
    pub is_binary: bool,
    /// Nội dung unified diff.
    pub diff: String,
    /// Đánh dấu diff đã bị cắt bớt do vượt quá kích thước tối đa.
    pub truncated: bool,
}

/// Tổng hợp thống kê diff toàn bộ task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DiffStats {
    /// Tổng số dòng thêm mới.
    pub total_insertions: u32,
    /// Tổng số dòng bị xóa.
    pub total_deletions: u32,
    /// Tổng số tệp có thay đổi.
    pub total_files_changed: usize,
}

/// Kết quả diff hoàn chỉnh giữa worktree và nhánh cơ sở.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiffResult {
    /// Nhánh hoặc commit cơ sở so sánh (ví dụ "main").
    pub base_ref: String,
    /// Danh sách diff từng tệp.
    pub files: Vec<DiffFile>,
    /// Thống kê tổng quan.
    pub stats: DiffStats,
}

/// Kiểm tra và làm sạch đường dẫn tệp, bảo đảm nằm an toàn trong worktree.
///
/// # Errors
/// Trả về `GitError::InvalidRef` nếu đường dẫn chứa ký tự điều hướng hoặc trỏ ra ngoài.
pub fn validate_path_inside_worktree(
    worktree_path: &Path,
    rel_path: &str,
) -> Result<PathBuf, GitError> {
    if rel_path.is_empty() || rel_path.starts_with('/') || rel_path.starts_with('\\') {
        return Err(GitError::InvalidPath(format!(
            "Đường dẫn không được là đường dẫn tuyệt đối hoặc rỗng: {rel_path}"
        )));
    }

    let p = Path::new(rel_path);
    for component in p.components() {
        match component {
            Component::Normal(_) => {}
            _ => {
                return Err(GitError::InvalidPath(format!(
                    "Đường dẫn không được chứa ký tự điều hướng thư mục (..) hoặc tiền tố ổ đĩa: {rel_path}"
                )));
            }
        }
    }

    Ok(worktree_path.join(p))
}

/// Phân tích một dòng đầu ra từ `git diff --numstat`.
#[must_use]
pub fn parse_numstat_line(line: &str) -> Option<(String, Option<String>, u32, u32, bool)> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 3 {
        return None;
    }

    let ins_str = parts.first()?;
    let del_str = parts.get(1)?;
    let raw_path = parts.get(2)?;

    let is_binary = *ins_str == "-" && *del_str == "-";
    let insertions = if is_binary {
        0
    } else {
        ins_str.parse::<u32>().unwrap_or(0)
    };
    let deletions = if is_binary {
        0
    } else {
        del_str.parse::<u32>().unwrap_or(0)
    };

    let (path, old_path) = parse_rename_path(raw_path);
    Some((path, old_path, insertions, deletions, is_binary))
}

/// Phân tích chuỗi đường dẫn có thể chứa ký hiệu đổi tên của Git (`old => new`).
#[must_use]
pub fn parse_rename_path(raw: &str) -> (String, Option<String>) {
    if let Some((old, new)) = raw.split_once(" => ") {
        // Xử lý dạng: {dir1 => dir2}/file hoặc dir/{old => new}.ext
        if let (Some(brace_start), Some(brace_end)) = (raw.find('{'), raw.find('}')) {
            let prefix = raw.get(..brace_start).unwrap_or("");
            let suffix = raw.get(brace_end + 1..).unwrap_or("");
            let inner = raw.get(brace_start + 1..brace_end).unwrap_or("");
            if let Some((in_old, in_new)) = inner.split_once(" => ") {
                let full_old = format!("{prefix}{in_old}{suffix}");
                let full_new = format!("{prefix}{in_new}{suffix}");
                return (full_new, Some(full_old));
            }
        }
        (new.to_string(), Some(old.to_string()))
    } else {
        (raw.to_string(), None)
    }
}

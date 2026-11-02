//! Tiện ích phát hiện sự tồn tại và thăm dò trạng thái của các CLI agent.

use std::path::{Path, PathBuf};

/// Kiểm tra chương trình thực thi có tồn tại trên PATH hoặc tại đường dẫn trực tiếp hay không.
#[must_use]
pub fn check_binary_exists(program_name: &str) -> bool {
    check_binary_exists_with_path(program_name, std::env::var("PATH").ok().as_deref())
}

/// Kiểm tra chương trình thực thi có tồn tại với chuỗi PATH tùy biến.
#[must_use]
pub fn check_binary_exists_with_path(program_name: &str, custom_path: Option<&str>) -> bool {
    let direct_path = Path::new(program_name);
    if direct_path.is_file() {
        return true;
    }

    let path_val = match custom_path {
        Some(p) => p,
        None => return false,
    };

    let separator = if cfg!(windows) { ';' } else { ':' };
    let extensions = if cfg!(windows) {
        vec!["", ".exe", ".cmd", ".bat", ".ps1"]
    } else {
        vec![""]
    };

    for dir in path_val.split(separator) {
        let trimmed = dir.trim();
        if trimmed.is_empty() {
            continue;
        }
        let dir_path = Path::new(trimmed);
        for ext in &extensions {
            let candidate = dir_path.join(format!("{program_name}{ext}"));
            if candidate.is_file() {
                return true;
            }
        }
    }

    false
}

/// Tìm đường dẫn thực thi đầy đủ của binary nếu có trên PATH.
#[must_use]
pub fn resolve_binary_path(program_name: &str) -> Option<PathBuf> {
    let direct = Path::new(program_name);
    if direct.is_file() {
        return Some(direct.to_path_buf());
    }

    let path_var = std::env::var("PATH").ok()?;
    let separator = if cfg!(windows) { ';' } else { ':' };
    let extensions = if cfg!(windows) {
        vec!["", ".exe", ".cmd", ".bat", ".ps1"]
    } else {
        vec![""]
    };

    for dir in path_var.split(separator) {
        let trimmed = dir.trim();
        if trimmed.is_empty() {
            continue;
        }
        let dir_path = Path::new(trimmed);
        for ext in &extensions {
            let candidate = dir_path.join(format!("{program_name}{ext}"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

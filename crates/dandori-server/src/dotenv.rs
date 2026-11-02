//! Đọc tệp `.env` tối giản (không thêm thư viện ngoài): chỉ nhận dòng `KEY=VALUE`.
//!
//! Chỉ các khóa `DANDORI_*` và `RUST_LOG` được nhận để tệp này không thể ghi đè biến hệ thống như `PATH`.
//! Biến môi trường thật luôn được ưu tiên hơn giá trị trong tệp.

use std::collections::HashMap;
use std::path::Path;

/// Kích thước tối đa của tệp `.env` (64 KiB).
const MAX_ENV_FILE_BYTES: u64 = 64 * 1024;

/// Phân tích nội dung `.env` thành bảng khóa-giá trị (chỉ khóa được phép).
#[must_use]
pub fn parse_env_text(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if !(key.starts_with("DANDORI_") || key == "RUST_LOG") {
            continue;
        }
        let value = value.trim();
        let unquoted = value
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
            .unwrap_or(value);
        map.insert(key.to_string(), unquoted.to_string());
    }
    map
}

/// Đọc tệp `.env` tại `path`; trả về bảng rỗng nếu không có, quá lớn hoặc không đọc được.
#[must_use]
pub fn load_env_file(path: &Path) -> HashMap<String, String> {
    let Ok(meta) = std::fs::metadata(path) else {
        return HashMap::new();
    };
    if !meta.is_file() || meta.len() > MAX_ENV_FILE_BYTES {
        return HashMap::new();
    }
    std::fs::read_to_string(path)
        .map(|t| parse_env_text(&t))
        .unwrap_or_default()
}

//! Công tắc chính sách mạng dùng chung (chỉ phục vụ kiểm thử).

use std::sync::atomic::{AtomicBool, Ordering};

static ALLOW_LOCAL_ENDPOINTS: AtomicBool = AtomicBool::new(false);

/// Bật/tắt việc cho phép điểm cuối cục bộ (127.0.0.1, http) cho các tích hợp ngoài.
///
/// **Chỉ dành cho kiểm thử** với máy chủ giả. Mã chạy thật không bao giờ gọi hàm này và mặc định là tắt;
/// không có đường HTTP hay biến môi trường nào bật được nó (việc dò biến môi trường từng khiến `cargo run`
/// bật nhầm chế độ kiểm thử).
#[doc(hidden)]
pub fn set_allow_local_endpoints_for_tests(allow: bool) {
    ALLOW_LOCAL_ENDPOINTS.store(allow, Ordering::SeqCst);
}

/// Hiện có cho phép điểm cuối cục bộ không (mặc định `false`).
#[must_use]
pub fn allow_local_endpoints() -> bool {
    ALLOW_LOCAL_ENDPOINTS.load(Ordering::SeqCst)
}

/// Tách phần tên máy (không scheme, không đường dẫn, không cổng) từ một Origin/Host/mục cấu hình, viết thường.
#[must_use]
pub fn origin_host(raw: &str) -> String {
    let lowered = raw.trim().to_ascii_lowercase();
    let t = lowered.as_str();
    let no_scheme = t
        .strip_prefix("https://")
        .or_else(|| t.strip_prefix("http://"))
        .unwrap_or(t);
    let authority = no_scheme.split('/').next().unwrap_or(no_scheme);
    let host = if let Some(rest) = authority.strip_prefix('[') {
        // IPv6 dạng [::1]:3000
        rest.split(']').next().unwrap_or(rest)
    } else {
        authority.split(':').next().unwrap_or(authority)
    };
    host.trim().to_ascii_lowercase()
}

/// Một mục trong danh sách Origin cho phép có hợp lệ không: tên máy/IP, hoặc `*.miền.đuôi` với ít nhất hai nhãn
/// sau dấu `*.` (cấm `*`, `*.com`, `*.` vì sẽ mở cho cả Internet).
#[must_use]
pub fn is_valid_allowed_origin_entry(raw: &str) -> bool {
    let host = origin_host(raw);
    if host.is_empty() || host.len() > 253 {
        return false;
    }
    let (wild, body) = match host.strip_prefix("*.") {
        Some(rest) => (true, rest),
        None => (false, host.as_str()),
    };
    if wild && body.split('.').count() < 2 {
        return false;
    }
    body.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == ':')
    })
}

/// So khớp tên máy của yêu cầu với một mục cho phép. Khớp đúng tên, hoặc với `*.miền` thì phải là tên miền con
/// thật (có dấu chấm ngăn cách), nên `*.example.com` KHÔNG khớp `evilexample.com`.
#[must_use]
pub fn origin_host_matches(request_host: &str, allowed_entry: &str) -> bool {
    if !is_valid_allowed_origin_entry(allowed_entry) {
        return false;
    }
    let host = origin_host(request_host);
    let allowed = origin_host(allowed_entry);
    match allowed.strip_prefix("*.") {
        Some(suffix) => host == suffix || host.ends_with(&format!(".{suffix}")),
        None => host == allowed,
    }
}

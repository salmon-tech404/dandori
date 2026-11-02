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

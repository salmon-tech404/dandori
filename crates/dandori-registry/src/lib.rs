//! Feature Registry quản lý trạng thái, vòng đời và tính hợp lệ của các module.

/// Khởi tạo module registry.
#[must_use]
pub fn init() -> &'static str {
    "dandori-registry initialized"
}

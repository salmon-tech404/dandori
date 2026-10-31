//! Quản lý tiến trình con an toàn của Dandori.

#![warn(missing_docs)]

/// Khởi tạo module tiến trình con.
#[must_use]
pub fn init() -> &'static str {
    "dandori-process initialized"
}

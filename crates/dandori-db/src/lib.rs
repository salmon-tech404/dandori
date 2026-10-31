//! Cơ sở dữ liệu SQLite và migrations lõi của Dandori.

#![warn(missing_docs)]

/// Khởi tạo module cơ sở dữ liệu.
#[must_use]
pub fn init() -> &'static str {
    "dandori-db initialized"
}

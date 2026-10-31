//! Kiểu dữ liệu dùng chung và định danh newtype của Dandori.

#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Phiên bản của crate types.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Trạng thái hoạt động tổng thể của hệ thống Dandori.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SystemStatus {
    /// Hệ thống đang khởi tạo các dịch vụ nền
    Initializing,
    /// Hệ thống sẵn sàng tiếp nhận điều phối agent
    Ready,
    /// Hệ thống đang hoạt động ở chế độ suy giảm hiệu năng
    Degraded,
    /// Hệ thống đang dừng hoặc bảo trì
    Stopped,
}

/// Sinh toàn bộ định nghĩa TypeScript từ các kiểu dữ liệu của Dandori.
#[must_use]
pub fn generate_typescript_types() -> String {
    let mut out = String::new();
    out.push_str("// TỆP SINH TỰ ĐỘNG bởi generate_types (RS-TS-001); không sửa tay.\n\n");
    let decl = SystemStatus::decl();
    if decl.starts_with("type ") || decl.starts_with("interface ") {
        out.push_str("export ");
        out.push_str(&decl);
    } else if decl.starts_with("export ") {
        out.push_str(&decl);
    } else {
        out.push_str("export type SystemStatus = ");
        out.push_str(&decl);
    }
    out.push('\n');
    out
}

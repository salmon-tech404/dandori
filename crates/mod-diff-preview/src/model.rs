//! Các cấu trúc dữ liệu cho module `diff-preview`.

use serde::{Deserialize, Serialize};

/// Tham số truy vấn khi yêu cầu diff.
#[derive(Debug, Deserialize, Default)]
pub struct DiffQuery {
    /// Nhánh hoặc commit cơ sở so sánh (nếu bỏ trống sẽ dùng default_branch của dự án).
    pub base_ref: Option<String>,
}

/// Yêu cầu khởi chạy phiên xem trước của task.
#[derive(Debug, Deserialize)]
pub struct StartPreviewRequest {
    /// Bắt buộc xác nhận trước khi khởi chạy lệnh.
    pub confirmed: bool,
    /// Mảng tham số dòng lệnh tùy chỉnh (nếu cấu hình dự án chưa có hoặc muốn ghi đè).
    pub command: Option<Vec<String>>,
    /// Cổng TCP tùy chỉnh trên 127.0.0.1.
    pub port: Option<u16>,
}

/// Trạng thái phiên xem trước đang chạy của task.
#[derive(Debug, Serialize, Clone)]
pub struct PreviewStatus {
    /// Phiên xem trước có đang chạy hay không.
    pub running: bool,
    /// PID tiến trình nếu đang chạy.
    pub pid: Option<u32>,
    /// Cổng lắng nghe.
    pub port: Option<u16>,
    /// Địa chỉ host (luôn là 127.0.0.1).
    pub host: String,
    /// URL hoàn chỉnh để truy cập bản xem trước (ví dụ http://127.0.0.1:3000).
    pub url: Option<String>,
    /// Lệnh đã/đang chạy dưới dạng mảng tham số.
    pub command: Vec<String>,
    /// Nhật ký gần đây của tiến trình xem trước.
    pub logs: Vec<String>,
}

/// Kiểm tra cổng xem trước hợp lệ (từ 1024 đến 65535).
///
/// # Errors
/// Trả về chuỗi lỗi nếu cổng nhỏ hơn 1024 hoặc lớn hơn 65535.
pub fn validate_preview_port(port: u16) -> Result<(), &'static str> {
    if port < 1024 {
        Err("Cổng xem trước phải từ 1024 trở lên (không được dùng cổng đặc quyền)")
    } else {
        Ok(())
    }
}

/// Kiểm tra địa chỉ host xem trước (chỉ chấp nhận loopback 127.0.0.1 hoặc localhost).
///
/// # Errors
/// Trả về chuỗi lỗi nếu host không phải loopback nội bộ.
pub fn validate_preview_host(host: &str) -> Result<(), &'static str> {
    if host == "127.0.0.1" || host == "localhost" {
        Ok(())
    } else {
        Err("Địa chỉ xem trước chỉ được phép là loopback 127.0.0.1")
    }
}

/// Chương trình bị cấm làm lệnh xem trước: trình thông dịch lệnh/shell và máy chủ script. Lệnh xem trước phải
/// là mảng tham số chạy thẳng chương trình (RS-SEC-001), không qua shell.
const FORBIDDEN_PREVIEW_PROGRAMS: [&str; 14] = [
    "cmd",
    "powershell",
    "pwsh",
    "sh",
    "bash",
    "zsh",
    "fish",
    "dash",
    "ksh",
    "wscript",
    "cscript",
    "mshta",
    "wsl",
    "env",
];

/// Kiểm lệnh xem trước: không rỗng, chương trình là tên trần (không đường dẫn), không phải shell.
///
/// # Errors
/// Trả về thông điệp lỗi nếu lệnh không hợp lệ.
pub fn validate_preview_command(command: &[String]) -> Result<(), &'static str> {
    let Some(program) = command.first().map(String::as_str) else {
        return Err("Lệnh xem trước rỗng");
    };
    if program.is_empty() || program.contains(['/', '\\']) || program.contains(char::is_whitespace)
    {
        return Err(
            "Chương trình xem trước phải là tên chương trình, không phải đường dẫn hay chuỗi lệnh",
        );
    }
    let stem = program
        .rsplit_once('.')
        .map_or(program, |(name, _ext)| name)
        .to_ascii_lowercase();
    if FORBIDDEN_PREVIEW_PROGRAMS.contains(&stem.as_str()) {
        return Err("Không được dùng shell hay trình thông dịch lệnh làm lệnh xem trước");
    }
    Ok(())
}

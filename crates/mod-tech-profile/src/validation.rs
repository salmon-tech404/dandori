//! Kiểm tra tính hợp lệ và an toàn của các mục hồ sơ kỹ thuật (P1-08).

/// Độ dài tối đa cho phép của một giá trị hồ sơ kỹ thuật.
pub const MAX_VALUE_LENGTH: usize = 2_000;

/// Độ dài tối đa của khối prompt tổng thể sinh từ hồ sơ kỹ thuật.
pub const MAX_PROFILE_PROMPT_LENGTH: usize = 4_000;

/// Dấu hiệu thông báo nội dung đã bị cắt do vượt quá giới hạn độ dài.
pub const TRUNCATION_MARKER: &str = "\n... [Cắt do vượt quá độ dài tối đa]";

/// Kiểm tra xem nội dung có chứa mẫu giống khóa bí mật hoặc token hay không (RS-SEC-005, AC4).
/// Dùng chung bộ nhận diện của `dandori-module-api` (nhận theo đầu từ, không khớp nhầm "task-").
pub fn check_secrets(content: &str) -> Result<(), &'static str> {
    if dandori_module_api::contains_secret_like(content) {
        return Err("Nội dung chứa chuỗi giống khóa bí mật hoặc API token bị từ chối");
    }
    Ok(())
}

/// Kiểm tra độ dài giá trị của một mục hồ sơ.
pub fn check_value_length(value: &str) -> Result<(), &'static str> {
    if value.len() > MAX_VALUE_LENGTH {
        return Err("Giá trị mục hồ sơ kỹ thuật vượt quá giới hạn 2,000 ký tự");
    }
    Ok(())
}

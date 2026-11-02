//! HTTP Client an toàn phục vụ các kết nối tích hợp ngoài (SEC-SSRF-001, ADR 0006).

use reqwest::redirect::Policy;
use reqwest::{Client, Response};
use std::time::Duration;
use url::Url;

use crate::ssrf::{SsrfError, resolve_and_check, validate_redirect_target};

/// Thời gian chờ mặc định cho các yêu cầu HTTP tích hợp (15 giây).
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);
/// Giới hạn kích thước phản hồi tối đa (2MB) để tránh quá tải bộ nhớ.
pub const MAX_RESPONSE_SIZE: usize = 2 * 1024 * 1024;

/// Xây dựng một `reqwest::Client` an toàn tuân thủ chính sách SSRF.
///
/// Phân giải tên miền ngay lúc này, kiểm mọi IP nhận được, rồi **ghim** kết nối vào đúng các IP đó (chống
/// DNS rebinding giữa lúc kiểm và lúc gọi). Chuyển hướng chỉ được phép trong cùng tên miền.
///
/// # Errors
/// Trả về [`SsrfError`] nếu không phân giải được, có IP bị cấm, hoặc không tạo được client.
pub async fn build_safe_client(base_url: &Url, allow_private: bool) -> Result<Client, SsrfError> {
    let addrs = resolve_and_check(base_url, allow_private).await?;
    let base_clone = base_url.clone();

    let redirect_policy = Policy::custom(move |attempt| {
        if attempt.previous().len() >= 5 {
            return attempt.stop();
        }
        let target = attempt.url();
        if let Err(e) = validate_redirect_target(&base_clone, target, allow_private) {
            tracing::warn!("Chuyển hướng HTTP bị chặn: {:?}", e);
            return attempt.stop();
        }
        attempt.follow()
    });

    let mut builder = Client::builder()
        .timeout(DEFAULT_TIMEOUT)
        .redirect(redirect_policy);
    if let Some(host) = base_url.host_str() {
        builder = builder.resolve_to_addrs(host, &addrs);
    }
    builder
        .build()
        .map_err(|e| SsrfError::InvalidUrl(e.to_string()))
}

/// Đọc nội dung phản hồi an toàn có giới hạn kích thước tối đa.
///
/// # Errors
/// Trả về lỗi dạng chuỗi nếu đọc thất bại, vượt giới hạn hoặc không phải UTF-8.
pub async fn read_limited_text(response: Response, max_bytes: usize) -> Result<String, String> {
    if response
        .content_length()
        .is_some_and(|len| len > max_bytes as u64)
    {
        return Err(format!(
            "Kích thước phản hồi vượt quá giới hạn {max_bytes} bytes"
        ));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Lỗi đọc nội dung phản hồi: {e}"))?;

    if bytes.len() > max_bytes {
        return Err(format!(
            "Kích thước phản hồi ({} bytes) vượt quá giới hạn {} bytes",
            bytes.len(),
            max_bytes
        ));
    }

    String::from_utf8(bytes.to_vec())
        .map_err(|e| format!("Nội dung phản hồi không phải UTF-8 hợp lệ: {e}"))
}

/// Che giấu token nhạy cảm khỏi thông điệp lỗi hoặc URL.
#[must_use]
pub fn sanitize_error_message(msg: &str, token: &str) -> String {
    if token.is_empty() {
        return msg.to_string();
    }
    msg.replace(token, "[REDACTED_TOKEN]")
}

//! Thực thi gửi thông báo tới các nền tảng Telegram, Discord và Slack (P3-12).
//!
//! Quy chuẩn trích dẫn tài liệu chính thức (Điều cấm 11):
//! - Telegram Bot API `sendMessage`: <https://core.telegram.org/bots/api#sendmessage>
//!   Tham số: `chat_id` (String/i64), `text` (String <= 4096 bytes), `parse_mode` ("MarkdownV2").
//! - Discord Webhook `execute-webhook`: <https://discord.com/developers/docs/resources/webhook#execute-webhook>
//!   Tham số: `content` (String <= 2000 characters).
//! - Slack Incoming Webhook: <https://api.slack.com/messaging/webhooks>
//!   Tham số: `text` (String, mrkdwn formatting).

use std::time::Duration;
use url::Url;

use dandori_module_api::safe_http::{
    SsrfError, build_safe_client_with_timeout, local_endpoint_allowed, sanitize_error_message,
    validate_domain_in_allowlist,
};
use dandori_module_api::secrets::redact_secrets;

use crate::model::ChannelType;

/// Thời gian chờ mặc định khi gửi thông báo (10 giây).
pub const NOTIFICATION_TIMEOUT: Duration = Duration::from_secs(10);

/// Lỗi phát sinh trong quá trình gửi thông báo.
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    /// Lỗi định dạng URL hoặc bảo mật SSRF.
    #[error("Lỗi SSRF hoặc URL không hợp lệ: {0}")]
    Ssrf(#[from] SsrfError),

    /// Lỗi cấu hình kênh (thiếu chat_id hoặc token).
    #[error("Cấu hình kênh không hợp lệ: {0}")]
    Config(String),

    /// Lỗi kết nối mạng hoặc gọi HTTP.
    #[error("Lỗi HTTP: {0}")]
    Http(String),

    /// Phản hồi từ nền tảng trả về mã lỗi.
    #[error("Nền tảng trả về lỗi (mã {0}): {1}")]
    PlatformError(u16, String),
}

/// Chỉ công tắc kiểm thử tường minh mới được nới chính sách mạng; mã chạy thật luôn chặn IP nội bộ/loopback.
fn test_local_allowed() -> bool {
    dandori_module_api::allow_local_endpoints()
}

/// Thoát ký tự đặc biệt cho Telegram MarkdownV2.
/// Ký tự cần thoát: `_`, `*`, `[`, `]`, `(`, `)`, `~`, `>`, `#`, `+`, `-`, `=`, `|`, `{`, `}`, `.`, `!`
#[must_use]
pub fn escape_telegram_markdown(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len() * 2);
    for ch in text.chars() {
        if matches!(
            ch,
            '_' | '*'
                | '['
                | ']'
                | '('
                | ')'
                | '~'
                | '>'
                | '#'
                | '+'
                | '-'
                | '='
                | '|'
                | '{'
                | '}'
                | '.'
                | '!'
                | '`'
                | '\\'
        ) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

/// Làm sạch và chuẩn hóa nội dung thông báo theo chính sách an toàn:
/// Chỉ gồm tiêu đề, trạng thái và liên kết; không chứa mã lệnh, diff, đường dẫn tuyệt đối hay bí mật.
#[must_use]
pub fn sanitize_notification_content(
    title: &str,
    status: &str,
    link: Option<&str>,
) -> (String, String, Option<String>) {
    // 1. Che giấu bí mật nếu có vô tình xuất hiện
    let clean_title = redact_secrets(title);
    let clean_status = redact_secrets(status);
    let clean_link = link.map(redact_secrets);

    // 2. Lọc bỏ các dấu hiệu của đường dẫn tuyệt đối (C:\, /etc, /home)
    let strip_paths = |s: &str| -> String {
        let lines: Vec<&str> = s
            .lines()
            .filter(|l| {
                let trimmed = l.trim();
                !trimmed.starts_with("+++")
                    && !trimmed.starts_with("---")
                    && !trimmed.starts_with("@@")
                    && !trimmed.contains("PRIVATE KEY")
            })
            .collect();
        lines.join(" ")
    };

    (
        strip_paths(&clean_title),
        strip_paths(&clean_status),
        clean_link,
    )
}

/// Gửi thông báo tới kênh Telegram.
///
/// # Errors
/// Trả về [`NotificationError`] nếu cấu hình sai, vi phạm SSRF hoặc gọi HTTP thất bại.
pub async fn send_telegram_notification(
    token: &str,
    chat_id: Option<&str>,
    title: &str,
    status: &str,
    link: Option<&str>,
    override_base_url: Option<&str>,
) -> Result<(), NotificationError> {
    let Some(chat_id) = chat_id.filter(|s| !s.is_empty()) else {
        return Err(NotificationError::Config(
            "Telegram yêu cầu chat_id".to_string(),
        ));
    };

    let base_str = override_base_url.unwrap_or("https://api.telegram.org");
    let base_url = Url::parse(base_str)
        .map_err(|e| NotificationError::Ssrf(SsrfError::InvalidUrl(e.to_string())))?;

    let host = base_url.host_str().unwrap_or_default();
    if !local_endpoint_allowed(host, test_local_allowed()) {
        validate_domain_in_allowlist(host, "telegram", &["api.telegram.org"], false)?;
    }

    let (title, status, link) = sanitize_notification_content(title, status, link);
    let mut message = format!(
        "*{}*\nTrạng thái: {}",
        escape_telegram_markdown(&title),
        escape_telegram_markdown(&status)
    );
    if let Some(l) = link {
        message.push_str(&format!("\nLiên kết: {}", escape_telegram_markdown(&l)));
    }

    if message.chars().count() > 4000 {
        message = message.chars().take(4000).collect();
    }

    let client =
        build_safe_client_with_timeout(&base_url, test_local_allowed(), NOTIFICATION_TIMEOUT)
            .await?;
    let endpoint = format!("{base_str}/bot{token}/sendMessage");

    let payload = serde_json::json!({
        "chat_id": chat_id,
        "text": message,
        "parse_mode": "MarkdownV2"
    });

    let res = client
        .post(&endpoint)
        .json(&payload)
        .send()
        .await
        .map_err(|e| NotificationError::Http(sanitize_error_message(&e.to_string(), token)))?;

    if !res.status().is_success() {
        let code = res.status().as_u16();
        let body = res.text().await.unwrap_or_default();
        return Err(NotificationError::PlatformError(
            code,
            sanitize_error_message(&body, token),
        ));
    }

    Ok(())
}

/// Gửi thông báo tới kênh Discord qua Webhook.
///
/// # Errors
/// Trả về [`NotificationError`] nếu vi phạm SSRF hoặc gọi HTTP thất bại.
pub async fn send_discord_notification(
    webhook_url: &str,
    title: &str,
    status: &str,
    link: Option<&str>,
) -> Result<(), NotificationError> {
    let url = Url::parse(webhook_url)
        .map_err(|e| NotificationError::Ssrf(SsrfError::InvalidUrl(e.to_string())))?;

    let host = url.host_str().unwrap_or_default();
    if !local_endpoint_allowed(host, test_local_allowed()) {
        validate_domain_in_allowlist(host, "discord", &["discord.com"], false)?;
    }

    let (title, status, link) = sanitize_notification_content(title, status, link);
    let mut content = format!("**{title}**\nTrạng thái: `{status}`");
    if let Some(l) = link {
        content.push_str(&format!("\nChi tiết: <{l}>"));
    }

    if content.chars().count() > 1900 {
        content = content.chars().take(1900).collect();
    }

    let client =
        build_safe_client_with_timeout(&url, test_local_allowed(), NOTIFICATION_TIMEOUT).await?;
    // `allowed_mentions.parse = []`: tiêu đề task do người dùng nhập không được kích hoạt @everyone/@here/@role.
    let payload = serde_json::json!({
        "content": content,
        "allowed_mentions": { "parse": [] }
    });

    let res = client
        .post(webhook_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            NotificationError::Http(sanitize_error_message(&e.to_string(), webhook_url))
        })?;

    if !res.status().is_success() {
        let code = res.status().as_u16();
        let body = res.text().await.unwrap_or_default();
        return Err(NotificationError::PlatformError(
            code,
            sanitize_error_message(&body, webhook_url),
        ));
    }

    Ok(())
}

/// Gửi thông báo tới kênh Slack qua Incoming Webhook.
///
/// # Errors
/// Trả về [`NotificationError`] nếu vi phạm SSRF hoặc gọi HTTP thất bại.
pub async fn send_slack_notification(
    webhook_url: &str,
    title: &str,
    status: &str,
    link: Option<&str>,
) -> Result<(), NotificationError> {
    let url = Url::parse(webhook_url)
        .map_err(|e| NotificationError::Ssrf(SsrfError::InvalidUrl(e.to_string())))?;

    let host = url.host_str().unwrap_or_default();
    if !local_endpoint_allowed(host, test_local_allowed()) {
        validate_domain_in_allowlist(host, "slack", &["hooks.slack.com"], false)?;
    }

    let (title, status, link) = sanitize_notification_content(title, status, link);
    let mut text = format!("*{title}*\nTrạng thái: `{status}`");
    if let Some(l) = link {
        text.push_str(&format!("\nChi tiết: <{l}>"));
    }

    let client =
        build_safe_client_with_timeout(&url, test_local_allowed(), NOTIFICATION_TIMEOUT).await?;
    let payload = serde_json::json!({ "text": text });

    let res = client
        .post(webhook_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            NotificationError::Http(sanitize_error_message(&e.to_string(), webhook_url))
        })?;

    if !res.status().is_success() {
        let code = res.status().as_u16();
        let body = res.text().await.unwrap_or_default();
        return Err(NotificationError::PlatformError(
            code,
            sanitize_error_message(&body, webhook_url),
        ));
    }

    Ok(())
}

/// Dispatcher gửi thông báo tuỳ theo loại kênh.
///
/// # Errors
/// Trả về [`NotificationError`] nếu xảy ra lỗi trong quá trình gửi.
pub async fn dispatch_notification(
    channel_type: ChannelType,
    secret_value: &str,
    chat_id: Option<&str>,
    title: &str,
    status: &str,
    link: Option<&str>,
    override_base_url: Option<&str>,
) -> Result<(), NotificationError> {
    match channel_type {
        ChannelType::Telegram => {
            send_telegram_notification(
                secret_value,
                chat_id,
                title,
                status,
                link,
                override_base_url,
            )
            .await
        }
        ChannelType::Discord => {
            let url = override_base_url.unwrap_or(secret_value);
            send_discord_notification(url, title, status, link).await
        }
        ChannelType::Slack => {
            let url = override_base_url.unwrap_or(secret_value);
            send_slack_notification(url, title, status, link).await
        }
    }
}

//! Định nghĩa các cấu trúc dữ liệu cho kênh thông báo và lịch sử gửi (P3-12).

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Loại kênh thông báo được hỗ trợ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    /// Telegram Bot qua Telegram Bot API (<https://core.telegram.org/bots/api#sendmessage>)
    Telegram,
    /// Discord Webhook (<https://discord.com/developers/docs/resources/webhook#execute-webhook>)
    Discord,
    /// Slack Incoming Webhook (<https://api.slack.com/messaging/webhooks>)
    Slack,
}

impl ChannelType {
    /// Chuyển đổi sang chuỗi định danh.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Telegram => "telegram",
            Self::Discord => "discord",
            Self::Slack => "slack",
        }
    }

    /// Phân tích từ chuỗi định danh.
    #[must_use]
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "telegram" => Some(Self::Telegram),
            "discord" => Some(Self::Discord),
            "slack" => Some(Self::Slack),
            _ => None,
        }
    }
}

/// Bản ghi cấu hình kênh thông báo trong cơ sở dữ liệu.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct NotificationChannel {
    /// ID định danh kênh.
    pub id: String,
    /// ID workspace quản lý.
    pub workspace_id: String,
    /// Tên gợi nhớ của kênh.
    pub name: String,
    /// Loại kênh (telegram, discord, slack).
    pub channel_type: String,
    /// ID bí mật trong kho khóa (`vault:<id>`). Không bao giờ trả về API rõ ràng.
    pub vault_secret_id: String,
    /// Chat ID nếu là Telegram.
    pub chat_id: Option<String>,
    /// Trạng thái bật/tắt của kênh.
    pub is_enabled: bool,
    /// Nhận thông báo khi task cần người xử lý (needs_human).
    pub notify_on_needs_human: bool,
    /// Nhận thông báo khi task thất bại (failed).
    pub notify_on_failed: bool,
    /// Nhận thông báo khi có yêu cầu phê duyệt (approval_requested).
    pub notify_on_approval_requested: bool,
    /// Nhận thông báo cảnh báo hạn mức (limit_warning).
    pub notify_on_limit_warning: bool,
    /// Thời điểm tạo.
    pub created_at: String,
    /// Thời điểm cập nhật cuối.
    pub updated_at: String,
}

/// Phiên bản công khai của kênh thông báo (không chứa vault_secret_id hay secret).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicNotificationChannel {
    /// ID định danh kênh.
    pub id: String,
    /// ID workspace quản lý.
    pub workspace_id: String,
    /// Tên gợi nhớ của kênh.
    pub name: String,
    /// Loại kênh.
    pub channel_type: String,
    /// Kênh đã có bí mật trong kho khóa chưa.
    pub has_secret: bool,
    /// Chat ID (Telegram).
    pub chat_id: Option<String>,
    /// Trạng thái bật/tắt của kênh.
    pub is_enabled: bool,
    /// Cấu hình loại sự kiện.
    pub notify_on_needs_human: bool,
    /// Cấu hình loại sự kiện.
    pub notify_on_failed: bool,
    /// Cấu hình loại sự kiện.
    pub notify_on_approval_requested: bool,
    /// Cấu hình loại sự kiện.
    pub notify_on_limit_warning: bool,
    /// Thời điểm tạo.
    pub created_at: String,
    /// Thời điểm cập nhật cuối.
    pub updated_at: String,
}

impl From<NotificationChannel> for PublicNotificationChannel {
    fn from(c: NotificationChannel) -> Self {
        Self {
            id: c.id,
            workspace_id: c.workspace_id,
            name: c.name,
            channel_type: c.channel_type,
            has_secret: !c.vault_secret_id.is_empty(),
            chat_id: c.chat_id,
            is_enabled: c.is_enabled,
            notify_on_needs_human: c.notify_on_needs_human,
            notify_on_failed: c.notify_on_failed,
            notify_on_approval_requested: c.notify_on_approval_requested,
            notify_on_limit_warning: c.notify_on_limit_warning,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

/// Yêu cầu tạo mới một kênh thông báo.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateChannelRequest {
    /// Tên kênh.
    pub name: String,
    /// Loại kênh: telegram, discord, slack.
    pub channel_type: String,
    /// Bí mật (bot token hoặc webhook URL).
    pub secret: String,
    /// Chat ID cho Telegram.
    pub chat_id: Option<String>,
    /// Bật thông báo needs_human (mặc định true).
    pub notify_on_needs_human: Option<bool>,
    /// Bật thông báo failed (mặc định true).
    pub notify_on_failed: Option<bool>,
    /// Bật thông báo approval_requested (mặc định true).
    pub notify_on_approval_requested: Option<bool>,
    /// Bật thông báo limit_warning (mặc định true).
    pub notify_on_limit_warning: Option<bool>,
}

/// Yêu cầu cập nhật cấu hình kênh thông báo.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateChannelRequest {
    /// Tên kênh mới.
    pub name: Option<String>,
    /// Bí mật mới (nếu muốn thay đổi).
    pub secret: Option<String>,
    /// Chat ID cho Telegram.
    pub chat_id: Option<String>,
    /// Bật/tắt kênh.
    pub is_enabled: Option<bool>,
    /// Bật thông báo needs_human.
    pub notify_on_needs_human: Option<bool>,
    /// Bật thông báo failed.
    pub notify_on_failed: Option<bool>,
    /// Bật thông báo approval_requested.
    pub notify_on_approval_requested: Option<bool>,
    /// Bật thông báo limit_warning.
    pub notify_on_limit_warning: Option<bool>,
}

/// Bản ghi lịch sử thông báo.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct NotificationHistoryRecord {
    /// ID bản ghi.
    pub id: String,
    /// ID workspace.
    pub workspace_id: String,
    /// ID kênh.
    pub channel_id: String,
    /// Loại sự kiện (needs_human, failed, test, v.v.).
    pub event_type: String,
    /// Task liên quan (nếu có).
    pub task_id: Option<String>,
    /// Tiêu đề thông báo.
    pub title: String,
    /// Trạng thái: sent, failed, deduplicated, rate_limited.
    pub status: String,
    /// Thông báo lỗi (nếu có).
    pub error_message: Option<String>,
    /// Thời điểm gửi/ghi nhận.
    pub created_at: String,
}

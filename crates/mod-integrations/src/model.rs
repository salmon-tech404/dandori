//! Cấu trúc dữ liệu và mô hình thực thể cho module `integrations` (P2-06).

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Kết nối tích hợp nền tảng Git hoặc quản lý công việc.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationConnection {
    /// ID kết nối duy nhất (UUID).
    pub id: String,
    /// ID workspace sở hữu kết nối.
    pub workspace_id: String,
    /// Nhà cung cấp (`github`, `gitlab`, `linear`).
    pub provider: String,
    /// Tên gợi nhớ của kết nối.
    pub name: String,
    /// URL cơ sở của API (`https://api.github.com`, `https://gitlab.com`, `https://api.linear.app`).
    pub base_url: String,
    /// Token truy cập đã được mã hóa trong kho khóa.
    pub token_encrypted: String,
    /// Cho phép kết nối máy chủ nội bộ (chỉ dành cho GitLab self-hosted có xác nhận).
    pub allow_private_network: bool,
    /// Trạng thái kết nối (`active`, `error`, `disabled`).
    pub status: String,
    /// Thời điểm khởi tạo.
    pub created_at: String,
    /// Thời điểm cập nhật cuối cùng.
    pub updated_at: String,
}

/// Dữ liệu kết nối trả về cho giao diện (tuyệt đối KHÔNG chứa token).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicIntegrationConnection {
    /// ID kết nối.
    pub id: String,
    /// ID workspace.
    pub workspace_id: String,
    /// Nhà cung cấp.
    pub provider: String,
    /// Tên kết nối.
    pub name: String,
    /// URL cơ sở.
    pub base_url: String,
    /// Cho phép mạng nội bộ.
    pub allow_private_network: bool,
    /// Trạng thái hoạt động.
    pub status: String,
    /// Thời điểm khởi tạo.
    pub created_at: String,
    /// Thời điểm cập nhật cuối cùng.
    pub updated_at: String,
}

impl From<IntegrationConnection> for PublicIntegrationConnection {
    fn from(c: IntegrationConnection) -> Self {
        Self {
            id: c.id,
            workspace_id: c.workspace_id,
            provider: c.provider,
            name: c.name,
            base_url: c.base_url,
            allow_private_network: c.allow_private_network,
            status: c.status,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

/// Yêu cầu tạo mới kết nối tích hợp.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateConnectionRequest {
    /// Nhà cung cấp (`github`, `gitlab`, `linear`).
    pub provider: String,
    /// Tên gợi nhớ.
    #[serde(default)]
    pub name: String,
    /// URL cơ sở tùy chọn (mặc định lấy theo provider).
    pub base_url: Option<String>,
    /// Token truy cập cá nhân (PAT).
    pub token: String,
    /// Cho phép mạng nội bộ (mặc định false).
    #[serde(default)]
    pub allow_private_network: bool,
}

/// Yêu cầu nhập issue từ Linear vào backlog task của dự án.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinearImportRequest {
    /// ID dự án nhận task.
    pub project_id: String,
    /// Danh sách ID của các issue trên Linear cần nhập.
    pub issue_ids: Vec<String>,
}

/// Thông tin một issue trên Linear.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearIssue {
    /// ID duy nhất của issue trên Linear.
    pub id: String,
    /// Mã định danh (ví dụ "DAN-123").
    pub identifier: String,
    /// Tiêu đề issue.
    pub title: String,
    /// Mô tả tóm tắt nếu có.
    pub description: Option<String>,
    /// Đường dẫn web tới issue.
    pub url: String,
    /// Tên trạng thái (ví dụ "Todo", "In Progress").
    pub state_name: String,
}

/// Bản ghi liên kết giữa Task trong Dandori và Issue trên hệ thống ngoài.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationIssueLink {
    /// ID bản ghi liên kết.
    pub id: String,
    /// ID workspace.
    pub workspace_id: String,
    /// ID dự án.
    pub project_id: String,
    /// ID task trong Dandori.
    pub task_id: String,
    /// Nhà cung cấp.
    pub provider: String,
    /// ID của issue bên ngoài.
    pub issue_id: String,
    /// Mã định danh issue.
    pub issue_key: String,
    /// Tiêu đề issue.
    pub issue_title: String,
    /// Đường dẫn tới issue.
    pub issue_url: String,
    /// Thời điểm khởi tạo.
    pub created_at: String,
}

impl IntegrationConnection {
    /// Tạo đối tượng kết nối mới kèm thời gian hiện tại.
    #[must_use]
    pub fn new(
        workspace_id: &str,
        provider: &str,
        name: &str,
        base_url: &str,
        token_encrypted: &str,
        allow_private_network: bool,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            workspace_id: workspace_id.to_string(),
            provider: provider.to_string(),
            name: name.to_string(),
            base_url: base_url.to_string(),
            token_encrypted: token_encrypted.to_string(),
            allow_private_network,
            status: "active".to_string(),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

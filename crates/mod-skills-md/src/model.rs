//! Định nghĩa các cấu trúc dữ liệu cho module `skills-md` (P3-08).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Trạng thái phê duyệt của kỹ năng.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SkillStatus {
    /// Đang chờ quản trị viên phê duyệt
    PendingApproval,
    /// Đã được phê duyệt, được phép đưa vào prompt
    Approved,
    /// Bị từ chối
    Rejected,
}

impl SkillStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PendingApproval => "pending_approval",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s {
            "approved" => Self::Approved,
            "rejected" => Self::Rejected,
            _ => Self::PendingApproval,
        }
    }
}

/// Tóm tắt thông tin kỹ năng để hiển thị danh sách.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct SkillSummary {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub has_scripts: bool,
    pub status: SkillStatus,
    pub content_hash: String,
    pub is_hash_matching_approved: bool,
    pub binding_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

/// Chi tiết đầy đủ của một kỹ năng.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct SkillDetail {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub tools: Vec<String>,
    pub content: String,
    pub content_hash: String,
    pub approved_hash: Option<String>,
    pub has_scripts: bool,
    pub script_files: Vec<String>,
    pub status: SkillStatus,
    pub bindings: Vec<SkillBinding>,
    pub created_at: String,
    pub updated_at: String,
}

/// Gắn kết kỹ năng với dự án và/hoặc vai trò agent.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct SkillBinding {
    pub id: String,
    pub workspace_id: String,
    pub skill_id: String,
    pub project_id: Option<String>,
    pub role: Option<String>,
    pub embed_full: bool,
    pub created_at: String,
}

/// Payload yêu cầu nhập kỹ năng từ thư mục cục bộ.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ImportDirPayload {
    pub dir_path: String,
}

/// Payload yêu cầu nhập kỹ năng từ tệp nén zip (mã hóa Base64).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ImportZipPayload {
    pub file_name: String,
    pub base64_content: String,
}

/// Payload tạo gắn kết kỹ năng với dự án/vai trò.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct CreateBindingPayload {
    pub project_id: Option<String>,
    pub role: Option<String>,
    pub embed_full: Option<bool>,
}

/// Tệp tin đã giải nén an toàn trong bộ nhớ.
#[derive(Debug, Clone)]
pub struct ExtractedFile {
    pub relative_path: String,
    pub content: Vec<u8>,
}

/// Kết quả phân tích cú pháp toàn diện từ thư mục hoặc tệp nén.
#[derive(Debug, Clone)]
pub struct ParsedSkill {
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub tools: Vec<String>,
    pub content: String,
    pub has_scripts: bool,
    pub script_files: Vec<String>,
    pub content_hash: String,
}

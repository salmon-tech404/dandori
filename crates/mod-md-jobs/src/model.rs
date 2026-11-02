//! Mô hình dữ liệu và DTO cho module `md-jobs`.

use serde::{Deserialize, Serialize};

/// Thông tin công việc Markdown được lưu trong cơ sở dữ liệu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct MdJobRecord {
    /// Định danh duy nhất của job (UUID).
    pub id: String,
    /// Định danh workspace sở hữu.
    pub workspace_id: String,
    /// Định danh dự án chứa job.
    pub project_id: String,
    /// Đường dẫn tương đối của tệp trong repo (ví dụ `.cron/jobs/check.md`).
    pub file_path: String,
    /// Tên công việc (từ trường `name` trong front matter).
    pub job_name: String,
    /// Biểu thức cron (từ trường `schedule`).
    pub schedule_cron: String,
    /// Agent thực thi (từ trường `agent`).
    pub agent_role: String,
    /// Tài khoản ưu tiên (tùy chọn, từ trường `account`).
    pub account_id: Option<String>,
    /// Nội dung prompt cho agent (thân tệp Markdown).
    pub prompt: String,
    /// Mã băm SHA-256 toàn bộ nội dung tệp hiện tại.
    pub content_hash: String,
    /// Mã băm SHA-256 của nội dung đã được chủ dự án phê duyệt.
    pub approved_hash: Option<String>,
    /// Trạng thái: 'pending_approval', 'approved', 'rejected', 'syntax_error'.
    pub status: String,
    /// Thời điểm kích hoạt dự kiến kế tiếp (RFC3339).
    pub next_fire: Option<String>,
    /// Thời điểm kích hoạt gần nhất (RFC3339).
    pub last_fire: Option<String>,
    /// Thời điểm phát hiện/tạo bản ghi (RFC3339).
    pub created_at: String,
    /// Thời điểm cập nhật trạng thái gần nhất (RFC3339).
    pub updated_at: String,
}

impl MdJobRecord {
    /// Chuyển đổi sang DTO trả về cho client.
    #[must_use]
    pub fn to_dto(&self) -> MdJobDto {
        MdJobDto {
            id: self.id.clone(),
            workspace_id: self.workspace_id.clone(),
            project_id: self.project_id.clone(),
            file_path: self.file_path.clone(),
            job_name: self.job_name.clone(),
            schedule_cron: self.schedule_cron.clone(),
            agent_role: self.agent_role.clone(),
            account_id: self.account_id.clone(),
            prompt: self.prompt.clone(),
            content_hash: self.content_hash.clone(),
            approved_hash: self.approved_hash.clone(),
            status: self.status.clone(),
            next_fire: self.next_fire.clone(),
            last_fire: self.last_fire.clone(),
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
        }
    }
}

/// DTO trả về danh sách/chi tiết công việc Markdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MdJobDto {
    /// Định danh duy nhất của job.
    pub id: String,
    /// Định danh workspace.
    pub workspace_id: String,
    /// Định danh dự án.
    pub project_id: String,
    /// Đường dẫn tệp.
    pub file_path: String,
    /// Tên công việc.
    pub job_name: String,
    /// Biểu thức cron.
    pub schedule_cron: String,
    /// Agent thực thi.
    pub agent_role: String,
    /// Tài khoản ưu tiên.
    pub account_id: Option<String>,
    /// Prompt cho agent.
    pub prompt: String,
    /// Mã băm nội dung hiện tại.
    pub content_hash: String,
    /// Mã băm nội dung đã duyệt.
    pub approved_hash: Option<String>,
    /// Trạng thái phê duyệt.
    pub status: String,
    /// Lần chạy tiếp theo.
    pub next_fire: Option<String>,
    /// Lần chạy gần nhất.
    pub last_fire: Option<String>,
    /// Thời điểm tạo.
    pub created_at: String,
    /// Thời điểm cập nhật.
    pub updated_at: String,
}

/// Kết quả phân tích tệp Markdown thành công.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedJob {
    /// Tên job.
    pub name: String,
    /// Biểu thức cron.
    pub schedule: String,
    /// Agent thực thi.
    pub agent: String,
    /// Tài khoản (tùy chọn).
    pub account: Option<String>,
    /// Thân Markdown (prompt).
    pub prompt: String,
    /// Mã băm SHA-256 toàn bộ nội dung tệp.
    pub content_hash: String,
}

/// Các loại lỗi khi phân tích cú pháp tệp Markdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Tệp vượt quá giới hạn kích thước an toàn (64 KB).
    FileTooLarge(usize),
    /// Thiếu thẻ mở hoặc đóng front matter `---`.
    MissingFrontmatterDelimiters,
    /// Khóa lạ trong front matter (chỉ cho phép `name`, `schedule`, `agent`, `account`).
    UnknownField(String),
    /// Thiếu trường bắt buộc (`name`, `schedule`, `agent`).
    MissingRequiredField(String),
    /// Cú pháp YAML lồng nhau hoặc quá phức tạp bị từ chối.
    NestingNotAllowed(String),
    /// Biểu thức cron không hợp lệ.
    InvalidCron(String),
    /// Tệp rỗng hoặc định dạng không hợp lệ.
    Malformed(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileTooLarge(sz) => write!(f, "Tệp vượt quá giới hạn 64KB ({sz} bytes)"),
            Self::MissingFrontmatterDelimiters => {
                write!(f, "Thiếu dấu phân cách front matter '---'")
            }
            Self::UnknownField(k) => write!(f, "Trường front matter không được phép: '{k}'"),
            Self::MissingRequiredField(k) => {
                write!(f, "Thiếu trường bắt buộc trong front matter: '{k}'")
            }
            Self::NestingNotAllowed(line) => {
                write!(f, "Cấu trúc YAML lồng nhau không được phép: '{line}'")
            }
            Self::InvalidCron(e) => write!(f, "Biểu thức cron không hợp lệ: {e}"),
            Self::Malformed(msg) => write!(f, "Định dạng tệp không hợp lệ: {msg}"),
        }
    }
}

impl std::error::Error for ParseError {}

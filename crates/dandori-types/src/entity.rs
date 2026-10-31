//! Các kiểu thực thể dữ liệu lõi của Dandori (RS-TYPE-001, RS-TYPE-003).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::id::{
    AgentAccountId, FeatureAuditId, FeatureModuleId, FeaturePackId, ProjectId, RunEventId, RunId,
    SecretId, SessionId, TaskId, UserId, WorkspaceId,
};

/// Người dùng trong hệ thống Dandori.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct User {
    /// Định danh người dùng.
    pub id: UserId,
    /// Tên đăng nhập duy nhất.
    pub username: String,
    /// Chuỗi băm mật khẩu (Argon2).
    pub password_hash: String,
    /// Vai trò hệ thống (`admin` hoặc `user`).
    pub system_role: String,
    /// Thời điểm tạo tài khoản.
    pub created_at: DateTime<Utc>,
}

/// Phiên đăng nhập người dùng.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct Session {
    /// Định danh phiên.
    pub id: SessionId,
    /// Định danh người dùng sở hữu phiên.
    pub user_id: UserId,
    /// Thời điểm hết hạn phiên.
    pub expires_at: DateTime<Utc>,
    /// Chuỗi băm của token phiên.
    pub token_hash: String,
}

/// Không gian làm việc của Dandori.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct Workspace {
    /// Định danh không gian làm việc.
    pub id: WorkspaceId,
    /// Tên không gian làm việc.
    pub name: String,
    /// Thời điểm khởi tạo.
    pub created_at: DateTime<Utc>,
}

/// Thành viên thuộc một không gian làm việc.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct WorkspaceMember {
    /// Không gian làm việc.
    pub workspace_id: WorkspaceId,
    /// Người dùng thành viên.
    pub user_id: UserId,
    /// Vai trò (`admin` hoặc `member`).
    pub role: String,
}

/// Dự án trực thuộc một không gian làm việc.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct Project {
    /// Định danh dự án.
    pub id: ProjectId,
    /// Không gian làm việc sở hữu dự án.
    pub workspace_id: WorkspaceId,
    /// Tên dự án.
    pub name: String,
    /// Đường dẫn thư mục kho mã nguồn (git repository).
    pub repo_path: String,
    /// Nhánh git mặc định.
    pub default_branch: String,
    /// Thời điểm tạo dự án.
    pub created_at: DateTime<Utc>,
}

/// Trạng thái của task công việc trong chu trình Kanban (T11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task nằm trong danh sách chờ.
    Backlog,
    /// Task đang được thực thi.
    Running,
    /// Task cần người dùng can thiệp hoặc phản hồi.
    NeedsHuman,
    /// Task đang trong quá trình duyệt lại mã nguồn.
    InReview,
    /// Task đã sẵn sàng để merge vào nhánh chính.
    ReadyToMerge,
    /// Task hoàn thành thành công.
    Done,
    /// Task thất bại.
    Failed,
    /// Task bị hủy bỏ.
    Cancelled,
}

impl TaskStatus {
    /// Kiểm tra xem việc chuyển từ trạng thái hiện tại sang `target` có hợp lệ không.
    #[must_use]
    pub fn can_transition_to(self, target: Self) -> bool {
        if self == target {
            return true;
        }
        match (self, target) {
            (Self::Backlog, Self::Running | Self::Cancelled) => true,
            (Self::Running, Self::NeedsHuman | Self::InReview | Self::Failed | Self::Cancelled) => {
                true
            }
            (Self::NeedsHuman, Self::Running | Self::Failed | Self::Cancelled) => true,
            (
                Self::InReview,
                Self::ReadyToMerge | Self::Running | Self::Failed | Self::Cancelled,
            ) => true,
            (Self::ReadyToMerge, Self::Done | Self::Running | Self::Failed | Self::Cancelled) => {
                true
            }
            (Self::Failed, Self::Backlog | Self::Cancelled) => true,
            (Self::Cancelled, Self::Backlog) => true,
            (Self::Done, _) => false,
            _ => false,
        }
    }

    /// Trả về chuỗi đại diện chuẩn của trạng thái.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Backlog => "backlog",
            Self::Running => "running",
            Self::NeedsHuman => "needs_human",
            Self::InReview => "in_review",
            Self::ReadyToMerge => "ready_to_merge",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "backlog" => Ok(Self::Backlog),
            "running" => Ok(Self::Running),
            "needs_human" => Ok(Self::NeedsHuman),
            "in_review" => Ok(Self::InReview),
            "ready_to_merge" => Ok(Self::ReadyToMerge),
            "done" => Ok(Self::Done),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            "todo" => Ok(Self::Backlog),
            "in_progress" => Ok(Self::Running),
            other => Err(format!("Trạng thái task '{other}' không hợp lệ")),
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Task công việc của một dự án.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct Task {
    /// Định danh task.
    pub id: TaskId,
    /// Dự án sở hữu task.
    pub project_id: ProjectId,
    /// Tiêu đề task.
    pub title: String,
    /// Mô tả chi tiết yêu cầu công việc.
    pub description: String,
    /// Trạng thái task (`todo`, `in_progress`, `done`, `failed`, `cancelled`).
    pub status: String,
    /// Định danh agent trưởng phối hợp.
    pub lead_agent: String,
    /// Định danh agent thực thi trực tiếp.
    pub worker_agent: String,
    /// Đường dẫn thư mục làm việc độc lập (worktree), nếu có.
    pub workdir_path: Option<String>,
    /// Thời điểm tạo task.
    pub created_at: DateTime<Utc>,
    /// Thời điểm cập nhật trạng thái gần nhất.
    pub updated_at: DateTime<Utc>,
}

/// Một lần chạy tác vụ của agent (Run).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct Run {
    /// Định danh lần chạy.
    pub id: RunId,
    /// Task được thực thi trong lần chạy này.
    pub task_id: TaskId,
    /// Agent thực thi (`claude`, `gemini`, `codex`, `fake`...).
    pub agent: String,
    /// Tài khoản agent được phân bổ.
    pub account: String,
    /// Trạng thái lần chạy (`queued`, `running`, `success`, `failed`, `cancelled`).
    pub status: String,
    /// Thời điểm bắt đầu chạy.
    pub started_at: DateTime<Utc>,
    /// Thời điểm kết thúc chạy, nếu đã xong.
    pub finished_at: Option<DateTime<Utc>>,
    /// Mã thoát của tiến trình (exit code), nếu có.
    pub exit_code: Option<i64>,
}

/// Sự kiện hoặc dòng dữ liệu nhật ký phát sinh trong một lần chạy (RunEvent).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct RunEvent {
    /// Định danh sự kiện lần chạy.
    pub id: RunEventId,
    /// Định danh lần chạy gắn liền.
    pub run_id: RunId,
    /// Thứ tự tuần tự của sự kiện trong lần chạy.
    pub seq: i64,
    /// Loại sự kiện (`output`, `tool_call`, `error`...).
    pub event_type: String,
    /// Nội dung dữ liệu định dạng JSON.
    pub data: String,
    /// Thời điểm ghi nhận sự kiện.
    pub created_at: DateTime<Utc>,
}

/// Bản ghi khóa bí mật đã mã hóa lưu trữ trong kho cục bộ.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SecretRecord {
    /// Định danh khóa bí mật.
    pub id: SecretId,
    /// Nhãn mô tả khóa.
    pub label: String,
    /// Giá trị nonce dùng cho thuật toán giải mã AEAD.
    pub nonce: String,
    /// Dữ liệu đã mã hóa (ciphertext dạng byte).
    pub ciphertext: Vec<u8>,
    /// Thời điểm lưu trữ khóa.
    pub created_at: DateTime<Utc>,
}

/// Tài khoản agent được kết nối với Dandori.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct AgentAccount {
    /// Định danh tài khoản.
    pub id: AgentAccountId,
    /// Nhà cung cấp AI (`anthropic`, `google`, `openai`...).
    pub provider: String,
    /// Nhãn hiển thị của tài khoản.
    pub label: String,
    /// Thư mục HOME riêng biệt dành cho CLI của tài khoản này.
    pub home_dir: String,
    /// Loại xác thực (`cli` hoặc `api_key`).
    pub auth_type: String,
    /// Tham chiếu khóa bí mật mã hóa nếu dùng `api_key`.
    pub secret_id: Option<SecretId>,
    /// Trạng thái đăng nhập (`not_installed`, `not_logged_in`, `ready`, `session_expired`, `unknown`).
    pub status: String,
    /// Có đang được kích hoạt hay không.
    pub enabled: bool,
    /// Độ ưu tiên điều phối (số nhỏ = ưu tiên cao).
    pub priority: i64,
    /// Thời điểm hết hạn mức (nếu đang bị tạm ngừng).
    pub exhausted_until: Option<DateTime<Utc>>,
    /// Thời điểm kiểm tra trạng thái gần nhất.
    pub last_checked_at: Option<DateTime<Utc>>,
}

/// Khai báo module tính năng trong hệ thống (Feature Module).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct FeatureModule {
    /// Định danh module.
    pub id: FeatureModuleId,
    /// Tên nhóm module.
    pub group_name: String,
    /// Tên hiển thị module.
    pub name: String,
    /// Mô tả chi tiết tính năng module.
    pub description: String,
    /// Phiên bản module.
    pub version: String,
    /// Danh sách định danh các module phụ thuộc bắc cầu.
    pub requires: Vec<String>,
    /// Các phạm vi được phép cấu hình (`system`, `workspace`, `project`).
    pub scopes: Vec<String>,
    /// Mức độ rủi ro (`low`, `medium`, `high`).
    pub risk: String,
    /// Mức độ công sức triển khai (`low`, `medium`, `high`).
    pub effort: String,
    /// Cờ đánh dấu module thuộc lõi hệ thống.
    pub core: bool,
}

/// Cấu hình trạng thái bật/tắt của module theo phạm vi (Feature Setting).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct FeatureSetting {
    /// Module được áp dụng cấu hình.
    pub module_id: FeatureModuleId,
    /// Loại phạm vi (`system`, `workspace`, `project`).
    pub scope_kind: String,
    /// Định danh của phạm vi đó.
    pub scope_id: String,
    /// Trạng thái bật hoặc tắt.
    pub enabled: bool,
    /// Trạng thái khóa không cho cấp dưới ghi đè.
    pub locked: bool,
    /// Chuỗi JSON cấu hình chi tiết theo schema của module.
    pub config: String,
    /// Người thực hiện thay đổi.
    pub changed_by: String,
    /// Thời điểm thay đổi.
    pub changed_at: DateTime<Utc>,
}

/// Gói mẫu tập hợp các module tính năng (Feature Pack).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct FeaturePack {
    /// Định danh gói mẫu.
    pub id: FeaturePackId,
    /// Tên gói mẫu.
    pub name: String,
    /// Danh sách các module thuộc gói.
    pub modules: Vec<String>,
}

/// Bản ghi nhật ký kiểm toán hành vi thay đổi tính năng (Feature Audit).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct FeatureAudit {
    /// Định danh sự kiện kiểm toán.
    pub id: FeatureAuditId,
    /// Module bị tác động.
    pub module_id: FeatureModuleId,
    /// Loại phạm vi (`system`, `workspace`, `project`).
    pub scope_kind: String,
    /// Định danh phạm vi tác động.
    pub scope_id: String,
    /// Hành động (`enable`, `disable`, `configure`, `rollback`).
    pub action: String,
    /// Người hoặc agent thực hiện hành động.
    pub performed_by: String,
    /// Lý do thực hiện.
    pub reason: String,
    /// Thời điểm ghi nhận.
    pub created_at: DateTime<Utc>,
}

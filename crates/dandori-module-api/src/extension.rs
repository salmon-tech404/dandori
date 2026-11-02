//! Các điểm mở rộng của lõi Dandori (01_KIEN_TRUC.md mục 4.5).
//!
//! Gồm `WorkdirProvider` (P1-01), `PromptContributor` (P1-01), và `TransitionGuard` (P2-03).

use chrono::{DateTime, Utc};
use dandori_agent_port::AgentRole;
use dandori_types::{AgentAccountId, Run, Task, TaskId, TaskStatus, UserId};
use std::fmt;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

/// Kiểu Future trả về cho các hàm bất đồng bộ của điểm mở rộng.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Thư mục làm việc chuẩn bị cho một task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workdir {
    /// Đường dẫn thư mục làm việc tuyệt đối hoặc tương đối chuẩn hóa.
    pub path: PathBuf,
}

impl Workdir {
    /// Khởi tạo một thể hiện `Workdir` mới.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

/// Lý do giải phóng thư mục làm việc sau khi thực thi task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseReason {
    /// Tác vụ hoàn thành thành công.
    Completed,
    /// Tác vụ thất bại hoặc lỗi.
    Failed,
    /// Tác vụ bị hủy bỏ chủ động.
    Cancelled,
    /// Dọn dẹp thông thường hoặc khôi phục trạng thái.
    Cleanup,
}

/// Lỗi phát sinh từ `WorkdirProvider`.
#[derive(Debug, thiserror::Error)]
pub enum WorkdirError {
    /// Lỗi khi chuẩn bị thư mục làm việc.
    #[error("chuẩn bị thư mục làm việc thất bại: {0}")]
    PrepareFailed(String),

    /// Lỗi khi giải phóng thư mục làm việc.
    #[error("giải phóng thư mục làm việc thất bại: {0}")]
    ReleaseFailed(String),

    /// Lỗi nội bộ khác.
    #[error("lỗi thư mục làm việc: {0}")]
    Other(String),
}

/// Điểm mở rộng quản lý việc chuẩn bị và giải phóng thư mục làm việc của task.
pub trait WorkdirProvider: fmt::Debug + Send + Sync {
    /// Tên định danh của provider.
    fn name(&self) -> &'static str;

    /// Chuẩn bị thư mục làm việc cho task.
    fn prepare(&self, task: &Task) -> BoxFuture<'static, Result<Workdir, WorkdirError>>;

    /// Giải phóng thư mục làm việc của task sau khi chạy xong.
    fn release(
        &self,
        task: &Task,
        reason: ReleaseReason,
    ) -> BoxFuture<'static, Result<(), WorkdirError>>;
}

/// Một phần chỉ thị (prompt) đóng góp vào prompt chung của lần chạy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptPart {
    /// Nhãn nguồn cung cấp phần chỉ thị này (ví dụ "prompt-library", "tech-profile").
    pub source: String,
    /// Thứ tự ưu tiên (số nhỏ hơn được xếp trước trong nội dung prompt).
    pub order: i32,
    /// Nội dung văn bản đóng góp.
    pub content: String,
}

impl PromptPart {
    /// Khởi tạo một `PromptPart` mới.
    #[must_use]
    pub fn new(source: impl Into<String>, order: i32, content: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            order,
            content: content.into(),
        }
    }
}

/// Điểm mở rộng cho phép module đóng góp nội dung vào prompt gửi tới agent.
pub trait PromptContributor: fmt::Debug + Send + Sync {
    /// Tên định danh của contributor.
    fn name(&self) -> &'static str;

    /// Đóng góp các đoạn prompt cho task và vai trò của agent.
    fn contribute(&self, task: &Task, role: &AgentRole) -> BoxFuture<'static, Vec<PromptPart>>;
}

/// Nguồn gốc của hành động từ hệ thống.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SystemSource {
    /// Kích hoạt từ bộ lập lịch: `scheduler:<module-id>`
    Scheduler(String),
    /// Kích hoạt từ module: `module:<module-id>`
    Module(String),
}

impl fmt::Display for SystemSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scheduler(mod_id) => write!(f, "scheduler:{mod_id}"),
            Self::Module(mod_id) => write!(f, "module:{mod_id}"),
        }
    }
}

/// Tác nhân yêu cầu chuyển trạng thái nhiệm vụ.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TransitionActor {
    /// Người dùng kích hoạt với user_id.
    User(String),
    /// Hệ thống kích hoạt với nguồn xác định.
    System(SystemSource),
}

/// Quyết định của TransitionGuard khi đánh giá một yêu cầu chuyển trạng thái.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardDecision {
    /// Cho phép tiếp tục chuyển đổi trạng thái.
    Allow,
    /// Từ chối chuyển đổi trạng thái kèm lý do và thời gian chờ thử lại tùy chọn.
    Deny {
        /// Lý do từ chối.
        reason: String,
        /// Thời điểm có thể thử lại (nếu có).
        retry_after: Option<DateTime<Utc>>,
    },
    /// Yêu cầu phê duyệt từ con người (đặt task vào needs_human và phát approval.requested).
    NeedsApproval {
        /// Phân loại yêu cầu phê duyệt (ví dụ: "approval-gate").
        kind: String,
        /// Lý do cần phê duyệt.
        reason: String,
    },
}

/// Lỗi phát sinh trong quá trình kiểm tra TransitionGuard.
#[derive(Debug, thiserror::Error)]
pub enum GuardError {
    /// Lỗi khi kiểm tra chuyển trạng thái.
    #[error("lỗi kiểm tra chuyển trạng thái: {0}")]
    CheckFailed(String),

    /// Lỗi nội bộ khác.
    #[error("lỗi nội bộ guard: {0}")]
    Other(String),
}

/// Điểm mở rộng kiểm soát và can thiệp việc chuyển đổi trạng thái nhiệm vụ (P2-03).
pub trait TransitionGuard: fmt::Debug + Send + Sync {
    /// Tên định danh của guard.
    fn name(&self) -> &'static str;

    /// Kiểm tra xem bước chuyển trạng thái có được phép hay không.
    fn check(
        &self,
        task: &Task,
        from: TaskStatus,
        to: TaskStatus,
        actor: &TransitionActor,
    ) -> BoxFuture<'static, Result<GuardDecision, GuardError>>;
}

/// Điểm mở rộng cho phép module khởi chạy lần chạy của task (P2-04, P2-05).
pub trait TaskRunner: fmt::Debug + Send + Sync {
    /// Bắt đầu một lần chạy cho task với vai trò và người dùng chỉ định.
    fn start_run(
        &self,
        task_id: TaskId,
        role: AgentRole,
        user_id: UserId,
        exclude_accounts: Vec<AgentAccountId>,
    ) -> BoxFuture<'static, Result<Run, String>>;
}

/// Dữ liệu đầu vào để tạo Pull Request / Merge Request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreatePullRequestPayload {
    /// Tiêu đề của Pull Request.
    pub title: String,
    /// Nội dung mô tả của Pull Request.
    pub body: String,
    /// Nhánh nguồn (head branch) chứa các thay đổi.
    pub source_branch: String,
    /// Nhánh đích (base branch) nhận các thay đổi.
    pub target_branch: String,
}

/// Trạng thái của Pull Request / Merge Request từ máy chủ từ xa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PullRequestState {
    /// Đang mở.
    Open,
    /// Đã đóng (không merge).
    Closed,
    /// Đã merge.
    Merged,
}

/// Thông tin chi tiết của một Pull Request / Merge Request trên máy chủ từ xa.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemotePullRequest {
    /// Số định danh của PR (PR number / MR IID).
    pub number: u64,
    /// URL hiển thị trên trình duyệt.
    pub url: String,
    /// Tiêu đề.
    pub title: String,
    /// Trạng thái PR.
    pub state: PullRequestState,
    /// Nhánh nguồn.
    pub source_branch: String,
    /// Nhánh đích.
    pub target_branch: String,
    /// Có thể merge được hay không (nếu máy chủ tính toán được).
    pub mergeable: Option<bool>,
}

/// Trạng thái tổng hợp của CI trên một commit hoặc ref.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CiState {
    /// Tất cả checks/pipelines đều thành công.
    Success,
    /// Đang chạy hoặc chờ chạy.
    Pending,
    /// Có kiểm tra thất bại.
    Failure,
    /// Gặp lỗi hệ thống hoặc bị hủy.
    Error,
}

/// Tóm tắt trạng thái CI từ Git host.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CiStatusSummary {
    /// Trạng thái tổng quát.
    pub state: CiState,
    /// Tóm tắt mô tả (ví dụ "12/12 checks passed").
    pub description: String,
    /// Số lượng checks thành công.
    pub success_count: u32,
    /// Số lượng checks thất bại.
    pub failure_count: u32,
    /// Số lượng checks đang chạy.
    pub pending_count: u32,
}

/// Phương thức merge Pull Request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MergeMethod {
    /// Merge commit thông thường.
    Merge,
    /// Squash and merge.
    Squash,
    /// Rebase and merge.
    Rebase,
}

/// Kết quả sau khi thực hiện lệnh merge.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MergeResult {
    /// Đã merge thành công hay chưa.
    pub merged: bool,
    /// SHA của commit merge (nếu có).
    pub sha: Option<String>,
    /// Thông điệp kết quả từ máy chủ.
    pub message: String,
}

/// Lỗi phát sinh từ `GitHostProvider`.
#[derive(Debug, thiserror::Error)]
pub enum GitHostError {
    /// Lỗi xác thực hoặc quyền hạn.
    #[error("lỗi xác thực git host: {0}")]
    AuthenticationFailed(String),

    /// Không tìm thấy tài nguyên (repo, PR, commit).
    #[error("không tìm thấy tài nguyên: {0}")]
    NotFound(String),

    /// Xung đột hoặc điều kiện tiên quyết không thỏa mãn (không thể merge).
    #[error("không thể merge PR do xung đột hoặc chưa đạt điều kiện: {0}")]
    MergeConflict(String),

    /// Lỗi mạng hoặc HTTP.
    #[error("lỗi kết nối mạng tới git host: {0}")]
    Network(String),

    /// Lỗi cấu hình hoặc tham số không hợp lệ.
    #[error("tham số git host không hợp lệ: {0}")]
    InvalidParameter(String),

    /// Lỗi nội bộ khác.
    #[error("lỗi máy chủ git: {0}")]
    Other(String),
}

/// Điểm mở rộng tích hợp nền tảng Git từ xa (P2-06).
pub trait GitHostProvider: fmt::Debug + Send + Sync {
    /// Tên định danh của provider (ví dụ: "github", "gitlab").
    fn name(&self) -> &'static str;

    /// Kiểm tra tính hợp lệ của kết nối và thông tin xác thực.
    fn test_connection(&self) -> BoxFuture<'static, Result<(), GitHostError>>;

    /// Tạo Pull Request / Merge Request mới.
    fn create_pull_request(
        &self,
        repo: &str,
        payload: &CreatePullRequestPayload,
    ) -> BoxFuture<'static, Result<RemotePullRequest, GitHostError>>;

    /// Lấy thông tin trạng thái của Pull Request / Merge Request.
    fn get_pull_request(
        &self,
        repo: &str,
        number: u64,
    ) -> BoxFuture<'static, Result<RemotePullRequest, GitHostError>>;

    /// Lấy trạng thái CI của commit/ref liên quan.
    fn get_ci_status(
        &self,
        repo: &str,
        git_ref: &str,
    ) -> BoxFuture<'static, Result<CiStatusSummary, GitHostError>>;

    /// Thực hiện merge Pull Request / Merge Request.
    fn merge_pull_request(
        &self,
        repo: &str,
        number: u64,
        method: MergeMethod,
    ) -> BoxFuture<'static, Result<MergeResult, GitHostError>>;
}

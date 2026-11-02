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

/// Lỗi xảy ra trong quá trình trang trí thư mục làm việc (P3-09).
#[derive(Debug, thiserror::Error)]
pub enum DecoratorError {
    /// Tên tệp không nằm trong danh sách cho phép của module.
    #[error("tệp '{0}' không nằm trong danh sách cho phép của module")]
    DisallowedFile(String),

    /// Đường dẫn tệp chứa ký tự phân cách thư mục hoặc '..'.
    #[error("đường dẫn tệp không hợp lệ hoặc chứa ký tự nguy hiểm: {0}")]
    UnsafePath(String),

    /// Lỗi I/O khi thao tác với tệp.
    #[error("lỗi I/O khi trang trí worktree: {0}")]
    Io(#[from] std::io::Error),

    /// Lỗi Git khi thao tác với repository/worktree.
    #[error("lỗi git khi cấu hình exclude worktree: {0}")]
    Git(String),

    /// Lỗi nội bộ khác.
    #[error("lỗi trang trí worktree: {0}")]
    Other(String),
}

/// Ngữ cảnh khi thực hiện trang trí thư mục làm việc (P3-09).
#[derive(Debug, Clone)]
pub struct DecoratorContext {
    /// ID workspace.
    pub workspace_id: String,
    /// ID dự án.
    pub project_id: String,
    /// ID tác vụ.
    pub task_id: String,
    /// Vai trò của agent đang chạy (worker, lead, test, doc...).
    pub role: AgentRole,
    /// Nhà cung cấp CLI (claude, codex, agy, cursor...).
    pub provider: Option<String>,
}

/// Kết quả trang trí thư mục làm việc (P3-09).
#[derive(Debug, Clone, Default)]
pub struct DecoratorResult {
    /// Danh sách tên tệp mới được tạo tại gốc worktree.
    pub created_files: Vec<String>,
    /// Danh sách tên tệp bị bỏ qua (do đã tồn tại trong repository gốc).
    pub skipped_files: Vec<String>,
    /// Danh sách tên tệp được nối thêm khối nội dung có ranh giới.
    pub appended_files: Vec<String>,
}

/// Điểm mở rộng trang trí thư mục làm việc trước khi agent chạy (P3-09).
pub trait WorkdirDecorator: fmt::Debug + Send + Sync {
    /// Danh sách tên tệp (tại gốc worktree) mà decorator được phép tạo/sửa.
    fn allowed_filenames(&self) -> &[&str];

    /// Thực hiện trang trí thư mục làm việc.
    fn decorate(
        &self,
        workdir: &std::path::Path,
        context: &DecoratorContext,
    ) -> BoxFuture<'static, Result<DecoratorResult, DecoratorError>>;
}

/// Tệp đích có tồn tại không; từ chối nếu đường dẫn là symlink/không phải tệp thường.
///
/// Không bao giờ ghi xuyên liên kết tượng trưng: kho mã không đáng tin có thể đặt `AGENTS.md` là symlink trỏ ra
/// ngoài worktree (kể cả symlink treo), khi đó ghi/nối sẽ sửa tệp ngoài thư mục làm việc.
async fn existing_regular_file(
    file_path: &std::path::Path,
    filename: &str,
) -> Result<bool, DecoratorError> {
    match tokio::fs::symlink_metadata(file_path).await {
        Ok(meta) => {
            if meta.file_type().is_symlink() || !meta.is_file() {
                return Err(DecoratorError::UnsafePath(filename.to_string()));
            }
            Ok(true)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(DecoratorError::from(e)),
    }
}

/// Thay hoặc nối khối nội dung có ranh giới `BEGIN/END` vào tệp hiện có.
fn merge_boundary_block(existing: &str, content: &str, marker: &str) -> String {
    let begin_marker = format!("<!-- BEGIN {marker} -->");
    let end_marker = format!("<!-- END {marker} -->");

    if existing.contains(&begin_marker) && existing.contains(&end_marker) {
        // Thay thế khối đã có
        let start = existing.find(&begin_marker).unwrap_or(0);
        let end = existing[start..]
            .find(&end_marker)
            .map_or(existing.len(), |i| start + i + end_marker.len());
        format!(
            "{}{}\n{}\n{}{}",
            &existing[..start],
            begin_marker,
            content.trim(),
            end_marker,
            &existing[end..]
        )
    } else {
        // Nối vào cuối tệp
        format!(
            "{}\n\n{}\n{}\n{}\n",
            existing.trim_end(),
            begin_marker,
            content.trim(),
            end_marker
        )
    }
}

/// Ghi hoặc nối thêm tệp an toàn vào gốc worktree và tự động thêm vào `.git/info/exclude` (P3-09).
///
/// # Errors
/// Trả về `DecoratorError` nếu đường dẫn không an toàn, không nằm trong danh sách cho phép, hoặc lỗi I/O.
pub async fn safe_write_worktree_file(
    workdir: &std::path::Path,
    filename: &str,
    content: &str,
    allowed: &[&str],
    append_mode: bool,
    boundary_marker: Option<&str>,
) -> Result<bool, DecoratorError> {
    // 1. Kiểm tra ký tự phân tách thư mục hoặc '..'
    if filename.trim().is_empty()
        || filename.contains('/')
        || filename.contains('\\')
        || filename.contains("..")
    {
        return Err(DecoratorError::UnsafePath(filename.to_string()));
    }

    // 2. Kiểm tra whitelist allowed_filenames
    if !allowed.contains(&filename) {
        return Err(DecoratorError::DisallowedFile(filename.to_string()));
    }

    let file_path = workdir.join(filename);
    let file_exists = existing_regular_file(&file_path, filename).await?;

    if file_exists {
        if !append_mode {
            // Chế độ skip: không ghi đè
            return Ok(false);
        }
        // Chế độ append: nối thêm khối có ranh giới
        let marker = boundary_marker.unwrap_or("DANDORI INSTRUCTION SYNC");
        let existing = tokio::fs::read_to_string(&file_path).await?;
        let updated = merge_boundary_block(&existing, content, marker);
        tokio::fs::write(&file_path, updated).await?;
    } else {
        // Tạo mới tệp
        tokio::fs::write(&file_path, content).await?;
    }

    // 3. Tự động thêm vào `.git/info/exclude` để chống bẩn worktree
    let git_client = dandori_git::GitClient::new(workdir);
    let _ = git_client.add_local_exclude(filename).await;

    Ok(true)
}

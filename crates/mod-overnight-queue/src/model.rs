//! Mô hình dữ liệu cho module `overnight-queue`.

use serde::{Deserialize, Serialize};

/// Trạng thái hoạt động của hàng đợi qua đêm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueStatus {
    /// Đang chờ tới giờ chạy hoặc chưa có task.
    Idle,
    /// Đang xử lý các task trong hàng đợi.
    Running,
    /// Tạm dừng bởi người dùng.
    Paused,
    /// Dừng an toàn (ví dụ: chạm giới hạn thất bại liên tiếp).
    Stopped,
}

impl QueueStatus {
    /// Chuyển sang chuỗi snake_case.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Stopped => "stopped",
        }
    }
}

impl std::str::FromStr for QueueStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "idle" => Ok(Self::Idle),
            "running" => Ok(Self::Running),
            "paused" => Ok(Self::Paused),
            "stopped" => Ok(Self::Stopped),
            other => Err(format!("Trạng thái hàng đợi không hợp lệ: {other}")),
        }
    }
}

/// Cấu hình hàng đợi qua đêm của một workspace.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OvernightQueueConfig {
    /// Mã không gian làm việc.
    pub workspace_id: String,
    /// Giờ bắt đầu "HH:MM" (mặc định "23:00").
    pub start_time: String,
    /// Múi giờ áp dụng.
    pub timezone: String,
    /// Số lượng task chạy song song tối đa (mặc định 1).
    pub max_concurrency: i64,
    /// Số task thất bại liên tiếp để kích hoạt dừng an toàn (mặc định 3).
    pub consecutive_failure_limit: i64,
    /// Trạng thái hiện tại của hàng đợi.
    pub status: String,
    /// Số lần thất bại liên tiếp gần nhất.
    pub consecutive_failures: i64,
    /// Tóm tắt chuỗi duyệt sáng hôm sau (JSON).
    pub last_run_summary: Option<String>,
    /// Thời điểm cập nhật (RFC 3339).
    pub updated_at: String,
}

/// Một mục trong hàng đợi qua đêm.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OvernightQueueItem {
    /// ID mục hàng đợi.
    pub id: String,
    /// Không gian làm việc.
    pub workspace_id: String,
    /// Mã nhiệm vụ.
    pub task_id: String,
    /// Thứ tự sắp xếp trong hàng đợi.
    pub order_index: i64,
    /// Trạng thái ('queued', 'running', 'completed', 'failed', 'paused').
    pub status: String,
    /// Mã lần chạy liên kết nếu có.
    pub run_id: Option<String>,
    /// Thời điểm thêm vào hàng đợi.
    pub created_at: String,
    /// Thời điểm cập nhật trạng thái.
    pub updated_at: String,
}

/// Mục hàng đợi kèm thông tin nhiệm vụ đầy đủ trả về cho giao diện.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItemWithTask {
    /// Mục hàng đợi.
    #[serde(flatten)]
    pub item: OvernightQueueItem,
    /// Tiêu đề task.
    pub task_title: String,
    /// Trạng thái task thực tế trong bảng task.
    pub task_status: String,
}

/// Dữ liệu tóm tắt sáng hôm sau ("Chuỗi để duyệt").
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MorningSummaryDto {
    /// Tổng số task đã hoàn thành.
    pub completed_count: usize,
    /// Tổng số task thất bại.
    pub failed_count: usize,
    /// Tổng số task cần người dùng can thiệp (needs_human).
    pub needs_human_count: usize,
    /// Danh sách chi tiết các task đã xử lý và nhánh git / PR chờ duyệt.
    pub review_items: Vec<ReviewChainItem>,
    /// Thời điểm tạo bản tóm tắt (RFC 3339).
    pub generated_at: String,
}

/// Một phần tử trong chuỗi để duyệt sáng hôm sau.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewChainItem {
    /// ID nhiệm vụ.
    pub task_id: String,
    /// Tiêu đề nhiệm vụ.
    pub title: String,
    /// Trạng thái kết quả (`in_review` hoặc `needs_human` hoặc `failed`).
    pub status: String,
    /// Nhánh git làm việc của task nếu có.
    pub branch_name: Option<String>,
}

/// Dữ liệu yêu cầu thêm task vào hàng đợi.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnqueueTasksRequest {
    /// Danh sách ID các task cần đưa vào hàng đợi.
    pub task_ids: Vec<String>,
}

/// Dữ liệu yêu cầu sắp xếp lại thứ tự hàng đợi.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderQueueRequest {
    /// Danh sách item_id theo thứ tự mới mong muốn.
    pub ordered_item_ids: Vec<String>,
}

/// Dữ liệu yêu cầu cập nhật cấu hình hàng đợi.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateQueueConfigRequest {
    /// Giờ bắt đầu "HH:MM".
    pub start_time: Option<String>,
    /// Múi giờ.
    pub timezone: Option<String>,
    /// Giới hạn song song N.
    pub max_concurrency: Option<i64>,
    /// Giới hạn thất bại liên tiếp K.
    pub consecutive_failure_limit: Option<i64>,
}

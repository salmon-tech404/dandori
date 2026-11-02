//! Định nghĩa các cấu trúc dữ liệu cho module `auto-lessons` (P3-11).

use chrono::Utc;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Cấu hình trích xuất bài học tự động theo dự án.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, TS)]
pub struct AutoLessonsConfig {
    /// Định danh dự án.
    pub project_id: String,
    /// Định danh không gian làm việc.
    pub workspace_id: String,
    /// Bật/tắt tính năng cho dự án.
    pub enabled: bool,
    /// Kích hoạt khi task chuyển sang `done`.
    pub trigger_on_done: bool,
    /// Kích hoạt khi task chuyển sang `failed`.
    pub trigger_on_failed: bool,
    /// Ngân sách token tối đa cho mỗi lần trích xuất.
    pub max_token_budget: i64,
    /// Độ dài tối thiểu của nội dung task để xử lý.
    pub min_content_length: i64,
    /// Thời điểm khởi tạo.
    pub created_at: String,
    /// Thời điểm cập nhật cuối cùng.
    pub updated_at: String,
}

impl AutoLessonsConfig {
    /// Tạo cấu hình mặc định cho dự án.
    #[must_use]
    pub fn default_for(workspace_id: &str, project_id: &str) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            project_id: project_id.to_string(),
            workspace_id: workspace_id.to_string(),
            enabled: true,
            trigger_on_done: true,
            trigger_on_failed: true,
            max_token_budget: 1000,
            min_content_length: 10,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

/// Yêu cầu cập nhật cấu hình auto-lessons.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct UpdateAutoLessonsConfigRequest {
    /// Bật/tắt tính năng.
    pub enabled: Option<bool>,
    /// Kích hoạt khi task done.
    pub trigger_on_done: Option<bool>,
    /// Kích hoạt khi task failed.
    pub trigger_on_failed: Option<bool>,
    /// Ngân sách token tối đa.
    pub max_token_budget: Option<i64>,
    /// Độ dài tối thiểu của task.
    pub min_content_length: Option<i64>,
}

/// Bản ghi tiến trình chạy rút bài học cho một task.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, TS)]
pub struct AutoLessonsRunRecord {
    /// Định danh bản ghi.
    pub id: String,
    /// Định danh không gian làm việc.
    pub workspace_id: String,
    /// Định danh dự án.
    pub project_id: String,
    /// Định danh task tương ứng (duy nhất, chống lặp).
    pub task_id: String,
    /// ID lần chạy agent trích xuất (nếu có).
    pub run_id: Option<String>,
    /// Trạng thái (`running`, `completed`, `skipped`, `failed`).
    pub status: String,
    /// Định danh mục nhớ bài học tạo trong `m_shared_memory_entry`.
    pub lesson_id: Option<String>,
    /// Lý do bỏ qua hoặc thất bại.
    pub reason: Option<String>,
    /// Thời điểm bắt đầu xử lý.
    pub created_at: String,
    /// Thời điểm kết thúc.
    pub finished_at: Option<String>,
}

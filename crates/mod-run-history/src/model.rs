//! Định nghĩa cấu trúc dữ liệu cho module `run-history`.

use serde::{Deserialize, Serialize};

/// Tóm tắt một lần chạy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    /// ID của lần chạy.
    pub id: String,
    /// ID của task.
    pub task_id: String,
    /// Tiêu đề của task.
    pub task_title: String,
    /// Nhà cung cấp / agent.
    pub agent: String,
    /// Tài khoản thực thi.
    pub account: String,
    /// Trạng thái của lần chạy.
    pub status: String,
    /// Thời điểm bắt đầu.
    pub started_at: String,
    /// Thời điểm kết thúc nếu có.
    pub finished_at: Option<String>,
    /// Mã thoát của tiến trình nếu có.
    pub exit_code: Option<i64>,
    /// Thời lượng tính bằng giây.
    pub duration_seconds: Option<i64>,
    /// Ghi chú người dùng gán.
    pub note: Option<String>,
    /// Danh sách nhãn người dùng gán.
    pub tags: Vec<String>,
}

/// Chi tiết một lần chạy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunDetail {
    /// Thông tin tóm tắt lần chạy.
    #[serde(flatten)]
    pub summary: RunSummary,
    /// Workspace ID sở hữu lần chạy này.
    pub workspace_id: String,
    /// Nguyên nhân kết thúc nếu có.
    pub termination_reason: Option<String>,
}

/// Một sự kiện của lần chạy phục vụ phát lại log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunEventItem {
    /// ID sự kiện.
    pub id: String,
    /// ID lần chạy.
    pub run_id: String,
    /// Thứ tự tuần tự của sự kiện.
    pub seq: i64,
    /// Loại sự kiện.
    pub event_type: String,
    /// Dữ liệu chi tiết sự kiện.
    pub data: String,
    /// Thời điểm phát sinh sự kiện.
    pub created_at: String,
}

/// Dữ liệu ghi chú và nhãn cho một lần chạy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunHistoryNote {
    /// ID của lần chạy.
    pub run_id: String,
    /// Nội dung ghi chú.
    pub note: String,
    /// Danh sách các nhãn.
    pub tags: Vec<String>,
    /// Thời điểm cập nhật cuối cùng.
    pub updated_at: String,
}

/// Payload gửi lên để cập nhật ghi chú và nhãn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNotePayload {
    /// Ghi chú người dùng nhập.
    pub note: String,
    /// Danh sách nhãn người dùng chọn.
    pub tags: Vec<String>,
}

/// Bộ lọc tham số truy vấn danh sách lần chạy.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListRunsQuery {
    /// Lọc theo task ID.
    pub task_id: Option<String>,
    /// Lọc theo agent / nhà cung cấp.
    pub agent_provider: Option<String>,
    /// Lọc theo tài khoản agent.
    pub agent_account_id: Option<String>,
    /// Lọc theo trạng thái lần chạy.
    pub status: Option<String>,
    /// Thời điểm bắt đầu từ (ISO string).
    pub from: Option<String>,
    /// Thời điểm kết thúc tới (ISO string).
    pub to: Option<String>,
    /// Giới hạn số lượng bản ghi.
    pub limit: Option<i64>,
    /// Vị trí dịch chuyển phân trang.
    pub offset: Option<i64>,
}

/// Tham số phân trang lấy danh sách sự kiện.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListEventsQuery {
    /// Giới hạn số bản ghi.
    pub limit: Option<i64>,
    /// Vị trí dịch chuyển.
    pub offset: Option<i64>,
}

/// Tham số xuất dữ liệu lần chạy.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ExportQuery {
    /// Định dạng xuất: "markdown" hoặc "json".
    pub format: Option<String>,
}

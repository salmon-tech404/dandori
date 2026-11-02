//! Mô hình dữ liệu cho module `reports`.

use serde::{Deserialize, Serialize};

/// Thống kê số lượng nhiệm vụ theo trạng thái trong kỳ.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportCounts {
    /// Số nhiệm vụ đã hoàn thành (`done`).
    pub completed: usize,
    /// Số nhiệm vụ thất bại (`failed`).
    pub failed: usize,
    /// Số nhiệm vụ cần người dùng can thiệp (`needs_human`).
    pub needs_human: usize,
    /// Số nhiệm vụ đang chạy (`in_progress` hoặc `running`).
    pub running: usize,
    /// Tổng số nhiệm vụ được cập nhật trong kỳ.
    pub total: usize,
}

/// Thống kê thời lượng các lần chạy trong kỳ.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurationStats {
    /// Tổng thời lượng chạy tính theo giây.
    pub total_duration_secs: i64,
    /// Thời lượng trung vị tính theo giây (median).
    pub median_duration_secs: i64,
    /// Thời lượng trung bình tính theo giây (average).
    pub avg_duration_secs: i64,
}

/// Thống kê sử dụng nhà cung cấp và tài khoản agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderAccountUsage {
    /// Tên nhà cung cấp / agent (`claude`, `gemini`, `codex`...).
    pub agent: String,
    /// Tài khoản được phân bổ.
    pub account: String,
    /// Số lần chạy thực hiện.
    pub run_count: usize,
}

/// Thống kê số lần chuyển tài khoản tự động (nếu có module auto-switch).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountSwitchStats {
    /// Số lần chuyển tài khoản ghi nhận được; None nếu module nguồn không có dữ liệu.
    pub switch_count: Option<usize>,
}

/// Thống kê chi phí nếu có module cost-tracking.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CostStats {
    /// Tổng chi phí ghi nhận được trong kỳ.
    pub total_cost: Option<f64>,
    /// Đơn vị tiền tệ (mặc định USD).
    pub currency: Option<String>,
    /// Cờ báo hiệu có số liệu chi phí hay không.
    pub has_data: bool,
    /// Lý do nếu thiếu số liệu ("không có số liệu chi phí" hoặc "module cost-tracking chưa kích hoạt").
    pub missing_reason: Option<String>,
}

/// Một nhiệm vụ nổi bật trong kỳ báo cáo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopTaskItem {
    /// Mã nhiệm vụ.
    pub task_id: String,
    /// Tiêu đề nhiệm vụ.
    pub title: String,
    /// Trạng thái nhiệm vụ.
    pub status: String,
    /// Thời lượng chạy tích lũy nếu có (giây).
    pub duration_secs: Option<i64>,
}

/// Toàn bộ dữ liệu tổng hợp của một kỳ báo cáo (ngày hoặc tuần).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportData {
    /// Mã không gian làm việc.
    pub workspace_id: String,
    /// Loại kỳ (`daily` hoặc `weekly`).
    pub period_type: String,
    /// Thời điểm bắt đầu kỳ (RFC 3339).
    pub period_start: String,
    /// Thời điểm kết thúc kỳ (RFC 3339).
    pub period_end: String,
    /// Múi giờ áp dụng.
    pub timezone: String,
    /// Số lượng nhiệm vụ theo trạng thái.
    pub counts: ReportCounts,
    /// Thống kê thời lượng chạy.
    pub duration: DurationStats,
    /// Thống kê sử dụng theo nhà cung cấp và tài khoản.
    pub usage: Vec<ProviderAccountUsage>,
    /// Thống kê chuyển đổi tài khoản.
    pub switches: AccountSwitchStats,
    /// Thống kê chi phí.
    pub cost: CostStats,
    /// Danh sách nhiệm vụ nổi bật.
    pub top_tasks: Vec<TopTaskItem>,
    /// Thời điểm tạo báo cáo (RFC 3339).
    pub generated_at: String,
}

/// Bản ghi cấu hình báo cáo định kỳ của workspace.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReportConfig {
    /// Mã không gian làm việc.
    pub workspace_id: String,
    /// Bật/tắt tự động chụp báo cáo (0: tắt, 1: bật).
    pub auto_generate: i64,
    /// Múi giờ tính toán.
    pub timezone: String,
    /// Giờ tạo báo cáo ngày (HH:MM).
    pub daily_time: String,
    /// Ngày trong tuần tạo báo cáo tuần (1=Mon..7=Sun).
    pub weekly_day: i64,
    /// Giờ tạo báo cáo tuần (HH:MM).
    pub weekly_time: String,
    /// Thời điểm cập nhật cấu hình.
    pub updated_at: String,
}

/// Yêu cầu cập nhật cấu hình báo cáo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportConfigRequest {
    /// Bật/tắt tự động tạo báo cáo.
    pub auto_generate: Option<bool>,
    /// Múi giờ.
    pub timezone: Option<String>,
    /// Giờ tạo báo cáo ngày "HH:MM".
    pub daily_time: Option<String>,
    /// Ngày tạo báo cáo tuần (1..7).
    pub weekly_day: Option<i64>,
    /// Giờ tạo báo cáo tuần "HH:MM".
    pub weekly_time: Option<String>,
}

/// Bản ghi bản chụp báo cáo lưu trữ trong cơ sở dữ liệu.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReportSnapshot {
    /// Định danh bản chụp.
    pub id: String,
    /// Không gian làm việc.
    pub workspace_id: String,
    /// Loại kỳ (`daily` hoặc `weekly`).
    pub period_type: String,
    /// Thời điểm bắt đầu kỳ.
    pub period_start: String,
    /// Thời điểm kết thúc kỳ.
    pub period_end: String,
    /// Múi giờ.
    pub timezone: String,
    /// Dữ liệu JSON chi tiết.
    pub data_json: String,
    /// Nội dung định dạng Markdown.
    pub markdown: String,
    /// Thời điểm tạo.
    pub created_at: String,
}

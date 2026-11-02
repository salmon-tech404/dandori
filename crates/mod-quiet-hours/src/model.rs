//! Mô hình dữ liệu cho module `quiet-hours`.

use serde::{Deserialize, Serialize};

/// Cấu hình một khung giờ cấm chạy.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuietWindow {
    /// Định danh duy nhất.
    pub id: String,
    /// Không gian làm việc sở hữu.
    pub workspace_id: String,
    /// Dự án áp dụng (nếu None thì áp dụng cho toàn workspace).
    pub project_id: Option<String>,
    /// Tên mô tả khung giờ (ví dụ: "Giờ làm việc", "Khung cấm ban đêm").
    pub name: String,
    /// Các ngày trong tuần áp dụng (chuỗi phân cách dấu phẩy, ví dụ "1,2,3,4,5" với 1=T2..7=CN).
    pub days_of_week: String,
    /// Thời điểm bắt đầu định dạng "HH:MM".
    pub start_time: String,
    /// Thời điểm kết thúc định dạng "HH:MM".
    pub end_time: String,
    /// Múi giờ áp dụng (ví dụ "UTC", "+07:00", "-05:00").
    pub timezone: String,
    /// Chặn cả chạy tay thủ công do người dùng khởi động (mặc định false).
    #[sqlx(try_from = "i64")]
    pub block_manual_runs: bool,
    /// Khung giờ đang được kích hoạt hay không.
    #[sqlx(try_from = "i64")]
    pub enabled: bool,
    /// Thời điểm tạo bản ghi (RFC 3339).
    pub created_at: String,
    /// Thời điểm cập nhật bản ghi gần nhất (RFC 3339).
    pub updated_at: String,
}

/// Dữ liệu yêu cầu tạo mới một khung giờ cấm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuietWindowDto {
    /// Không gian làm việc.
    pub workspace_id: String,
    /// Dự án áp dụng tùy chọn.
    pub project_id: Option<String>,
    /// Tên khung giờ.
    pub name: String,
    /// Các ngày trong tuần áp dụng (1..7).
    pub days_of_week: String,
    /// Giờ bắt đầu "HH:MM".
    pub start_time: String,
    /// Giờ kết thúc "HH:MM".
    pub end_time: String,
    /// Múi giờ (mặc định UTC).
    pub timezone: Option<String>,
    /// Chặn cả chạy tay (mặc định false).
    pub block_manual_runs: Option<bool>,
}

/// Dữ liệu yêu cầu cập nhật khung giờ cấm.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateQuietWindowDto {
    /// Tên khung giờ mới.
    pub name: Option<String>,
    /// Các ngày trong tuần áp dụng mới.
    pub days_of_week: Option<String>,
    /// Giờ bắt đầu "HH:MM".
    pub start_time: Option<String>,
    /// Giờ kết thúc "HH:MM".
    pub end_time: Option<String>,
    /// Múi giờ mới.
    pub timezone: Option<String>,
    /// Chặn cả chạy tay mới.
    pub block_manual_runs: Option<bool>,
    /// Trạng thái bật/tắt mới.
    pub enabled: Option<bool>,
}

/// Trạng thái kiểm tra thời điểm hiện tại đối với khung giờ cấm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietStatusDto {
    /// Hiện tại có đang nằm trong khung giờ cấm hay không.
    pub in_quiet_window: bool,
    /// Danh sách tên các khung giờ cấm đang có hiệu lực tại thời điểm này.
    pub active_window_names: Vec<String>,
    /// Thời điểm hết khung giờ cấm (có thể thử lại tác vụ), định dạng RFC 3339.
    pub retry_after: Option<String>,
    /// Khung giờ cấm này có chặn cả các lượt chạy thủ công của người dùng hay không.
    pub blocks_manual_runs: bool,
}

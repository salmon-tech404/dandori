//! Các cấu trúc dữ liệu cho module nhật ký kiểm toán (P3-14).

use serde::{Deserialize, Serialize};

/// Genesis Hash khởi nguyên chuẩn SHA-256 (64 ký tự 0) cho bản ghi đầu tiên của mỗi workspace.
pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Bản ghi nhật ký kiểm toán trong cơ sở dữ liệu.
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLogEntry {
    /// Định danh duy nhất của bản ghi.
    pub id: String,
    /// Định danh workspace.
    pub workspace_id: String,
    /// Số thứ tự tuần tự trong chuỗi băm.
    pub sequence: i64,
    /// Định danh tác nhân thực hiện.
    pub actor_id: String,
    /// Tên hiển thị của tác nhân.
    pub actor_name: String,
    /// Tên hành động nhạy cảm.
    pub action: String,
    /// Loại đối tượng tác động.
    pub target_type: String,
    /// Định danh đối tượng tác động.
    pub target_id: String,
    /// Dữ liệu chi tiết dạng JSON đã làm sạch bí mật.
    pub details: String,
    /// Nguồn gốc yêu cầu (web/mobile/scheduler/system).
    pub source: String,
    /// Băm SHA-256 của bản ghi trước đó.
    pub prev_hash: String,
    /// Băm SHA-256 của bản ghi hiện tại.
    pub hash: String,
    /// Thời điểm tạo bản ghi chuẩn ISO 8601.
    pub created_at: String,
}

/// Yêu cầu thêm bản ghi kiểm toán mới.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppendLogRequest {
    /// Định danh tác nhân (API bỏ qua giá trị này và dùng người dùng của phiên).
    #[serde(default)]
    pub actor_id: String,
    /// Tên tác nhân (API bỏ qua giá trị này).
    #[serde(default)]
    pub actor_name: String,
    /// Tên hành động.
    pub action: String,
    /// Loại đối tượng.
    pub target_type: String,
    /// Định danh đối tượng.
    pub target_id: String,
    /// Chi tiết hành động.
    #[serde(default)]
    pub details: serde_json::Value,
    /// Nguồn gốc hành động.
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "web".to_string()
}

/// Kết quả kiểm tra tính toàn vẹn của chuỗi băm.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifyResult {
    /// Chuỗi có toàn vẹn hay không.
    pub is_valid: bool,
    /// Tổng số bản ghi đã kiểm tra.
    pub records_checked: usize,
    /// Định danh bản ghi bị đứt chuỗi nếu có.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broken_at_id: Option<String>,
    /// Số thứ tự bản ghi bị đứt chuỗi nếu có.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broken_at_sequence: Option<i64>,
    /// Lý do đứt chuỗi nếu có.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Tham số lọc danh sách nhật ký.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ListLogsQuery {
    /// Giới hạn số lượng bản ghi trả về.
    pub limit: Option<i64>,
    /// Vị trí bắt đầu lấy bản ghi.
    pub offset: Option<i64>,
    /// Lọc theo tên hành động.
    pub action: Option<String>,
    /// Từ khóa tìm kiếm.
    pub search: Option<String>,
}

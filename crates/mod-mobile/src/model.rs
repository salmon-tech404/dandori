//! Các kiểu dữ liệu và mô hình CSDL cho module mobile (P3-13).

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Bản ghi thiết bị di động trong CSDL.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MobileDevice {
    /// Định danh duy nhất của thiết bị.
    pub id: String,
    /// ID workspace quản lý thiết bị.
    pub workspace_id: String,
    /// Tên gọi gợi nhớ của thiết bị.
    pub name: String,
    /// Chuỗi băm SHA-256 của token ghép nối (không bao giờ lưu bản rõ).
    pub token_hash: String,
    /// ID người dùng quản trị đã tạo thiết bị.
    pub created_by: String,
    /// Chế độ chỉ xem (không thể duyệt/từ chối).
    pub read_only: bool,
    /// Cờ thu hồi token ngay lập tức.
    pub is_revoked: bool,
    /// Thời điểm hết hạn token (RFC 3339).
    pub expires_at: String,
    /// Thời điểm sử dụng gần nhất.
    pub last_used_at: Option<String>,
    /// Thời điểm tạo.
    pub created_at: String,
    /// Thời điểm cập nhật gần nhất.
    pub updated_at: String,
}

/// Dữ liệu công khai của thiết bị di động (không chứa token hay token_hash).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicMobileDevice {
    /// Định danh duy nhất của thiết bị.
    pub id: String,
    /// ID workspace quản lý thiết bị.
    pub workspace_id: String,
    /// Tên gọi gợi nhớ của thiết bị.
    pub name: String,
    /// ID người dùng quản trị đã tạo thiết bị.
    pub created_by: String,
    /// Chế độ chỉ xem.
    pub read_only: bool,
    /// Cờ thu hồi token.
    pub is_revoked: bool,
    /// Thời điểm hết hạn token.
    pub expires_at: String,
    /// Thời điểm sử dụng gần nhất.
    pub last_used_at: Option<String>,
    /// Thời điểm tạo.
    pub created_at: String,
}

impl From<MobileDevice> for PublicMobileDevice {
    fn from(d: MobileDevice) -> Self {
        Self {
            id: d.id,
            workspace_id: d.workspace_id,
            name: d.name,
            created_by: d.created_by,
            read_only: d.read_only,
            is_revoked: d.is_revoked,
            expires_at: d.expires_at,
            last_used_at: d.last_used_at,
            created_at: d.created_at,
        }
    }
}

/// Yêu cầu tạo thiết bị di động mới.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateDeviceRequest {
    /// Tên thiết bị.
    pub name: String,
    /// Tùy chọn chế độ chỉ xem (mặc định false).
    pub read_only: Option<bool>,
    /// Thời gian sống của token tính bằng giờ (mặc định 720 = 30 ngày).
    pub ttl_hours: Option<i64>,
}

/// Kết quả tạo thiết bị di động mới (trả về token thô đúng 1 lần duy nhất).
#[derive(Debug, Clone, Serialize)]
pub struct CreateDeviceResponse {
    /// Thông tin thiết bị.
    pub device: PublicMobileDevice,
    /// Token ghép nối thô (chỉ hiển thị một lần duy nhất lúc tạo).
    pub token: String,
}

/// Cấu hình truy cập di động theo workspace.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MobileConfig {
    /// ID workspace.
    pub workspace_id: String,
    /// Danh sách các Origin bổ sung được phép (JSON string trong DB).
    pub allowed_origins: String,
    /// Cờ áp dụng chế độ chỉ xem cho toàn bộ thiết bị.
    pub read_only: bool,
    /// Thời điểm cập nhật.
    pub updated_at: String,
}

/// Cấu hình công khai trả về cho client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicMobileConfig {
    /// ID workspace.
    pub workspace_id: String,
    /// Danh sách các Origin được phép truy cập từ xa.
    pub allowed_origins: Vec<String>,
    /// Chế độ chỉ xem.
    pub read_only: bool,
}

/// Yêu cầu cập nhật cấu hình truy cập di động.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateMobileConfigRequest {
    /// Danh sách Origin cho phép mới.
    pub allowed_origins: Option<Vec<String>>,
    /// Chế độ chỉ xem.
    pub read_only: Option<bool>,
}

/// Tóm tắt trạng thái hệ thống cho trang di động.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileStatusResponse {
    /// Tổng số task đang chờ duyệt / cần người can thiệp.
    pub pending_approvals_count: i64,
    /// Tổng số task đang chạy trong workspace.
    pub active_runs_count: i64,
    /// Trạng thái thiết bị hiện tại (chỉ xem hay có quyền duyệt).
    pub is_read_only: bool,
    /// Tên thiết bị di động đang truy cập.
    pub device_name: String,
}

/// Mục yêu cầu phê duyệt hiển thị trên di động.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileApprovalItem {
    /// ID yêu cầu phê duyệt hoặc ID task.
    pub id: String,
    /// ID task tương ứng.
    pub task_id: String,
    /// Tiêu đề task.
    pub task_title: String,
    /// Lý do yêu cầu phê duyệt.
    pub reason: String,
    /// Trạng thái task hoặc yêu cầu (needs_human, in_review, pending).
    pub status: String,
    /// Thời điểm tạo.
    pub created_at: String,
}

/// Quyết định phê duyệt từ thiết bị di động.
#[derive(Debug, Clone, Deserialize)]
pub struct MobileDecisionRequest {
    /// Hành động: `approve` hoặc `reject`.
    pub action: String,
    /// Ghi chú / lý do của người duyệt.
    pub comment: Option<String>,
}

/// Bản ghi nhật ký hành động của thiết bị di động.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MobileActionLog {
    /// ID bản ghi.
    pub id: String,
    /// ID workspace.
    pub workspace_id: String,
    /// ID thiết bị thực hiện.
    pub device_id: String,
    /// Hành động thực hiện (approve, reject).
    pub action: String,
    /// ID đối tượng tác động (approval_id hoặc task_id).
    pub target_id: Option<String>,
    /// Chi tiết hành động.
    pub detail: Option<String>,
    /// Thời điểm ghi nhận.
    pub created_at: String,
}

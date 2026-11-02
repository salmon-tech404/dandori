//! Các cấu trúc dữ liệu và DTO cho module `approval-gate` (P2-03).

use serde::{Deserialize, Serialize};

/// Chính sách phê duyệt chuyển trạng thái của dự án.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApprovalPolicy {
    /// ID workspace.
    pub workspace_id: String,
    /// ID dự án.
    pub project_id: String,
    /// Có bắt buộc duyệt khi chuyển từ in_review sang ready_to_merge không.
    pub require_in_review_to_merge: bool,
    /// Có cấm người yêu cầu tự duyệt không.
    pub disallow_self_approval: bool,
    /// Số giờ trước khi yêu cầu hết hạn.
    pub expiry_hours: i64,
}

impl Default for ApprovalPolicy {
    fn default() -> Self {
        Self {
            workspace_id: String::new(),
            project_id: String::new(),
            require_in_review_to_merge: true,
            disallow_self_approval: true,
            expiry_hours: 24,
        }
    }
}

/// Trạng thái của một yêu cầu phê duyệt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    /// Đang chờ phê duyệt.
    Pending,
    /// Đã được chấp thuận.
    Approved,
    /// Đã bị từ chối.
    Rejected,
    /// Đã hết hạn.
    Expired,
}

impl ApprovalStatus {
    /// Trả về chuỗi đại diện của trạng thái.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
        }
    }
}

/// Một bản ghi yêu cầu phê duyệt trong cơ sở dữ liệu.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApprovalRequest {
    /// ID yêu cầu phê duyệt.
    pub id: String,
    /// ID workspace.
    pub workspace_id: String,
    /// ID dự án.
    pub project_id: String,
    /// ID nhiệm vụ.
    pub task_id: String,
    /// Loại cổng phê duyệt.
    pub kind: String,
    /// Trạng thái nguồn trước khi chuyển đổi.
    pub from_status: String,
    /// Trạng thái đích mong muốn.
    pub target_status: String,
    /// ID người hoặc hệ thống gửi yêu cầu.
    pub requester_id: String,
    /// Trạng thái yêu cầu (`pending`, `approved`, `rejected`, `expired`).
    pub status: String,
    /// Lý do yêu cầu phê duyệt.
    pub reason: Option<String>,
    /// ID người đưa ra quyết định.
    pub decider_id: Option<String>,
    /// Thời điểm tạo yêu cầu (ISO 8601 UTC).
    pub created_at: String,
    /// Thời điểm đưa ra quyết định (ISO 8601 UTC).
    pub decided_at: Option<String>,
    /// Thời điểm hết hạn (ISO 8601 UTC).
    pub expires_at: String,
    /// Lý do từ chối (nếu bị từ chối).
    pub rejection_reason: Option<String>,
}

/// DTO yêu cầu quyết định phê duyệt hoặc từ chối.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecideRequest {
    /// Lý do quyết định hoặc lý do từ chối.
    pub reason: Option<String>,
}

/// DTO cập nhật chính sách phê duyệt.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdatePolicyRequest {
    /// Bắt buộc duyệt khi chuyển từ in_review sang ready_to_merge.
    pub require_in_review_to_merge: Option<bool>,
    /// Cấm người yêu cầu tự duyệt.
    pub disallow_self_approval: Option<bool>,
    /// Số giờ hết hạn.
    pub expiry_hours: Option<i64>,
}

/// Tham số lọc danh sách yêu cầu phê duyệt.
#[derive(Debug, Clone, Deserialize)]
pub struct ListRequestsQuery {
    /// Lọc theo ID workspace.
    pub workspace_id: Option<String>,
    /// Lọc theo ID dự án.
    pub project_id: Option<String>,
    /// Lọc theo ID nhiệm vụ.
    pub task_id: Option<String>,
    /// Lọc theo trạng thái (`pending`, `approved`, `rejected`, `expired`).
    pub status: Option<String>,
}

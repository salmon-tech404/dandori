//! Các kiểu dữ liệu của Feature Registry (01_KIEN_TRUC.md mục 5).

use serde::{Deserialize, Serialize};

/// Cấp phạm vi quản lý của module.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegistryScope {
    /// Cấp toàn hệ thống
    System,
    /// Cấp workspace
    Workspace(String),
    /// Cấp dự án trực thuộc workspace
    Project {
        /// ID workspace chứa dự án
        workspace_id: String,
        /// ID dự án
        project_id: String,
    },
}

impl RegistryScope {
    /// Tên loại phạm vi ("system", "workspace", "project").
    #[must_use]
    pub fn scope_kind(&self) -> &str {
        match self {
            Self::System => "system",
            Self::Workspace(_) => "workspace",
            Self::Project { .. } => "project",
        }
    }

    /// Định danh cụ thể của phạm vi ("system", ID workspace hoặc ID dự án).
    #[must_use]
    pub fn scope_id(&self) -> &str {
        match self {
            Self::System => "system",
            Self::Workspace(ws_id) => ws_id,
            Self::Project { project_id, .. } => project_id,
        }
    }
}

/// Vai trò phân quyền của người dùng đối với các cấp phạm vi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RegistryRole {
    /// Thành viên thông thường (không có quyền đổi cấu hình tính năng)
    Member = 0,
    /// Chủ dự án (quản lý phạm vi cấp dự án)
    ProjectOwner = 1,
    /// Quản trị viên workspace (quản lý phạm vi workspace và dự án con)
    WorkspaceAdmin = 2,
    /// Chủ sở hữu hệ thống (toàn quyền trên mọi phạm vi)
    SystemOwner = 3,
}

impl RegistryRole {
    /// Kiểm tra người dùng có đủ thẩm quyền quản lý phạm vi hay không.
    #[must_use]
    pub fn can_manage_scope(&self, scope: &RegistryScope) -> bool {
        match scope {
            RegistryScope::System => *self >= Self::SystemOwner,
            RegistryScope::Workspace(_) => *self >= Self::WorkspaceAdmin,
            RegistryScope::Project { .. } => *self >= Self::ProjectOwner,
        }
    }

    /// Tên chuỗi đại diện của vai trò.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Member => "Member",
            Self::ProjectOwner => "ProjectOwner",
            Self::WorkspaceAdmin => "WorkspaceAdmin",
            Self::SystemOwner => "SystemOwner",
        }
    }
}

/// Trạng thái hiệu lực được tính toán động của một module tại ngữ cảnh cụ thể (không lưu DB).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectiveState {
    /// ID module
    pub module_id: String,
    /// Trạng thái bật hoặc tắt
    pub enabled: bool,
    /// Nguồn gốc quyết định trạng thái ("system", "workspace", "project", "default")
    pub source_scope: String,
    /// Bị khóa cứng từ cấp trên
    pub locked: bool,
    /// Cấu hình hiệu lực của module
    pub config: Option<serde_json::Value>,
}

/// Kế hoạch kích hoạt module cùng danh sách phụ thuộc bắc cầu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnablePlan {
    /// ID module mục tiêu
    pub module_id: String,
    /// Danh sách các module sẽ được bật kèm (sắp xếp theo thứ tự topo từ gốc đến ngọn)
    pub will_enable_with: Vec<String>,
    /// Có cần xác nhận rủi ro từ người dùng hay không
    pub needs_risk_ack: bool,
    /// Ghi chú rủi ro cảnh báo cho người dùng
    pub risk_note: Option<String>,
}

/// Kế hoạch vô hiệu hóa module cùng danh sách các module bị ảnh hưởng.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisablePlan {
    /// ID module mục tiêu muốn tắt
    pub module_id: String,
    /// Danh sách các module đang bật khác sẽ bị mất chức năng vì cần nó
    pub will_break: Vec<String>,
}

/// Yêu cầu thay đổi trạng thái của module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeRequest {
    /// ID module
    pub module_id: String,
    /// Phạm vi áp dụng
    pub scope: RegistryScope,
    /// Bật hoặc tắt
    pub enabled: bool,
    /// Khóa trạng thái đối với cấp dưới
    pub locked: bool,
    /// Cấu hình cập nhật (nếu có)
    pub config: Option<serde_json::Value>,
    /// Lý do thay đổi (ghi vào audit)
    pub reason: String,
    /// Người thực hiện
    pub performed_by: String,
    /// Vai trò của người thực hiện
    pub actor_role: RegistryRole,
    /// Đã xác nhận cảnh báo rủi ro (đối với module risk != low)
    pub risk_acknowledged: bool,
}

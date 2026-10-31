//! Kiểu dữ liệu dùng chung và định danh newtype của Dandori.

#![warn(missing_docs)]

pub mod entity;
pub mod event;
pub mod id;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use entity::{
    AgentAccount, FeatureAudit, FeatureModule, FeaturePack, FeatureSetting, Project, Run, RunEvent,
    SecretRecord, Session, Task, TaskStatus, User, Workspace, WorkspaceMember,
};
pub use event::{Envelope, EventKind, EventPayload, EventScope};
pub use id::{
    AgentAccountId, FeatureAuditId, FeatureModuleId, FeaturePackId, ProjectId, RunEventId, RunId,
    SecretId, SessionId, TaskId, UserId, WorkspaceId,
};

/// Phiên bản của crate types.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Trạng thái hoạt động tổng thể của hệ thống Dandori.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SystemStatus {
    /// Hệ thống đang khởi tạo các dịch vụ nền
    Initializing,
    /// Hệ thống sẵn sàng tiếp nhận điều phối agent
    Ready,
    /// Hệ thống đang hoạt động ở chế độ suy giảm hiệu năng
    Degraded,
    /// Hệ thống đang dừng hoặc bảo trì
    Stopped,
}

fn append_ts_decl(out: &mut String, decl: &str) {
    if decl.starts_with("type ") || decl.starts_with("interface ") {
        out.push_str("export ");
    }
    out.push_str(decl);
    out.push_str("\n\n");
}

/// Sinh toàn bộ định nghĩa TypeScript từ các kiểu dữ liệu của Dandori.
#[must_use]
pub fn generate_typescript_types() -> String {
    let mut out = String::new();
    out.push_str("// TỆP SINH TỰ ĐỘNG bởi generate_types (RS-TS-001); không sửa tay.\n\n");

    append_ts_decl(&mut out, &SystemStatus::decl());

    append_ts_decl(&mut out, &UserId::decl());
    append_ts_decl(&mut out, &SessionId::decl());
    append_ts_decl(&mut out, &WorkspaceId::decl());
    append_ts_decl(&mut out, &ProjectId::decl());
    append_ts_decl(&mut out, &TaskId::decl());
    append_ts_decl(&mut out, &RunId::decl());
    append_ts_decl(&mut out, &RunEventId::decl());
    append_ts_decl(&mut out, &AgentAccountId::decl());
    append_ts_decl(&mut out, &SecretId::decl());
    append_ts_decl(&mut out, &FeatureAuditId::decl());
    append_ts_decl(&mut out, &FeatureModuleId::decl());
    append_ts_decl(&mut out, &FeaturePackId::decl());

    append_ts_decl(&mut out, &User::decl());
    append_ts_decl(&mut out, &Session::decl());
    append_ts_decl(&mut out, &Workspace::decl());
    append_ts_decl(&mut out, &WorkspaceMember::decl());
    append_ts_decl(&mut out, &Project::decl());
    append_ts_decl(&mut out, &TaskStatus::decl());
    append_ts_decl(&mut out, &Task::decl());
    append_ts_decl(&mut out, &Run::decl());
    append_ts_decl(&mut out, &RunEvent::decl());
    append_ts_decl(&mut out, &SecretRecord::decl());
    append_ts_decl(&mut out, &AgentAccount::decl());
    append_ts_decl(&mut out, &FeatureModule::decl());
    append_ts_decl(&mut out, &FeatureSetting::decl());
    append_ts_decl(&mut out, &FeaturePack::decl());
    append_ts_decl(&mut out, &FeatureAudit::decl());

    append_ts_decl(&mut out, &EventKind::decl());
    append_ts_decl(&mut out, &EventScope::decl());
    append_ts_decl(&mut out, &EventPayload::decl());
    append_ts_decl(&mut out, &Envelope::decl());

    out.trim_end().to_string() + "\n"
}

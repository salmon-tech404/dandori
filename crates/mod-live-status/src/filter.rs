//! Bộ lọc sự kiện và xác thực loại sự kiện cho module live-status.

use dandori_types::{Envelope, EventKind, WorkspaceId};

/// Danh sách các loại sự kiện được phép gửi qua luồng SSE (run.*, task.*, module.*).
#[must_use]
pub fn allowed_event_kinds() -> Vec<EventKind> {
    vec![
        EventKind::RunStarted,
        EventKind::RunOutput,
        EventKind::RunFinished,
        EventKind::RunFailed,
        EventKind::TaskCreated,
        EventKind::TaskStatusChanged,
        EventKind::TaskAssigned,
        EventKind::ModuleEnabled,
        EventKind::ModuleDisabled,
    ]
}

/// Kiểm tra xem một loại sự kiện có nằm trong danh sách cho phép hay không.
#[must_use]
pub fn is_allowed_kind(kind: &EventKind) -> bool {
    allowed_event_kinds().contains(kind)
}

/// Chuyển đổi EventKind thành chuỗi snake_case cho trường event trong SSE.
#[must_use]
pub fn event_kind_to_string(kind: &EventKind) -> &'static str {
    match kind {
        EventKind::RunStarted => "run_started",
        EventKind::RunOutput => "run_output",
        EventKind::RunFinished => "run_finished",
        EventKind::RunFailed => "run_failed",
        EventKind::TaskCreated => "task_created",
        EventKind::TaskStatusChanged => "task_status_changed",
        EventKind::TaskAssigned => "task_assigned",
        EventKind::ModuleEnabled => "module_enabled",
        EventKind::ModuleDisabled => "module_disabled",
        _ => "unknown",
    }
}

/// Phạm vi workspace mà một kết nối SSE được phép nhận sự kiện.
#[derive(Debug, Clone)]
pub enum WorkspaceScope {
    /// Mọi workspace (chỉ dành cho quản trị hệ thống).
    All,
    /// Chỉ các workspace này.
    Only(Vec<WorkspaceId>),
}

/// Kiểm tra sự kiện có thuộc phạm vi được phép xem hay không.
#[must_use]
pub fn matches_scope(envelope: &Envelope, scope: &WorkspaceScope) -> bool {
    match scope {
        WorkspaceScope::All => true,
        WorkspaceScope::Only(allowed) => match &envelope.scope.workspace_id {
            Some(env_ws) => allowed.contains(env_ws),
            None => matches!(
                envelope.kind,
                EventKind::ModuleEnabled | EventKind::ModuleDisabled
            ),
        },
    }
}

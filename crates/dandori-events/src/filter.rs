//! Bộ lọc sự kiện theo loại và phạm vi không gian làm việc/dự án.

use dandori_types::{Envelope, EventKind, ProjectId, RunId, TaskId, WorkspaceId};

/// Bộ lọc điều kiện nhận sự kiện từ EventBus.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventFilter {
    /// Danh sách các loại sự kiện quan tâm (None = nhận mọi loại).
    pub kinds: Option<Vec<EventKind>>,
    /// Lọc theo không gian làm việc cụ thể.
    pub workspace_id: Option<WorkspaceId>,
    /// Lọc theo dự án cụ thể.
    pub project_id: Option<ProjectId>,
    /// Lọc theo task cụ thể.
    pub task_id: Option<TaskId>,
    /// Lọc theo lần chạy cụ thể.
    pub run_id: Option<RunId>,
}

impl EventFilter {
    /// Tạo bộ lọc nhận toàn bộ mọi sự kiện không giới hạn.
    #[must_use]
    pub fn all() -> Self {
        Self::default()
    }

    /// Lọc sự kiện thuộc một không gian làm việc.
    #[must_use]
    pub fn for_workspace(ws_id: WorkspaceId) -> Self {
        Self {
            workspace_id: Some(ws_id),
            ..Default::default()
        }
    }

    /// Lọc sự kiện thuộc một dự án cụ thể.
    #[must_use]
    pub fn for_project(proj_id: ProjectId) -> Self {
        Self {
            project_id: Some(proj_id),
            ..Default::default()
        }
    }

    /// Lọc sự kiện thuộc một task cụ thể.
    #[must_use]
    pub fn for_task(task_id: TaskId) -> Self {
        Self {
            task_id: Some(task_id),
            ..Default::default()
        }
    }

    /// Lọc sự kiện theo danh mục loại sự kiện.
    #[must_use]
    pub fn for_kinds(kinds: Vec<EventKind>) -> Self {
        Self {
            kinds: Some(kinds),
            ..Default::default()
        }
    }

    /// Kiểm tra một bao thư sự kiện có thỏa mãn điều kiện lọc hay không.
    #[must_use]
    pub fn matches(&self, env: &Envelope) -> bool {
        if self.kinds.as_ref().is_some_and(|k| !k.contains(&env.kind)) {
            return false;
        }

        if self.workspace_id.is_some() && env.scope.workspace_id != self.workspace_id {
            return false;
        }

        if self.project_id.is_some() && env.scope.project_id != self.project_id {
            return false;
        }

        if self.task_id.is_some() && env.scope.task_id != self.task_id {
            return false;
        }

        if self.run_id.is_some() && env.scope.run_id != self.run_id {
            return false;
        }

        true
    }
}

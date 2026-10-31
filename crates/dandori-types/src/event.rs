//! Các kiểu dữ liệu sự kiện trong tiến trình của Dandori (01_KIEN_TRUC.md mục 7).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::id::{
    AgentAccountId, FeatureAuditId, FeatureModuleId, ProjectId, RunId, TaskId, WorkspaceId,
};

/// Các loại sự kiện lõi trong hệ thống Dandori (01_KIEN_TRUC.md mục 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// Module tính năng được kích hoạt.
    ModuleEnabled,
    /// Module tính năng bị vô hiệu hóa.
    ModuleDisabled,
    /// Task mới được tạo.
    TaskCreated,
    /// Trạng thái task thay đổi.
    TaskStatusChanged,
    /// Task được giao cho agent.
    TaskAssigned,
    /// Lần chạy agent bắt đầu.
    RunStarted,
    /// Luồng xuất dữ liệu/log của lần chạy.
    RunOutput,
    /// Lần chạy kết thúc thành công hoặc hoàn thành.
    RunFinished,
    /// Lần chạy thất bại do lỗi.
    RunFailed,
    /// Yêu cầu phê duyệt từ cổng chuyển trạng thái.
    ApprovalRequested,
    /// Quyết định phê duyệt đã được đưa ra.
    ApprovalDecided,
    /// Hạn mức tài khoản agent được cập nhật.
    AccountQuotaUpdated,
    /// Yêu cầu phát thông báo tới người dùng.
    NotificationRequested,
    /// Bản ghi nhật ký kiểm toán tính năng được ghi.
    AuditRecorded,
    /// Tiến trình worker nền của module gặp lỗi.
    WorkerFailed,
}

/// Phạm vi gắn liền với một sự kiện.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EventScope {
    /// Định danh không gian làm việc (nếu có).
    pub workspace_id: Option<WorkspaceId>,
    /// Định danh dự án (nếu có).
    pub project_id: Option<ProjectId>,
    /// Định danh task (nếu có).
    pub task_id: Option<TaskId>,
    /// Định danh lần chạy agent (nếu có).
    pub run_id: Option<RunId>,
}

impl EventScope {
    /// Tạo phạm vi trống không giới hạn.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Tạo phạm vi chỉ gắn với một không gian làm việc.
    #[must_use]
    pub fn workspace(ws_id: WorkspaceId) -> Self {
        Self {
            workspace_id: Some(ws_id),
            ..Default::default()
        }
    }

    /// Tạo phạm vi gắn với dự án và workspace.
    #[must_use]
    pub fn project(ws_id: WorkspaceId, proj_id: ProjectId) -> Self {
        Self {
            workspace_id: Some(ws_id),
            project_id: Some(proj_id),
            ..Default::default()
        }
    }

    /// Tạo phạm vi gắn với task, project và workspace.
    #[must_use]
    pub fn task(ws_id: WorkspaceId, proj_id: ProjectId, task_id: TaskId) -> Self {
        Self {
            workspace_id: Some(ws_id),
            project_id: Some(proj_id),
            task_id: Some(task_id),
            ..Default::default()
        }
    }
}

/// Dữ liệu chi tiết của từng loại sự kiện lõi (chặt chẽ về kiểu, không chứa bí mật nguyên văn).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum EventPayload {
    /// Dữ liệu kích hoạt module.
    ModuleEnabled {
        /// ID module được bật.
        module_id: FeatureModuleId,
    },
    /// Dữ liệu tắt module.
    ModuleDisabled {
        /// ID module bị tắt.
        module_id: FeatureModuleId,
    },
    /// Dữ liệu tạo task.
    TaskCreated {
        /// ID task được tạo.
        task_id: TaskId,
        /// Tiêu đề task.
        title: String,
    },
    /// Dữ liệu chuyển trạng thái task.
    TaskStatusChanged {
        /// ID task.
        task_id: TaskId,
        /// Trạng thái trước đó.
        old_status: String,
        /// Trạng thái mới.
        new_status: String,
    },
    /// Dữ liệu giao task cho agent.
    TaskAssigned {
        /// ID task.
        task_id: TaskId,
        /// Tên agent được phân bổ.
        agent: String,
    },
    /// Dữ liệu khởi động lần chạy.
    RunStarted {
        /// ID lần chạy.
        run_id: RunId,
        /// Tên agent.
        agent: String,
        /// Nhãn tài khoản agent.
        account: String,
    },
    /// Dữ liệu xuất log dòng/chunk.
    RunOutput {
        /// ID lần chạy.
        run_id: RunId,
        /// Thứ tự tuần tự.
        seq: i64,
        /// Loại luồng (`stdout` hoặc `stderr`).
        stream: String,
        /// Nội dung log chunk.
        chunk: String,
    },
    /// Dữ liệu kết thúc lần chạy.
    RunFinished {
        /// ID lần chạy.
        run_id: RunId,
        /// Mã thoát của tiến trình (nếu có).
        exit_code: Option<i64>,
    },
    /// Dữ liệu lần chạy gặp sự cố thất bại.
    RunFailed {
        /// ID lần chạy.
        run_id: RunId,
        /// Thông điệp lỗi chi tiết.
        error_message: String,
    },
    /// Dữ liệu yêu cầu phê duyệt chuyển trạng thái task.
    ApprovalRequested {
        /// Định danh cổng phê duyệt.
        gate_id: String,
        /// ID task cần phê duyệt.
        task_id: TaskId,
        /// Lý do yêu cầu.
        reason: String,
    },
    /// Dữ liệu kết quả quyết định phê duyệt.
    ApprovalDecided {
        /// Định danh cổng phê duyệt.
        gate_id: String,
        /// ID task.
        task_id: TaskId,
        /// Được duyệt hay từ chối.
        approved: bool,
        /// Người hoặc agent đưa ra quyết định.
        decided_by: String,
    },
    /// Dữ liệu cập nhật hạn mức sử dụng của tài khoản agent.
    AccountQuotaUpdated {
        /// Định danh tài khoản agent.
        account_id: AgentAccountId,
        /// Thời điểm hết hạn mức (nếu đang bị tạm ngừng).
        exhausted_until: Option<DateTime<Utc>>,
    },
    /// Dữ liệu yêu cầu gửi thông báo cho người dùng.
    NotificationRequested {
        /// Tiêu đề thông báo.
        title: String,
        /// Nội dung chi tiết.
        message: String,
        /// Mức độ thông báo (`info`, `warning`, `error`).
        level: String,
    },
    /// Dữ liệu sự kiện kiểm toán tính năng.
    AuditRecorded {
        /// ID bản ghi kiểm toán.
        audit_id: FeatureAuditId,
        /// Hành động tác động (`enable`, `disable`...).
        action: String,
    },
    /// Dữ liệu worker nền gặp sự cố.
    WorkerFailed {
        /// Định danh worker hoặc module sở hữu.
        worker_id: String,
        /// Thông điệp lỗi.
        error_message: String,
    },
}

impl EventPayload {
    /// Trả về `EventKind` tương ứng với payload.
    #[must_use]
    pub fn kind(&self) -> EventKind {
        match self {
            Self::ModuleEnabled { .. } => EventKind::ModuleEnabled,
            Self::ModuleDisabled { .. } => EventKind::ModuleDisabled,
            Self::TaskCreated { .. } => EventKind::TaskCreated,
            Self::TaskStatusChanged { .. } => EventKind::TaskStatusChanged,
            Self::TaskAssigned { .. } => EventKind::TaskAssigned,
            Self::RunStarted { .. } => EventKind::RunStarted,
            Self::RunOutput { .. } => EventKind::RunOutput,
            Self::RunFinished { .. } => EventKind::RunFinished,
            Self::RunFailed { .. } => EventKind::RunFailed,
            Self::ApprovalRequested { .. } => EventKind::ApprovalRequested,
            Self::ApprovalDecided { .. } => EventKind::ApprovalDecided,
            Self::AccountQuotaUpdated { .. } => EventKind::AccountQuotaUpdated,
            Self::NotificationRequested { .. } => EventKind::NotificationRequested,
            Self::AuditRecorded { .. } => EventKind::AuditRecorded,
            Self::WorkerFailed { .. } => EventKind::WorkerFailed,
        }
    }
}

/// Bao thư chuẩn hóa chứa sự kiện của Dandori (Envelope theo 01_KIEN_TRUC.md mục 7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct Envelope {
    /// Định danh duy nhất của sự kiện (UUID v4).
    #[ts(type = "string")]
    pub id: Uuid,
    /// Thời điểm phát sinh sự kiện (UTC timestamp).
    pub at: DateTime<Utc>,
    /// Loại sự kiện.
    pub kind: EventKind,
    /// Phạm vi tác động của sự kiện.
    pub scope: EventScope,
    /// Dữ liệu chi tiết của sự kiện.
    pub payload: EventPayload,
}

impl Envelope {
    /// Khởi tạo bao thư sự kiện mới với UUID v4 ngẫu nhiên.
    #[must_use]
    pub fn new(at: DateTime<Utc>, scope: EventScope, payload: EventPayload) -> Self {
        let kind = payload.kind();
        Self {
            id: Uuid::new_v4(),
            at,
            kind,
            scope,
            payload,
        }
    }
}

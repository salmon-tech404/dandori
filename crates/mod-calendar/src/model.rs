//! Mô hình dữ liệu và DTO cho module `calendar`.

use serde::{Deserialize, Serialize};

/// Hành động khi lịch kích hoạt.
///
/// THEO TIÊU CHÍ NGHIỆM THU AC 5:
/// Tuyệt đối cấm hành động chạy lệnh; danh sách enum chỉ có MỘT biến thể duy nhất là tạo task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CalendarAction {
    /// Tạo nhiệm vụ mới từ mẫu.
    #[serde(rename = "create_task")]
    CreateTask {
        /// Tiêu đề nhiệm vụ.
        title: String,
        /// Mô tả nhiệm vụ.
        description: String,
        /// Agent trưởng nhóm (lead).
        lead_agent: String,
        /// Agent thực thi (worker).
        worker_agent: String,
    },
}

/// Mẫu nhiệm vụ được sử dụng khi kích hoạt lịch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskTemplateDto {
    /// Tiêu đề nhiệm vụ.
    pub title: String,
    /// Mô tả chi tiết nhiệm vụ.
    pub description: String,
    /// Agent trưởng nhóm (lead).
    pub lead_agent: String,
    /// Agent thực thi nhiệm vụ (worker).
    pub worker_agent: String,
}

/// Bản ghi lịch định kỳ trong cơ sở dữ liệu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScheduleRecord {
    /// Định danh duy nhất của lịch.
    pub id: String,
    /// Định danh workspace sở hữu.
    pub workspace_id: String,
    /// Định danh dự án gắn với lịch.
    pub project_id: String,
    /// Tên hiển thị của lịch.
    pub title: String,
    /// Loại lịch: "cron" hoặc "interval".
    pub schedule_type: String,
    /// Biểu thức cron hoặc số giây/phút khoảng đều.
    pub schedule_expr: String,
    /// Múi giờ áp dụng (mặc định "UTC").
    pub timezone: String,
    /// Dữ liệu mẫu nhiệm vụ (dưới dạng chuỗi JSON).
    pub task_template: String,
    /// Tự động giao việc sau khi tạo (1: có, 0: không).
    pub auto_dispatch: i64,
    /// Chính sách khi bị lỡ lượt: "skip" hoặc "run_once".
    pub missed_policy: String,
    /// Trạng thái kích hoạt (1: đang bật, 0: tạm dừng).
    pub enabled: i64,
    /// Thời điểm kích hoạt gần nhất (RFC3339).
    pub last_fire: Option<String>,
    /// Thời điểm kích hoạt dự kiến kế tiếp (RFC3339).
    pub next_fire: Option<String>,
    /// Thời điểm tạo lịch (RFC3339).
    pub created_at: String,
    /// Thời điểm cập nhật gần nhất (RFC3339).
    pub updated_at: String,
}

/// DTO chi tiết lịch trả về cho client / API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleDto {
    /// Định danh duy nhất của lịch.
    pub id: String,
    /// Định danh workspace sở hữu.
    pub workspace_id: String,
    /// Định danh dự án.
    pub project_id: String,
    /// Tên hiển thị của lịch.
    pub title: String,
    /// Loại lịch: "cron" hoặc "interval".
    pub schedule_type: String,
    /// Biểu thức cron hoặc số giây.
    pub schedule_expr: String,
    /// Múi giờ áp dụng.
    pub timezone: String,
    /// Mẫu nhiệm vụ.
    pub task_template: TaskTemplateDto,
    /// Tự động giao việc.
    pub auto_dispatch: bool,
    /// Chính sách bỏ lỡ.
    pub missed_policy: String,
    /// Trạng thái bật/tắt.
    pub enabled: bool,
    /// Thời điểm kích hoạt gần nhất.
    pub last_fire: Option<String>,
    /// Thời điểm kích hoạt kế tiếp.
    pub next_fire: Option<String>,
    /// Thời điểm tạo.
    pub created_at: String,
    /// Thời điểm cập nhật.
    pub updated_at: String,
}

impl ScheduleRecord {
    /// Chuyển đổi từ bản ghi DB sang DTO.
    #[must_use]
    pub fn to_dto(&self) -> ScheduleDto {
        let task_template: TaskTemplateDto = serde_json::from_str(&self.task_template)
            .unwrap_or_else(|_| TaskTemplateDto {
                title: self.title.clone(),
                description: String::new(),
                lead_agent: String::new(),
                worker_agent: String::new(),
            });

        ScheduleDto {
            id: self.id.clone(),
            workspace_id: self.workspace_id.clone(),
            project_id: self.project_id.clone(),
            title: self.title.clone(),
            schedule_type: self.schedule_type.clone(),
            schedule_expr: self.schedule_expr.clone(),
            timezone: self.timezone.clone(),
            task_template,
            auto_dispatch: self.auto_dispatch == 1,
            missed_policy: self.missed_policy.clone(),
            enabled: self.enabled == 1,
            last_fire: self.last_fire.clone(),
            next_fire: self.next_fire.clone(),
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
        }
    }
}

/// Yêu cầu tạo mới lịch định kỳ.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduleRequest {
    /// Định danh dự án.
    pub project_id: String,
    /// Tiêu đề lịch.
    pub title: String,
    /// Loại lịch: "cron" hoặc "interval".
    pub schedule_type: String,
    /// Biểu thức cron hoặc khoảng đều tính theo giây/phút.
    pub schedule_expr: String,
    /// Múi giờ (tùy chọn, mặc định "UTC").
    pub timezone: Option<String>,
    /// Mẫu nhiệm vụ.
    pub task_template: TaskTemplateDto,
    /// Tùy chọn tự động giao việc.
    pub auto_dispatch: Option<bool>,
    /// Chính sách bỏ lỡ: "skip" hoặc "run_once".
    pub missed_policy: Option<String>,
}

/// Yêu cầu cập nhật lịch định kỳ.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateScheduleRequest {
    /// Tiêu đề lịch.
    pub title: Option<String>,
    /// Loại lịch: "cron" hoặc "interval".
    pub schedule_type: Option<String>,
    /// Biểu thức lịch mới.
    pub schedule_expr: Option<String>,
    /// Múi giờ.
    pub timezone: Option<String>,
    /// Mẫu nhiệm vụ mới.
    pub task_template: Option<TaskTemplateDto>,
    /// Cờ tự động giao việc.
    pub auto_dispatch: Option<bool>,
    /// Chính sách bỏ lỡ.
    pub missed_policy: Option<String>,
    /// Bật hoặc tạm dừng lịch.
    pub enabled: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calendar_action_has_only_create_task() {
        // AC 5: Tuyệt đối cấm hành động chạy lệnh; CalendarAction chỉ có MỘT biến thể.
        let action = CalendarAction::CreateTask {
            title: "T".into(),
            description: "D".into(),
            lead_agent: "lead".into(),
            worker_agent: "worker".into(),
        };

        match &action {
            CalendarAction::CreateTask {
                title,
                description,
                lead_agent,
                worker_agent,
            } => {
                assert_eq!(title, "T");
                assert_eq!(description, "D");
                assert_eq!(lead_agent, "lead");
                assert_eq!(worker_agent, "worker");
            }
        }

        let json_str = serde_json::to_string(&action).unwrap();
        assert!(json_str.contains(r#""type":"create_task""#));
    }
}

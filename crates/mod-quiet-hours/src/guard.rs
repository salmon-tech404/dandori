//! Hiện thực điểm mở rộng `TransitionGuard` cho module `quiet-hours`.

#![expect(
    clippy::too_many_lines,
    reason = "Xử lý guard check với nhiều nhánh tác nhân và truy vấn"
)]

use chrono::{DateTime, Utc};
use dandori_module_api::{
    BoxFuture, GuardDecision, GuardError, SystemSource, TransitionActor, TransitionGuard,
};
use dandori_types::{Task, TaskStatus};
use sqlx::SqlitePool;
use std::fmt;
use std::sync::Arc;
use tracing::info;

use crate::model::QuietWindow;
use crate::window::evaluate_windows;

/// Trait trừu tượng hóa đồng hồ cho module quiet-hours (RS-TEST-004).
pub trait Clock: fmt::Debug + Send + Sync + 'static {
    /// Lấy thời điểm hiện tại UTC.
    fn now(&self) -> DateTime<Utc>;
}

/// Đồng hồ thời gian thực dùng trong môi trường chạy thật.
#[derive(Clone, Copy, Debug, Default)]
pub struct RealClock;

impl Clock for RealClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Đồng hồ giả định có thể điều chỉnh thời gian phục vụ kiểm thử.
#[derive(Debug)]
pub struct MockClock {
    current: std::sync::RwLock<DateTime<Utc>>,
}

impl MockClock {
    /// Khởi tạo đồng hồ giả định với thời gian ban đầu.
    #[must_use]
    pub fn new(initial: DateTime<Utc>) -> Self {
        Self {
            current: std::sync::RwLock::new(initial),
        }
    }

    /// Đặt lại thời điểm của đồng hồ giả định.
    pub fn set(&self, new_time: DateTime<Utc>) {
        if let Ok(mut lock) = self.current.write() {
            *lock = new_time;
        }
    }
}

impl Clock for MockClock {
    fn now(&self) -> DateTime<Utc> {
        self.current
            .read()
            .map(|t| *t)
            .unwrap_or_else(|_| Utc::now())
    }
}

/// Điểm mở rộng TransitionGuard kiểm soát khung giờ cấm.
#[derive(Clone)]
pub struct QuietHoursGuard {
    pool: SqlitePool,
    clock: Arc<dyn Clock>,
}

impl fmt::Debug for QuietHoursGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QuietHoursGuard").finish_non_exhaustive()
    }
}

impl QuietHoursGuard {
    /// Khởi tạo Guard mới với kết nối database và đồng hồ.
    #[must_use]
    pub fn new(pool: SqlitePool, clock: Arc<dyn Clock>) -> Self {
        Self { pool, clock }
    }
}

impl TransitionGuard for QuietHoursGuard {
    fn name(&self) -> &'static str {
        "quiet-hours"
    }

    fn check(
        &self,
        task: &Task,
        _from: TaskStatus,
        to: TaskStatus,
        actor: &TransitionActor,
    ) -> BoxFuture<'static, Result<GuardDecision, GuardError>> {
        // Chỉ can thiệp khi task chuyển sang trạng thái Running
        if to != TaskStatus::Running {
            return Box::pin(async { Ok(GuardDecision::Allow) });
        }

        let pool = self.pool.clone();
        let clock = self.clock.clone();
        let project_id = task.project_id.to_string();
        let task_id = task.id;
        let actor = actor.clone();

        Box::pin(async move {
            // 1. Xác định workspace_id từ project_id của task
            let ws_row: Option<(String,)> =
                sqlx::query_as("SELECT workspace_id FROM project WHERE id = ?")
                    .bind(&project_id)
                    .fetch_optional(&pool)
                    .await
                    .map_err(|e| {
                        GuardError::CheckFailed(format!("Lỗi truy vấn workspace của project: {e}"))
                    })?;

            let workspace_id = match ws_row {
                Some((ws,)) => ws,
                None => {
                    // Nếu không tìm thấy dự án trong bảng project, cho phép tiếp tục
                    return Ok(GuardDecision::Allow);
                }
            };

            // 2. Tải tất cả các khung giờ cấm đang bật của workspace (toàn workspace hoặc riêng dự án này)
            let windows: Vec<QuietWindow> = sqlx::query_as(
                r#"
                SELECT id, workspace_id, project_id, name, days_of_week, start_time,
                       end_time, timezone, block_manual_runs, enabled, created_at, updated_at
                FROM m_quiet_hours_window
                WHERE workspace_id = ? AND (project_id = ? OR project_id IS NULL) AND enabled = 1
                "#,
            )
            .bind(&workspace_id)
            .bind(&project_id)
            .fetch_all(&pool)
            .await
            .map_err(|e| GuardError::CheckFailed(format!("Lỗi tải cấu hình quiet hours: {e}")))?;

            if windows.is_empty() {
                return Ok(GuardDecision::Allow);
            }

            // 3. Đánh giá thời điểm hiện tại
            let now = clock.now();
            let status = evaluate_windows(&windows, now, None)
                .map_err(|e| GuardError::CheckFailed(format!("Lỗi đánh giá khung giờ: {e}")))?;

            if !status.in_quiet_window {
                return Ok(GuardDecision::Allow);
            }

            let retry_after_dt = status.retry_after.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            });

            // 4. Kiểm tra quyền và nguồn kích hoạt (AC 1, AC 3)
            match actor {
                TransitionActor::User(_) => {
                    if status.blocks_manual_runs {
                        info!(
                            task_id = %task_id,
                            "Chặn chạy tay task trong khung giờ cấm có cấu hình chặn chạy tay"
                        );
                        Ok(GuardDecision::Deny {
                            reason: format!(
                                "Nhiệm vụ bị chặn do đang trong khung giờ cấm chạy ({}) có áp dụng cho chạy thủ công",
                                status.active_window_names.join(", ")
                            ),
                            retry_after: retry_after_dt,
                        })
                    } else {
                        // Mặc định: Chạy tay do người dùng khởi động KHÔNG bị chặn
                        Ok(GuardDecision::Allow)
                    }
                }
                TransitionActor::System(SystemSource::Scheduler(sched)) => {
                    info!(
                        task_id = %task_id,
                        scheduler = %sched,
                        "Hoãn task tự động từ scheduler trong khung giờ cấm"
                    );
                    Ok(GuardDecision::Deny {
                        reason: format!(
                            "Nhiệm vụ tự động từ bộ lập lịch ({sched}) bị hoãn do đang trong khung giờ cấm chạy ({})",
                            status.active_window_names.join(", ")
                        ),
                        retry_after: retry_after_dt,
                    })
                }
                TransitionActor::System(SystemSource::Module(module_id)) => {
                    info!(
                        task_id = %task_id,
                        module_id = %module_id,
                        "Hoãn task tự động từ module trong khung giờ cấm"
                    );
                    Ok(GuardDecision::Deny {
                        reason: format!(
                            "Nhiệm vụ tự động từ module ({module_id}) bị hoãn do đang trong khung giờ cấm chạy ({})",
                            status.active_window_names.join(", ")
                        ),
                        retry_after: retry_after_dt,
                    })
                }
            }
        })
    }
}

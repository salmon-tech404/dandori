//! Worker nền thực thi lịch định kỳ cho module `calendar`.

#![expect(
    clippy::too_many_lines,
    reason = "Xử lý vòng lặp và logic worker kèm atomic claim"
)]

use chrono::{DateTime, Utc};
use dandori_module_api::EventBus;
use dandori_types::{
    Envelope, EventPayload, EventScope, ProjectId, TaskId, TaskStatus, WorkspaceId,
};
use sqlx::SqlitePool;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::model::{CalendarAction, ScheduleRecord, TaskTemplateDto};
use crate::schedule::compute_next_fire;

/// Trait trừu tượng hóa đồng hồ để phục vụ kiểm thử đơn vị nhanh và chính xác (RS-TEST-004).
pub trait Clock: std::fmt::Debug + Send + Sync + 'static {
    /// Lấy thời điểm hiện tại theo UTC.
    fn now(&self) -> DateTime<Utc>;
}

/// Đồng hồ thời gian thực dùng cho môi trường chạy thực tế.
#[derive(Clone, Copy, Debug, Default)]
pub struct RealClock;

impl Clock for RealClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Đồng hồ giả có thể điều chỉnh thời gian phục vụ kiểm thử (MockClock).
#[derive(Debug)]
pub struct MockClock {
    current: std::sync::RwLock<DateTime<Utc>>,
}

impl MockClock {
    /// Khởi tạo đồng hồ giả với thời điểm ban đầu.
    #[must_use]
    pub fn new(initial: DateTime<Utc>) -> Self {
        Self {
            current: std::sync::RwLock::new(initial),
        }
    }

    /// Tiến đồng hồ thêm một khoảng thời gian.
    pub fn advance(&self, duration: chrono::Duration) {
        if let Ok(mut lock) = self.current.write() {
            *lock += duration;
        }
    }

    /// Đặt lại thời điểm của đồng hồ giả.
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

/// Cấu hình worker lịch.
#[derive(Clone, Debug)]
pub struct CalendarWorkerConfig {
    /// Tần suất thăm dò lịch (mặc định 1 giây).
    pub poll_interval: Duration,
}

impl Default for CalendarWorkerConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(1),
        }
    }
}

/// Khởi chạy vòng lặp worker nền cho module calendar.
pub async fn run_calendar_worker(
    pool: SqlitePool,
    clock: Arc<dyn Clock>,
    token: CancellationToken,
    event_bus: Arc<EventBus>,
    config: CalendarWorkerConfig,
) {
    info!("Bắt đầu calendar_worker nền");

    while !token.is_cancelled() {
        let now = clock.now();
        if let Err(e) = process_due_schedules(&pool, &*clock, now, &event_bus).await {
            error!("Lỗi xử lý lịch định kỳ: {e}");
        }

        tokio::select! {
            () = token.cancelled() => {
                info!("calendar_worker nhận tín hiệu dừng");
                break;
            }
            () = tokio::time::sleep(config.poll_interval) => {}
        }
    }

    info!("calendar_worker đã dừng hoàn toàn");
}

/// Xử lý các lịch đã tới giờ kích hoạt tại thời điểm `now`.
pub async fn process_due_schedules(
    pool: &SqlitePool,
    clock: &dyn Clock,
    now: DateTime<Utc>,
    event_bus: &EventBus,
) -> Result<usize, sqlx::Error> {
    let now_str = now.to_rfc3339();

    // 1. Tìm các lịch đang bật và có next_fire <= now
    let due_schedules: Vec<ScheduleRecord> = sqlx::query_as(
        r#"
        SELECT id, workspace_id, project_id, title, schedule_type, schedule_expr, timezone,
               task_template, auto_dispatch, missed_policy, enabled, last_fire, next_fire,
               created_at, updated_at
        FROM m_calendar_schedule
        WHERE enabled = 1 AND next_fire IS NOT NULL AND next_fire <= ?
        ORDER BY next_fire ASC
        "#,
    )
    .bind(&now_str)
    .fetch_all(pool)
    .await?;

    let count = due_schedules.len();
    for sched in due_schedules {
        if let Err(e) = fire_single_schedule(pool, clock, &sched, now, event_bus).await {
            warn!("Không thể kích hoạt lịch {}: {e}", sched.id);
        }
    }

    Ok(count)
}

/// Kích hoạt một lịch cụ thể và cập nhật trạng thái atomically.
pub async fn fire_single_schedule(
    pool: &SqlitePool,
    _clock: &dyn Clock,
    sched: &ScheduleRecord,
    now: DateTime<Utc>,
    event_bus: &EventBus,
) -> Result<bool, String> {
    let next_fire_old = match &sched.next_fire {
        Some(nf) => nf.clone(),
        None => return Ok(false),
    };

    // Tính thời điểm tiếp theo
    let computed_next = compute_next_fire(
        &sched.schedule_type,
        &sched.schedule_expr,
        &sched.timezone,
        now,
    )?;
    let next_fire_new_str = computed_next.to_rfc3339();
    let now_str = now.to_rfc3339();

    // Cập nhật nguyên tử (Atomic CAS)
    let res = sqlx::query(
        r#"
        UPDATE m_calendar_schedule
        SET last_fire = ?, next_fire = ?, updated_at = ?
        WHERE id = ? AND enabled = 1 AND next_fire = ?
        "#,
    )
    .bind(&now_str)
    .bind(&next_fire_new_str)
    .bind(&now_str)
    .bind(&sched.id)
    .bind(&next_fire_old)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    if res.rows_affected() == 0 {
        return Ok(false); // Đã có worker khác nhận xử lý
    }

    // Kiểm tra chính sách bỏ lỡ: Nếu máy tắt một thời gian dài và missed_policy == "skip"
    // Nếu scheduled_time quá cũ (ví dụ chậm hơn 1 giờ) và missed_policy == "skip", bỏ qua tạo task
    let is_severely_missed = if let Ok(scheduled_dt) = DateTime::parse_from_rfc3339(&next_fire_old)
    {
        let scheduled_utc = scheduled_dt.with_timezone(&Utc);
        (now - scheduled_utc) > chrono::Duration::hours(2)
    } else {
        false
    };

    if is_severely_missed && sched.missed_policy == "skip" {
        info!(
            "Lịch {} bị lỡ sâu (quá 2h) và chính sách là skip -> bỏ qua tạo task",
            sched.id
        );
        return Ok(true);
    }

    // Tiêu chí nghiệm thu AC 5: Thực hiện hành động duy nhất là Tạo Task từ mẫu
    let template: TaskTemplateDto =
        serde_json::from_str(&sched.task_template).unwrap_or_else(|_| TaskTemplateDto {
            title: sched.title.clone(),
            description: String::new(),
            lead_agent: "lead".to_string(),
            worker_agent: "worker".to_string(),
        });

    let action = CalendarAction::CreateTask {
        title: template.title,
        description: template.description,
        lead_agent: template.lead_agent,
        worker_agent: template.worker_agent,
    };

    execute_calendar_action(pool, sched, action, now, event_bus).await?;
    Ok(true)
}

/// Thực thi hành động tạo nhiệm vụ từ lịch.
async fn execute_calendar_action(
    pool: &SqlitePool,
    sched: &ScheduleRecord,
    action: CalendarAction,
    now: DateTime<Utc>,
    event_bus: &EventBus,
) -> Result<(), String> {
    match action {
        CalendarAction::CreateTask {
            title,
            description,
            lead_agent,
            worker_agent,
        } => {
            let task_id = TaskId::new();
            let now_str = now.to_rfc3339();

            sqlx::query(
                r#"
                INSERT INTO task (
                    id, project_id, title, description, status,
                    lead_agent, worker_agent, created_at, updated_at
                )
                VALUES (?, ?, ?, ?, 'backlog', ?, ?, ?, ?)
                "#,
            )
            .bind(task_id.to_string())
            .bind(&sched.project_id)
            .bind(&title)
            .bind(&description)
            .bind(&lead_agent)
            .bind(&worker_agent)
            .bind(&now_str)
            .bind(&now_str)
            .execute(pool)
            .await
            .map_err(|e| format!("Lỗi tạo task từ calendar: {e}"))?;

            let ws_id = WorkspaceId::from_str(&sched.workspace_id)
                .unwrap_or_else(|_| WorkspaceId::from(uuid::Uuid::nil()));
            let prj_id = ProjectId::from_str(&sched.project_id)
                .unwrap_or_else(|_| ProjectId::from(uuid::Uuid::nil()));

            event_bus.publish(Envelope::new(
                now,
                EventScope::task(ws_id, prj_id, task_id),
                EventPayload::TaskStatusChanged {
                    task_id,
                    old_status: "created".to_string(),
                    new_status: TaskStatus::Backlog.as_str().to_string(),
                },
            ));

            info!(
                "Calendar {} đã tạo thành công task {} trong dự án {}",
                sched.id, task_id, sched.project_id
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let core_schema =
            include_str!("../../../crates/dandori-db/migrations/0001_core_schema.sql");
        sqlx::query(core_schema).execute(&pool).await.unwrap();

        let calendar_schema = include_str!("../migrations/0001_calendar.sql");
        sqlx::query(calendar_schema).execute(&pool).await.unwrap();

        pool
    }

    #[tokio::test]
    async fn test_hourly_schedule_fires_24_times_in_24_hours() {
        let pool = setup_test_db().await;
        let event_bus = Arc::new(EventBus::new(100));

        // Tạo workspace và project mẫu
        let ws_id = WorkspaceId::new().to_string();
        let prj_id = ProjectId::new().to_string();
        let now_str = Utc::now().to_rfc3339();

        sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS', ?)")
            .bind(&ws_id)
            .bind(&now_str)
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'PRJ', 'd:/test', 'main', ?)")
            .bind(&prj_id)
            .bind(&ws_id)
            .bind(&now_str)
            .execute(&pool)
            .await
            .unwrap();

        // Bắt đầu lúc 2026-11-02 00:00:00 UTC
        let start_time = Utc.with_ymd_and_hms(2026, 11, 2, 0, 0, 0).unwrap();
        let clock = Arc::new(MockClock::new(start_time));

        // Lịch chạy mỗi giờ tại phút 0 ("0 * * * *")
        // Lần kích hoạt đầu tiên: 01:00:00 UTC
        let next_fire = Utc
            .with_ymd_and_hms(2026, 11, 2, 1, 0, 0)
            .unwrap()
            .to_rfc3339();
        let sched_id = "sched_hourly_24";

        sqlx::query(
            r#"
            INSERT INTO m_calendar_schedule (
                id, workspace_id, project_id, title, schedule_type, schedule_expr, timezone,
                task_template, auto_dispatch, missed_policy, enabled, next_fire, created_at, updated_at
            )
            VALUES (?, ?, ?, 'Hourly Task', 'cron', '0 * * * *', 'UTC', '{}', 0, 'run_once', 1, ?, ?, ?)
            "#,
        )
        .bind(sched_id)
        .bind(&ws_id)
        .bind(&prj_id)
        .bind(&next_fire)
        .bind(&now_str)
        .bind(&now_str)
        .execute(&pool)
        .await
        .unwrap();

        let mut fires_count = 0;
        // Chạy qua 24 giờ, mỗi lần nhảy 1 giờ
        for _ in 1..=24 {
            clock.advance(chrono::Duration::hours(1));
            let processed = process_due_schedules(&pool, &*clock, clock.now(), &event_bus)
                .await
                .unwrap();
            fires_count += processed;
        }

        // AC 1: Đúng 24 lần kích hoạt trong 24 giờ
        assert_eq!(fires_count, 24);

        // Kiểm tra số lượng task được tạo trong DB
        let task_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM task WHERE project_id = ?")
            .bind(&prj_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(task_count, 24);
    }

    #[tokio::test]
    async fn test_restart_and_missed_policy() {
        let pool = setup_test_db().await;
        let event_bus = Arc::new(EventBus::new(100));

        let ws_id = WorkspaceId::new().to_string();
        let prj_id = ProjectId::new().to_string();
        let now_str = Utc::now().to_rfc3339();

        sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS', ?)")
            .bind(&ws_id)
            .bind(&now_str)
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'PRJ', 'd:/test', 'main', ?)")
            .bind(&prj_id)
            .bind(&ws_id)
            .bind(&now_str)
            .execute(&pool)
            .await
            .unwrap();

        // Máy tắt lúc 10:00, lịch đặt lúc 11:00
        let sched_time = Utc.with_ymd_and_hms(2026, 11, 2, 11, 0, 0).unwrap();
        // Bật lại máy lúc 16:00 (chậm 5 tiếng)
        let resume_time = Utc.with_ymd_and_hms(2026, 11, 2, 16, 0, 0).unwrap();
        let clock = Arc::new(MockClock::new(resume_time));

        // Trường hợp 1: missed_policy = "skip" -> bỏ qua tạo task lỡ sâu
        let sched_skip = "sched_skip";
        sqlx::query(
            r#"
            INSERT INTO m_calendar_schedule (
                id, workspace_id, project_id, title, schedule_type, schedule_expr, timezone,
                task_template, auto_dispatch, missed_policy, enabled, next_fire, created_at, updated_at
            )
            VALUES (?, ?, ?, 'Skip Task', 'cron', '0 * * * *', 'UTC', '{}', 0, 'skip', 1, ?, ?, ?)
            "#,
        )
        .bind(sched_skip)
        .bind(&ws_id)
        .bind(&prj_id)
        .bind(sched_time.to_rfc3339())
        .bind(&now_str)
        .bind(&now_str)
        .execute(&pool)
        .await
        .unwrap();

        // Xử lý sau khi khởi động lại
        process_due_schedules(&pool, &*clock, clock.now(), &event_bus)
            .await
            .unwrap();

        // Không có task nào được tạo vì đã bị bỏ qua (skip)
        let count_skip: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM task WHERE project_id = ?")
            .bind(&prj_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count_skip, 0);

        // Trường hợp 2: missed_policy = "run_once" -> tạo 1 task bù
        let sched_run_once = "sched_run_once";
        sqlx::query(
            r#"
            INSERT INTO m_calendar_schedule (
                id, workspace_id, project_id, title, schedule_type, schedule_expr, timezone,
                task_template, auto_dispatch, missed_policy, enabled, next_fire, created_at, updated_at
            )
            VALUES (?, ?, ?, 'RunOnce Task', 'cron', '0 * * * *', 'UTC', '{}', 0, 'run_once', 1, ?, ?, ?)
            "#,
        )
        .bind(sched_run_once)
        .bind(&ws_id)
        .bind(&prj_id)
        .bind(sched_time.to_rfc3339())
        .bind(&now_str)
        .bind(&now_str)
        .execute(&pool)
        .await
        .unwrap();

        process_due_schedules(&pool, &*clock, clock.now(), &event_bus)
            .await
            .unwrap();

        let count_run_once: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM task WHERE project_id = ?")
                .bind(&prj_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        // Đúng 1 task bù được tạo
        assert_eq!(count_run_once, 1);
    }
}

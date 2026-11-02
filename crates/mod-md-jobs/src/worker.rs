//! Worker nền thực thi các công việc Markdown đã được duyệt (`md-jobs`).

#![expect(
    clippy::too_many_lines,
    reason = "Worker xử lý atomic claim, tính toán cron và phát sự kiện tạo task"
)]

use chrono::{DateTime, Utc};
use croner::Cron;
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

use crate::model::MdJobRecord;

/// Trait trừu tượng hóa đồng hồ để phục vụ kiểm thử nhanh (RS-TEST-004).
pub trait Clock: std::fmt::Debug + Send + Sync + 'static {
    /// Lấy thời điểm hiện tại theo UTC.
    fn now(&self) -> DateTime<Utc>;
}

/// Đồng hồ thời gian thực dùng cho môi trường sản xuất.
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
    /// Khởi tạo đồng hồ giả với mốc thời gian ban đầu.
    #[must_use]
    pub fn new(initial: DateTime<Utc>) -> Self {
        Self {
            current: std::sync::RwLock::new(initial),
        }
    }

    /// Tiến thời gian thêm một khoảng.
    pub fn advance(&self, duration: chrono::Duration) {
        if let Ok(mut lock) = self.current.write() {
            *lock += duration;
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

/// Cấu hình worker cho `md-jobs`.
#[derive(Clone, Debug)]
pub struct MdJobsWorkerConfig {
    /// Tần suất thăm dò CSDL (mặc định 1 giây).
    pub poll_interval: Duration,
}

impl Default for MdJobsWorkerConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(1),
        }
    }
}

/// Vòng lặp worker nền thực thi các job đã đến giờ kích hoạt.
pub async fn run_md_jobs_worker(
    pool: SqlitePool,
    clock: Arc<dyn Clock>,
    token: CancellationToken,
    event_bus: Arc<EventBus>,
    config: MdJobsWorkerConfig,
) {
    info!("Bắt đầu md_jobs_worker nền");

    while !token.is_cancelled() {
        let now = clock.now();
        if let Err(e) = process_due_md_jobs(&pool, &*clock, now, &event_bus).await {
            error!("Lỗi xử lý md-jobs: {e}");
        }

        tokio::select! {
            () = token.cancelled() => {
                info!("md_jobs_worker nhận tín hiệu dừng");
                break;
            }
            () = tokio::time::sleep(config.poll_interval) => {}
        }
    }

    info!("md_jobs_worker đã dừng hoàn toàn");
}

/// Quét và xử lý các job đã đến thời điểm kích hoạt.
pub async fn process_due_md_jobs(
    pool: &SqlitePool,
    _clock: &dyn Clock,
    now: DateTime<Utc>,
    event_bus: &EventBus,
) -> Result<usize, sqlx::Error> {
    let now_str = now.to_rfc3339();

    // Tiêu chí AC 1: CHỈ CÁC JOB ĐÃ DUYỆT (status = 'approved') MỚI ĐƯỢC CHẠY
    let due_jobs: Vec<MdJobRecord> = sqlx::query_as(
        r#"
        SELECT id, workspace_id, project_id, file_path, job_name, schedule_cron,
               agent_role, account_id, prompt, content_hash, approved_hash, status,
               next_fire, last_fire, created_at, updated_at
        FROM m_md_job
        WHERE status = 'approved' AND next_fire IS NOT NULL AND next_fire <= ?
        ORDER BY next_fire ASC
        "#,
    )
    .bind(&now_str)
    .fetch_all(pool)
    .await?;

    let count = due_jobs.len();
    for job in due_jobs {
        if let Err(e) = fire_single_md_job(pool, &job, now, event_bus).await {
            warn!("Không thể kích hoạt job {}: {e}", job.id);
        }
    }

    Ok(count)
}

/// Kích hoạt một job và cập nhật nguyên tử `next_fire`.
pub async fn fire_single_md_job(
    pool: &SqlitePool,
    job: &MdJobRecord,
    now: DateTime<Utc>,
    event_bus: &EventBus,
) -> Result<bool, String> {
    let old_next_fire = match &job.next_fire {
        Some(nf) => nf.clone(),
        None => return Ok(false),
    };

    // Tính thời điểm tiếp theo theo biểu thức cron
    let cron = Cron::new(&job.schedule_cron)
        .parse()
        .map_err(|e| format!("Lỗi phân tích cron: {e}"))?;

    let next_occurrence = cron
        .find_next_occurrence(&now, false)
        .map_err(|e| format!("Lỗi tính lần chạy kế tiếp: {e}"))?;

    let now_str = now.to_rfc3339();
    let next_fire_str = next_occurrence.to_rfc3339();

    // Cập nhật nguyên tử (Atomic CAS)
    let res = sqlx::query(
        r#"
        UPDATE m_md_job
        SET last_fire = ?, next_fire = ?, updated_at = ?
        WHERE id = ? AND status = 'approved' AND next_fire = ?
        "#,
    )
    .bind(&now_str)
    .bind(&next_fire_str)
    .bind(&now_str)
    .bind(&job.id)
    .bind(&old_next_fire)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    if res.rows_affected() == 0 {
        return Ok(false); // Đã có worker khác claim
    }

    // Tạo task trong CSDL lõi
    let task_id = TaskId::new();
    let title = format!("[Job] {}", job.job_name);
    let worker_agent = if job.agent_role.is_empty() {
        "worker".to_string()
    } else {
        job.agent_role.clone()
    };

    sqlx::query(
        r#"
        INSERT INTO task (
            id, project_id, title, description, status,
            lead_agent, worker_agent, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, 'backlog', 'lead', ?, ?, ?)
        "#,
    )
    .bind(task_id.to_string())
    .bind(&job.project_id)
    .bind(&title)
    .bind(&job.prompt)
    .bind(&worker_agent)
    .bind(&now_str)
    .bind(&now_str)
    .execute(pool)
    .await
    .map_err(|e| format!("Lỗi tạo task từ md-job: {e}"))?;

    let ws_id = WorkspaceId::from_str(&job.workspace_id)
        .unwrap_or_else(|_| WorkspaceId::from(uuid::Uuid::nil()));
    let prj_id =
        ProjectId::from_str(&job.project_id).unwrap_or_else(|_| ProjectId::from(uuid::Uuid::nil()));

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
        "MdJob {} đã tạo thành công task {} trong dự án {}",
        job.id, task_id, job.project_id
    );

    Ok(true)
}

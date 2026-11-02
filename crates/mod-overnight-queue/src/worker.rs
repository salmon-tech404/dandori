//! Worker nền thực thi hàng đợi qua đêm, kiểm soát dừng an toàn và phát tóm tắt sáng.

#![expect(
    clippy::too_many_lines,
    reason = "Xử lý vòng lặp điều phối hàng đợi qua đêm, giới hạn song song và dừng an toàn"
)]

use chrono::{DateTime, Timelike, Utc};
use dandori_module_api::{AgentRole, EventBus, TaskRunner};
use dandori_types::{Envelope, EventPayload, EventScope, TaskId, UserId, WorkspaceId};
use sqlx::SqlitePool;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::model::{MorningSummaryDto, OvernightQueueConfig, OvernightQueueItem, ReviewChainItem};

/// Trait trừu tượng hóa đồng hồ cho module overnight-queue (RS-TEST-004).
pub trait Clock: fmt::Debug + Send + Sync + 'static {
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

/// Đồng hồ giả có thể điều chỉnh thời gian phục vụ kiểm thử.
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

/// Cấu hình vòng lặp worker hàng đợi qua đêm.
#[derive(Clone, Debug)]
pub struct OvernightQueueWorkerConfig {
    /// Tần suất thăm dò hàng đợi.
    pub poll_interval: Duration,
}

impl Default for OvernightQueueWorkerConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(1),
        }
    }
}

/// Khởi chạy vòng lặp worker nền cho module overnight-queue.
pub async fn run_overnight_queue_worker(
    pool: SqlitePool,
    clock: Arc<dyn Clock>,
    token: CancellationToken,
    event_bus: Arc<EventBus>,
    runner: Option<Arc<dyn TaskRunner>>,
    config: OvernightQueueWorkerConfig,
) {
    info!("Bắt đầu overnight_queue_worker nền");

    while !token.is_cancelled() {
        let now = clock.now();
        if let Err(e) =
            process_overnight_queues(&pool, &*clock, now, &event_bus, runner.as_deref()).await
        {
            error!("Lỗi xử lý hàng đợi qua đêm: {e}");
        }

        tokio::select! {
            () = token.cancelled() => {
                info!("overnight_queue_worker nhận tín hiệu dừng");
                break;
            }
            () = tokio::time::sleep(config.poll_interval) => {}
        }
    }

    info!("overnight_queue_worker đã dừng hoàn toàn");
}

/// Quét và xử lý tất cả các hàng đợi qua đêm của các workspace.
pub async fn process_overnight_queues(
    pool: &SqlitePool,
    _clock: &dyn Clock,
    now: DateTime<Utc>,
    event_bus: &EventBus,
    runner: Option<&dyn TaskRunner>,
) -> Result<(), sqlx::Error> {
    let configs: Vec<OvernightQueueConfig> = sqlx::query_as(
        r#"
        SELECT workspace_id, start_time, timezone, max_concurrency,
               consecutive_failure_limit, status, consecutive_failures,
               last_run_summary, updated_at
        FROM m_overnight_queue_config
        "#,
    )
    .fetch_all(pool)
    .await?;

    for cfg in configs {
        let is_start_time = check_is_start_time(&cfg.start_time, now);
        let mut status = cfg.status.clone();

        // Tự động kích hoạt khi đến giờ bắt đầu nếu đang ở trạng thái idle
        if status == "idle" && is_start_time {
            info!(ws = %cfg.workspace_id, "Tới giờ bắt đầu hàng đợi qua đêm ({})", cfg.start_time);
            let now_str = now.to_rfc3339();
            sqlx::query("UPDATE m_overnight_queue_config SET status = 'running', updated_at = ? WHERE workspace_id = ?")
                .bind(&now_str)
                .bind(&cfg.workspace_id)
                .execute(pool)
                .await?;
            status = "running".to_string();
        }

        if status != "running" {
            continue;
        }

        process_single_queue(pool, &cfg, now, event_bus, runner).await?;
    }

    Ok(())
}

fn check_is_start_time(start_time_str: &str, now: DateTime<Utc>) -> bool {
    let parts: Vec<&str> = start_time_str.split(':').collect();
    if parts.len() != 2 {
        return false;
    }
    if let (Some(h_str), Some(m_str)) = (parts.first(), parts.get(1))
        && let (Ok(h), Ok(m)) = (h_str.parse::<u32>(), m_str.parse::<u32>())
    {
        now.hour() == h && now.minute() == m
    } else {
        false
    }
}

async fn process_single_queue(
    pool: &SqlitePool,
    cfg: &OvernightQueueConfig,
    now: DateTime<Utc>,
    event_bus: &EventBus,
    runner: Option<&dyn TaskRunner>,
) -> Result<(), sqlx::Error> {
    // 1. Đếm số task đang chạy (running) trong hàng đợi
    let active_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM m_overnight_queue_item WHERE workspace_id = ? AND status = 'running'",
    )
    .bind(&cfg.workspace_id)
    .fetch_one(pool)
    .await?;

    let slots_available = (cfg.max_concurrency - active_count).max(0) as usize;
    if slots_available == 0 {
        return Ok(());
    }

    // 2. Lấy các task tiếp theo đang ở trạng thái 'queued' theo thứ tự order_index
    let next_items: Vec<OvernightQueueItem> = sqlx::query_as(
        r#"
        SELECT id, workspace_id, task_id, order_index, status, run_id, created_at, updated_at
        FROM m_overnight_queue_item
        WHERE workspace_id = ? AND status = 'queued'
        ORDER BY order_index ASC
        LIMIT ?
        "#,
    )
    .bind(&cfg.workspace_id)
    .bind(slots_available as i64)
    .fetch_all(pool)
    .await?;

    // 3. Nếu không còn task nào queued
    if next_items.is_empty() {
        if active_count == 0 {
            // Hàng đợi đã chạy xong toàn bộ task!
            finalize_overnight_queue(pool, cfg, now, event_bus, false).await?;
        }
        return Ok(());
    }

    // 4. Khởi chạy từng task
    for item in next_items {
        execute_queue_item(pool, cfg, &item, now, event_bus, runner).await?;
    }

    Ok(())
}

async fn execute_queue_item(
    pool: &SqlitePool,
    cfg: &OvernightQueueConfig,
    item: &OvernightQueueItem,
    now: DateTime<Utc>,
    event_bus: &EventBus,
    runner: Option<&dyn TaskRunner>,
) -> Result<(), sqlx::Error> {
    let now_str = now.to_rfc3339();

    // Kiểm tra trạng thái thực tế của task trong bảng task
    let task_row: Option<(String, String)> =
        sqlx::query_as("SELECT status, project_id FROM task WHERE id = ?")
            .bind(&item.task_id)
            .fetch_optional(pool)
            .await?;

    let (task_status, _project_id) = match task_row {
        Some(t) => t,
        None => {
            // Task không còn tồn tại -> đánh dấu failed
            sqlx::query(
                "UPDATE m_overnight_queue_item SET status = 'failed', updated_at = ? WHERE id = ?",
            )
            .bind(&now_str)
            .bind(&item.id)
            .execute(pool)
            .await?;
            return Ok(());
        }
    };

    // Bền vững qua khởi động lại: nếu task đã done hoặc ready_to_merge, không chạy lại
    if task_status == "done" || task_status == "ready_to_merge" {
        sqlx::query(
            "UPDATE m_overnight_queue_item SET status = 'completed', updated_at = ? WHERE id = ?",
        )
        .bind(&now_str)
        .bind(&item.id)
        .execute(pool)
        .await?;
        return Ok(());
    }

    // Đánh dấu item là running
    sqlx::query(
        "UPDATE m_overnight_queue_item SET status = 'running', updated_at = ? WHERE id = ?",
    )
    .bind(&now_str)
    .bind(&item.id)
    .execute(pool)
    .await?;

    let task_id = TaskId::from_str(&item.task_id).unwrap_or_else(|_| TaskId::new());

    let run_res = if let Some(r) = runner {
        let system_user = UserId::new();
        r.start_run(task_id, AgentRole::Worker, system_user, vec![])
            .await
    } else {
        // Giả lập trong môi trường kiểm thử không có runner
        sqlx::query("UPDATE task SET status = 'in_review' WHERE id = ?")
            .bind(&item.task_id)
            .execute(pool)
            .await?;
        Ok(dandori_types::Run {
            id: dandori_types::RunId::new(),
            task_id,
            agent: "fake".to_string(),
            account: dandori_types::AgentAccountId::new().to_string(),
            status: "success".to_string(),
            started_at: now,
            finished_at: Some(now),
            exit_code: Some(0),
        })
    };

    match run_res {
        Ok(run) => {
            info!(task = %item.task_id, "Overnight queue task thành công");
            // Reset consecutive_failures về 0
            sqlx::query(
                "UPDATE m_overnight_queue_config SET consecutive_failures = 0, updated_at = ? WHERE workspace_id = ?",
            )
            .bind(&now_str)
            .bind(&cfg.workspace_id)
            .execute(pool)
            .await?;

            sqlx::query(
                "UPDATE m_overnight_queue_item SET status = 'completed', run_id = ?, updated_at = ? WHERE id = ?",
            )
            .bind(run.id.to_string())
            .bind(&now_str)
            .bind(&item.id)
            .execute(pool)
            .await?;
        }
        Err(err_msg) => {
            warn!(task = %item.task_id, err = %err_msg, "Overnight queue task thất bại");
            let new_failures = cfg.consecutive_failures + 1;

            sqlx::query(
                "UPDATE m_overnight_queue_item SET status = 'failed', updated_at = ? WHERE id = ?",
            )
            .bind(&now_str)
            .bind(&item.id)
            .execute(pool)
            .await?;

            if new_failures >= cfg.consecutive_failure_limit {
                warn!(
                    ws = %cfg.workspace_id,
                    failures = new_failures,
                    "Kích hoạt DỪNG AN TOÀN sau K task thất bại liên tiếp"
                );
                // Dừng an toàn hàng đợi
                sqlx::query(
                    r#"
                    UPDATE m_overnight_queue_config
                    SET status = 'stopped', consecutive_failures = ?, updated_at = ?
                    WHERE workspace_id = ?
                    "#,
                )
                .bind(new_failures)
                .bind(&now_str)
                .bind(&cfg.workspace_id)
                .execute(pool)
                .await?;

                finalize_overnight_queue(pool, cfg, now, event_bus, true).await?;
            } else {
                sqlx::query(
                    "UPDATE m_overnight_queue_config SET consecutive_failures = ?, updated_at = ? WHERE workspace_id = ?",
                )
                .bind(new_failures)
                .bind(&now_str)
                .bind(&cfg.workspace_id)
                .execute(pool)
                .await?;
            }
        }
    }

    Ok(())
}

/// Hoàn tất hàng đợi, tổng hợp tóm tắt sáng hôm sau và phát sự kiện thông báo (một lần).
async fn finalize_overnight_queue(
    pool: &SqlitePool,
    cfg: &OvernightQueueConfig,
    now: DateTime<Utc>,
    event_bus: &EventBus,
    is_stopped: bool,
) -> Result<(), sqlx::Error> {
    let now_str = now.to_rfc3339();

    // Cập nhật trạng thái về idle nếu chưa bị stopped
    if !is_stopped {
        sqlx::query(
            "UPDATE m_overnight_queue_config SET status = 'idle', updated_at = ? WHERE workspace_id = ?",
        )
        .bind(&now_str)
        .bind(&cfg.workspace_id)
        .execute(pool)
        .await?;
    }

    // Tổng hợp số liệu
    let items: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT qi.task_id, t.status
        FROM m_overnight_queue_item qi
        JOIN task t ON t.id = qi.task_id
        WHERE qi.workspace_id = ?
        "#,
    )
    .bind(&cfg.workspace_id)
    .fetch_all(pool)
    .await?;

    let mut completed_count = 0;
    let mut failed_count = 0;
    let mut needs_human_count = 0;
    let mut review_items = Vec::new();

    for (tid, status) in items {
        let task_title: String = sqlx::query_scalar("SELECT title FROM task WHERE id = ?")
            .bind(&tid)
            .fetch_one(pool)
            .await
            .unwrap_or_else(|_| "Task".to_string());

        match status.as_str() {
            "done" | "in_review" | "ready_to_merge" => {
                completed_count += 1;
                review_items.push(ReviewChainItem {
                    task_id: tid,
                    title: task_title,
                    status,
                    branch_name: Some("dandori/overnight-task".to_string()),
                });
            }
            "needs_human" => {
                needs_human_count += 1;
                review_items.push(ReviewChainItem {
                    task_id: tid,
                    title: task_title,
                    status,
                    branch_name: None,
                });
            }
            "failed" => {
                failed_count += 1;
            }
            _ => {}
        }
    }

    let summary = MorningSummaryDto {
        completed_count,
        failed_count,
        needs_human_count,
        review_items,
        generated_at: now_str.clone(),
    };

    let summary_json = serde_json::to_string(&summary).unwrap_or_default();
    sqlx::query(
        "UPDATE m_overnight_queue_config SET last_run_summary = ?, updated_at = ? WHERE workspace_id = ?",
    )
    .bind(&summary_json)
    .bind(&now_str)
    .bind(&cfg.workspace_id)
    .execute(pool)
    .await?;

    let ws_id = WorkspaceId::from_str(&cfg.workspace_id)
        .unwrap_or_else(|_| WorkspaceId::from(uuid::Uuid::nil()));
    info!(
        ws = %cfg.workspace_id,
        completed = completed_count,
        failed = failed_count,
        needs_human = needs_human_count,
        "Phát sự kiện NotificationRequested cho tóm tắt hàng đợi qua đêm"
    );

    event_bus.publish(Envelope::new(
        now,
        EventScope::workspace(ws_id),
        EventPayload::NotificationRequested {
            title: "Bản tóm tắt hàng đợi qua đêm".to_string(),
            message: format!(
                "Hàng đợi qua đêm đã hoàn tất: {completed_count} nhiệm vụ hoàn thành, {needs_human_count} cần bạn xem xét, {failed_count} thất bại."
            ),
            level: if failed_count > 0 { "warning".to_string() } else { "info".to_string() },
        },
    ));

    Ok(())
}

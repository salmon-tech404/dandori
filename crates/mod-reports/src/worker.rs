//! Worker nền tự động chụp báo cáo tổng kết ngày/tuần và phát sự kiện thông báo.

use chrono::{DateTime, Datelike, Timelike, Utc};
use dandori_module_api::EventBus;
use dandori_types::{Envelope, EventPayload, EventScope, WorkspaceId};
use sqlx::SqlitePool;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use uuid::Uuid;

use crate::generator::{
    calculate_period_bounds, export_report_to_markdown, generate_report_data, parse_timezone,
};
use crate::model::ReportConfig;

/// Trait trừu tượng hóa đồng hồ cho module reports (RS-TEST-004).
pub trait Clock: fmt::Debug + Send + Sync + 'static {
    /// Lấy thời điểm hiện tại theo UTC.
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
            .map(|l| *l)
            .unwrap_or_else(|_| Utc::now())
    }
}

/// Cấu hình vòng lặp worker nền reports.
#[derive(Clone, Copy, Debug)]
pub struct ReportsWorkerConfig {
    /// Khoảng thời gian giữa các chu kỳ kiểm tra.
    pub check_interval: Duration,
}

impl Default for ReportsWorkerConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(60),
        }
    }
}

/// Chạy vòng lặp worker nền cho module reports (dừng an toàn khi token bị hủy).
pub async fn run_reports_worker(
    pool: SqlitePool,
    clock: Arc<dyn Clock>,
    token: CancellationToken,
    event_bus: Arc<EventBus>,
    config: ReportsWorkerConfig,
) {
    info!("Bắt đầu reports_worker nền");

    while !token.is_cancelled() {
        let now = clock.now();
        if let Err(e) = process_report_snapshots(&pool, now, &event_bus).await {
            error!("Lỗi xử lý tự động chụp báo cáo: {e}");
        }

        tokio::select! {
            () = token.cancelled() => {
                info!("reports_worker nhận tín hiệu dừng an toàn");
                break;
            }
            () = tokio::time::sleep(config.check_interval) => {}
        }
    }

    info!("reports_worker đã kết thúc");
}

/// Kiểm tra cấu hình và chụp báo cáo ngày/tuần tự động cho các workspace đã bật tính năng.
pub async fn process_report_snapshots(
    pool: &SqlitePool,
    now: DateTime<Utc>,
    event_bus: &EventBus,
) -> Result<(), sqlx::Error> {
    let configs: Vec<ReportConfig> = sqlx::query_as(
        r#"
        SELECT workspace_id, auto_generate, timezone, daily_time, weekly_day, weekly_time, updated_at
        FROM m_report_config
        WHERE auto_generate = 1
        "#,
    )
    .fetch_all(pool)
    .await?;

    for cfg in configs {
        let tz = parse_timezone(&cfg.timezone);
        let local_now = now.with_timezone(&tz);
        let current_time_str = format!("{:02}:{:02}", local_now.hour(), local_now.minute());

        // 1. Kiểm tra chụp báo cáo ngày
        if current_time_str == cfg.daily_time {
            let (start_utc, end_utc) = calculate_period_bounds(now, "daily", tz);
            if !has_snapshot_for_period(pool, &cfg.workspace_id, "daily", &start_utc).await? {
                create_and_save_snapshot(
                    pool,
                    &cfg.workspace_id,
                    "daily",
                    start_utc,
                    end_utc,
                    &cfg.timezone,
                    event_bus,
                )
                .await?;
            }
        }

        // 2. Kiểm tra chụp báo cáo tuần
        let current_weekday = local_now.weekday().number_from_monday() as i64;
        if current_weekday == cfg.weekly_day && current_time_str == cfg.weekly_time {
            let (start_utc, end_utc) = calculate_period_bounds(now, "weekly", tz);
            if !has_snapshot_for_period(pool, &cfg.workspace_id, "weekly", &start_utc).await? {
                create_and_save_snapshot(
                    pool,
                    &cfg.workspace_id,
                    "weekly",
                    start_utc,
                    end_utc,
                    &cfg.timezone,
                    event_bus,
                )
                .await?;
            }
        }
    }

    Ok(())
}

async fn has_snapshot_for_period(
    pool: &SqlitePool,
    workspace_id: &str,
    period_type: &str,
    start_utc: &DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let start_str = start_utc.to_rfc3339();
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM m_report_snapshot
        WHERE workspace_id = ? AND period_type = ? AND period_start = ?
        "#,
    )
    .bind(workspace_id)
    .bind(period_type)
    .bind(&start_str)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}

#[expect(
    clippy::too_many_lines,
    reason = "Tạo và lưu trữ bản chụp snapshot báo cáo"
)]
async fn create_and_save_snapshot(
    pool: &SqlitePool,
    workspace_id: &str,
    period_type: &str,
    start_utc: DateTime<Utc>,
    end_utc: DateTime<Utc>,
    timezone: &str,
    event_bus: &EventBus,
) -> Result<(), sqlx::Error> {
    let data = generate_report_data(
        pool,
        workspace_id,
        period_type,
        start_utc,
        end_utc,
        timezone,
    )
    .await?;
    let md = export_report_to_markdown(&data);
    let json_str = serde_json::to_string(&data).unwrap_or_default();
    let snapshot_id = Uuid::new_v4().to_string();
    let now_str = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO m_report_snapshot (
            id, workspace_id, period_type, period_start, period_end, timezone, data_json, markdown, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&snapshot_id)
    .bind(workspace_id)
    .bind(period_type)
    .bind(start_utc.to_rfc3339())
    .bind(end_utc.to_rfc3339())
    .bind(timezone)
    .bind(&json_str)
    .bind(&md)
    .bind(&now_str)
    .execute(pool)
    .await?;

    // Phát sự kiện thông báo báo cáo mới
    let ws_uuid =
        WorkspaceId::from_str(workspace_id).unwrap_or_else(|_| WorkspaceId::from(Uuid::nil()));
    let payload = EventPayload::NotificationRequested {
        title: format!(
            "Báo cáo tổng kết {} mới đã sẵn sàng",
            if period_type == "weekly" {
                "tuần"
            } else {
                "ngày"
            }
        ),
        message: format!(
            "Báo cáo tổng kết kỳ {} cho workspace vừa được tạo tự động.",
            period_type
        ),
        level: "info".to_string(),
    };
    event_bus.publish(Envelope::new(
        Utc::now(),
        EventScope::workspace(ws_uuid),
        payload,
    ));

    Ok(())
}

#![expect(clippy::unwrap_used, reason = "Kiểm thử cho phép unwrap")]

//! Các bài kiểm thử đơn vị cho module `reports`.

use chrono::{FixedOffset, TimeZone, Utc};
use dandori_module_api::EventBus;
use mod_reports::generator::{
    calculate_median, calculate_period_bounds, export_report_to_markdown, generate_report_data,
    redact_secrets,
};
use mod_reports::worker::{MockClock, ReportsWorkerConfig, run_reports_worker};
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

async fn setup_test_db() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE workspace (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE TABLE project (
            id TEXT PRIMARY KEY,
            workspace_id TEXT NOT NULL,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE TABLE task (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            status TEXT NOT NULL,
            lead_agent TEXT NOT NULL,
            worker_agent TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE run (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            agent TEXT NOT NULL,
            account TEXT NOT NULL,
            status TEXT NOT NULL,
            started_at TEXT NOT NULL,
            finished_at TEXT,
            exit_code INTEGER
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Chạy migration của reports
    let migration_sql = include_str!("../migrations/0001_reports.sql");
    sqlx::query(migration_sql).execute(&pool).await.unwrap();

    pool
}

#[tokio::test]
async fn test_ac1_report_counts_and_median_duration_calculated_accurately() {
    let pool = setup_test_db().await;

    // 1. Tạo workspace và project
    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES ('ws-1', 'WS 1', '2026-11-01T00:00:00Z')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO project (id, workspace_id, name, created_at) VALUES ('prj-1', 'ws-1', 'Prj 1', '2026-11-01T00:00:00Z')")
        .execute(&pool)
        .await
        .unwrap();

    // 2. Thêm 4 task với trạng thái khác nhau
    // Task 1: done (thời lượng 100s)
    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES ('t-1', 'prj-1', 'Task 1', 'desc', 'done', 'lead', 'worker', '2026-11-02T01:00:00Z', '2026-11-02T01:10:00Z')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) VALUES ('r-1', 't-1', 'claude', 'acc-1', 'success', '2026-11-02T01:00:00Z', '2026-11-02T01:01:40Z', 0)").execute(&pool).await.unwrap();

    // Task 2: failed (thời lượng 50s)
    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES ('t-2', 'prj-1', 'Task 2', 'desc', 'failed', 'lead', 'worker', '2026-11-02T02:00:00Z', '2026-11-02T02:10:00Z')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) VALUES ('r-2', 't-2', 'claude', 'acc-1', 'failed', '2026-11-02T02:00:00Z', '2026-11-02T02:00:50Z', 1)").execute(&pool).await.unwrap();

    // Task 3: needs_human (thời lượng 200s)
    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES ('t-3', 'prj-1', 'Task 3', 'desc', 'needs_human', 'lead', 'worker', '2026-11-02T03:00:00Z', '2026-11-02T03:10:00Z')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) VALUES ('r-3', 't-3', 'gemini', 'acc-2', 'needs_human', '2026-11-02T03:00:00Z', '2026-11-02T03:03:20Z', 0)").execute(&pool).await.unwrap();

    // Task 4: in_progress (đang chạy, chưa có finished_at)
    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES ('t-4', 'prj-1', 'Task 4', 'desc', 'in_progress', 'lead', 'worker', '2026-11-02T04:00:00Z', '2026-11-02T04:05:00Z')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) VALUES ('r-4', 't-4', 'codex', 'acc-3', 'running', '2026-11-02T04:00:00Z', NULL, NULL)").execute(&pool).await.unwrap();

    let start = Utc.with_ymd_and_hms(2026, 11, 2, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 11, 2, 23, 59, 59).unwrap();

    let report = generate_report_data(&pool, "ws-1", "daily", start, end, "UTC")
        .await
        .unwrap();

    // Kiểm tra tính toán thủ công độc lập:
    // Tổng số: 4, hoàn thành: 1, thất bại: 1, cần bạn: 1, đang chạy: 1
    assert_eq!(report.counts.total, 4);
    assert_eq!(report.counts.completed, 1);
    assert_eq!(report.counts.failed, 1);
    assert_eq!(report.counts.needs_human, 1);
    assert_eq!(report.counts.running, 1);

    // Thời lượng chạy: [50, 100, 200]
    // Tổng: 350s, Trung vị (median) của 3 phần tử lẻ là phần tử giữa: 100s, Trung bình: 350 / 3 = 116s
    assert_eq!(report.duration.total_duration_secs, 350);
    assert_eq!(report.duration.median_duration_secs, 100);
    assert_eq!(report.duration.avg_duration_secs, 116);

    // Sử dụng account:
    // claude:acc-1 (2 lần), gemini:acc-2 (1 lần), codex:acc-3 (1 lần)
    assert_eq!(report.usage.len(), 3);
    assert_eq!(report.usage[0].agent, "claude");
    assert_eq!(report.usage[0].account, "acc-1");
    assert_eq!(report.usage[0].run_count, 2);
}

#[test]
fn test_ac1_median_even_and_odd_samples() {
    let odd = vec![10, 20, 30];
    assert_eq!(calculate_median(&odd), 20);

    let even = vec![10, 20, 30, 40];
    assert_eq!(calculate_median(&even), 25);

    let single = vec![42];
    assert_eq!(calculate_median(&single), 42);

    let empty: Vec<i64> = vec![];
    assert_eq!(calculate_median(&empty), 0);
}

#[tokio::test]
async fn test_ac2_missing_source_module_clearly_states_no_data() {
    let pool = setup_test_db().await;
    let start = Utc.with_ymd_and_hms(2026, 11, 2, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 11, 2, 23, 59, 59).unwrap();

    let report = generate_report_data(&pool, "ws-1", "daily", start, end, "UTC")
        .await
        .unwrap();

    assert!(!report.cost.has_data);
    assert!(report.cost.total_cost.is_none());
    assert_eq!(
        report.cost.missing_reason,
        Some("module cost-tracking chưa kích hoạt".to_string())
    );

    let md = export_report_to_markdown(&report);
    assert!(md.contains("Chi phí: module cost-tracking chưa kích hoạt"));
    assert!(md.contains("Không có số liệu chuyển đổi tài khoản"));
}

#[test]
fn test_ac3_period_bounds_calculation_with_timezone() {
    // Thời điểm: 2026-11-02 03:00:00 UTC -> tương đương 2026-11-02 10:00:00 tại +07:00
    let now = Utc.with_ymd_and_hms(2026, 11, 2, 3, 0, 0).unwrap();
    let tz_vn = FixedOffset::east_opt(7 * 3600).unwrap();

    let (start_utc, end_utc) = calculate_period_bounds(now, "daily", tz_vn);

    // Ngày 2026-11-02 tại +07:00 bắt đầu lúc 00:00:00 (+07:00) -> 2026-11-01 17:00:00 UTC
    let expected_start = Utc.with_ymd_and_hms(2026, 11, 1, 17, 0, 0).unwrap();
    // Kết thúc lúc 23:59:59 (+07:00) -> 2026-11-02 16:59:59 UTC
    let expected_end = Utc.with_ymd_and_hms(2026, 11, 2, 16, 59, 59).unwrap();

    assert_eq!(start_utc, expected_start);
    assert_eq!(end_utc, expected_end);
}

#[test]
fn test_ac4_markdown_export_redacts_secrets_and_private_keys() {
    let key_header = format!("-----BEGIN {} PRIVATE KEY-----", "RSA");
    let key_footer = format!("-----END {} PRIVATE KEY-----", "RSA");
    let input = format!(
        "# Báo cáo\nAPI Key OpenAI: `sk-live-1234567890abcdef12345678`\nGitHub Token: ghp_9876543210fedcba9876\nKhóa riêng tư:\n{}\nMIIEowIBAAKCAQEA0Y1v...\n{}\nĐã hoàn tất.\n",
        key_header, key_footer
    );

    let redacted = redact_secrets(&input);

    assert!(!redacted.contains("sk-live-1234567890abcdef12345678"));
    assert!(!redacted.contains("ghp_9876543210fedcba9876"));
    assert!(!redacted.contains("MIIEowIBAAKCAQEA0Y1v"));
    assert!(redacted.contains("[REDACTED_SECRET]"));
    assert!(redacted.contains("[REDACTED_PRIVATE_KEY]"));
}

#[tokio::test]
async fn test_ac6_worker_stops_safely_when_cancelled() {
    let pool = setup_test_db().await;
    let event_bus = Arc::new(EventBus::new(100));
    let clock = Arc::new(MockClock::new(Utc::now()));
    let token = CancellationToken::new();

    let config = ReportsWorkerConfig {
        check_interval: Duration::from_millis(50),
    };

    let worker_handle = tokio::spawn({
        let pool = pool.clone();
        let token = token.clone();
        async move {
            run_reports_worker(pool, clock, token, event_bus, config).await;
        }
    });

    // Worker đang chạy
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    assert!(!worker_handle.is_finished());

    // Hủy token (tương đương khi disable module)
    token.cancel();

    // Worker phải kết thúc an toàn và không bị treo
    let res = tokio::time::timeout(tokio::time::Duration::from_millis(500), worker_handle).await;
    assert!(
        res.is_ok(),
        "Worker phải dừng an toàn khi nhận token cancel"
    );
}

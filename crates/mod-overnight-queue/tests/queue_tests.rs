//! Các bài kiểm thử đơn vị cho module `overnight-queue`.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử đơn vị overnight-queue kiểm tra nhiều luồng nghiệp vụ và dùng unwrap"
)]

use chrono::{TimeZone, Utc};
use dandori_module_api::{AgentRole, EventBus};
use mod_overnight_queue::model::OvernightQueueConfig;
use mod_overnight_queue::worker::{MockClock, process_overnight_queues};
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;

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
            repo_path TEXT NOT NULL,
            default_branch TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE TABLE task (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            status TEXT NOT NULL,
            lead_agent TEXT,
            worker_agent TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!("../migrations/0001_overnight_queue.sql"))
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES ('ws-1', 'WS 1', '2026-01-01T00:00:00Z')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES ('prj-1', 'ws-1', 'Prj 1', '/tmp', 'main', '2026-01-01T00:00:00Z')")
        .execute(&pool)
        .await
        .unwrap();

    pool
}

async fn insert_test_task(pool: &sqlx::SqlitePool, id: &str, title: &str, status: &str) {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES (?, 'prj-1', ?, 'desc', ?, 'lead', 'worker', ?, ?)"
    )
    .bind(id)
    .bind(title)
    .bind(status)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn test_ac1_start_time_triggers_queue_in_order_with_concurrency_limit() {
    let pool = setup_test_db().await;
    let event_bus = Arc::new(EventBus::new(100));

    // Thêm 3 task vào bảng task
    insert_test_task(&pool, "t-1", "Task 1", "backlog").await;
    insert_test_task(&pool, "t-2", "Task 2", "backlog").await;
    insert_test_task(&pool, "t-3", "Task 3", "backlog").await;

    // Cấu hình overnight-queue: start_time = "23:00", max_concurrency = 1
    sqlx::query(
        r#"
        INSERT INTO m_overnight_queue_config (
            workspace_id, start_time, timezone, max_concurrency,
            consecutive_failure_limit, status, consecutive_failures, updated_at
        ) VALUES ('ws-1', '23:00', 'UTC', 1, 3, 'idle', 0, '2026-01-01T00:00:00Z')
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Thêm các task vào hàng đợi với thứ tự 1, 2, 3
    sqlx::query("INSERT INTO m_overnight_queue_item (id, workspace_id, task_id, order_index, status, created_at, updated_at) VALUES ('qi-1', 'ws-1', 't-1', 1, 'queued', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO m_overnight_queue_item (id, workspace_id, task_id, order_index, status, created_at, updated_at) VALUES ('qi-2', 'ws-1', 't-2', 2, 'queued', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO m_overnight_queue_item (id, workspace_id, task_id, order_index, status, created_at, updated_at) VALUES ('qi-3', 'ws-1', 't-3', 3, 'queued', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')").execute(&pool).await.unwrap();

    // 1. Lúc 22:59 UTC: chưa tới giờ bắt đầu -> status vẫn là idle, chưa có task chạy
    let time_2259 = Utc.with_ymd_and_hms(2026, 11, 2, 22, 59, 0).unwrap();
    let clock = Arc::new(MockClock::new(time_2259));
    process_overnight_queues(&pool, &*clock, time_2259, &event_bus, None)
        .await
        .unwrap();

    let cfg: OvernightQueueConfig =
        sqlx::query_as("SELECT * FROM m_overnight_queue_config WHERE workspace_id = 'ws-1'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(cfg.status, "idle");

    // 2. Lúc 23:00 UTC: đúng giờ bắt đầu -> chuyển status = running và chạy tuần tự task 1
    let time_2300 = Utc.with_ymd_and_hms(2026, 11, 2, 23, 0, 0).unwrap();
    clock.set(time_2300);
    process_overnight_queues(&pool, &*clock, time_2300, &event_bus, None)
        .await
        .unwrap();

    let qi1_status: String =
        sqlx::query_scalar("SELECT status FROM m_overnight_queue_item WHERE id = 'qi-1'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(qi1_status, "completed");

    // Task 2 vẫn queued vì concurrency = 1
    let qi2_status: String =
        sqlx::query_scalar("SELECT status FROM m_overnight_queue_item WHERE id = 'qi-2'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(qi2_status, "queued");

    // 3. Tick tiếp theo: chạy task 2
    process_overnight_queues(&pool, &*clock, time_2300, &event_bus, None)
        .await
        .unwrap();
    let qi2_status: String =
        sqlx::query_scalar("SELECT status FROM m_overnight_queue_item WHERE id = 'qi-2'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(qi2_status, "completed");

    // 4. Tick tiếp theo: chạy task 3
    process_overnight_queues(&pool, &*clock, time_2300, &event_bus, None)
        .await
        .unwrap();
    let qi3_status: String =
        sqlx::query_scalar("SELECT status FROM m_overnight_queue_item WHERE id = 'qi-3'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(qi3_status, "completed");

    // 5. Tick tiếp theo: hoàn tất hàng đợi, tổng hợp tóm tắt sáng
    process_overnight_queues(&pool, &*clock, time_2300, &event_bus, None)
        .await
        .unwrap();
    let cfg: OvernightQueueConfig =
        sqlx::query_as("SELECT * FROM m_overnight_queue_config WHERE workspace_id = 'ws-1'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(cfg.status, "idle");
    assert!(cfg.last_run_summary.is_some());
}

#[tokio::test]
async fn test_ac2_safe_stop_after_consecutive_failures() {
    let pool = setup_test_db().await;
    let event_bus = Arc::new(EventBus::new(100));

    insert_test_task(&pool, "t-f1", "Fail 1", "backlog").await;
    insert_test_task(&pool, "t-f2", "Fail 2", "backlog").await;
    insert_test_task(&pool, "t-f3", "Fail 3", "backlog").await;
    insert_test_task(&pool, "t-ok", "Pending Task", "backlog").await;

    // consecutive_failure_limit = 2
    sqlx::query(
        r#"
        INSERT INTO m_overnight_queue_config (
            workspace_id, start_time, timezone, max_concurrency,
            consecutive_failure_limit, status, consecutive_failures, updated_at
        ) VALUES ('ws-1', '23:00', 'UTC', 1, 2, 'running', 0, '2026-01-01T00:00:00Z')
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO m_overnight_queue_item (id, workspace_id, task_id, order_index, status, created_at, updated_at) VALUES ('qi-1', 'ws-1', 't-f1', 1, 'queued', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO m_overnight_queue_item (id, workspace_id, task_id, order_index, status, created_at, updated_at) VALUES ('qi-2', 'ws-1', 't-f2', 2, 'queued', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO m_overnight_queue_item (id, workspace_id, task_id, order_index, status, created_at, updated_at) VALUES ('qi-3', 'ws-1', 't-ok', 3, 'queued', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')").execute(&pool).await.unwrap();

    // Mock runner luôn trả về lỗi
    struct FailingRunner;
    impl std::fmt::Debug for FailingRunner {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "FailingRunner")
        }
    }
    impl dandori_module_api::TaskRunner for FailingRunner {
        fn start_run(
            &self,
            _task_id: dandori_types::TaskId,
            _role: AgentRole,
            _user_id: dandori_types::UserId,
            _exclude: Vec<dandori_types::AgentAccountId>,
        ) -> dandori_module_api::BoxFuture<'static, Result<dandori_types::Run, String>> {
            Box::pin(async { Err("Hết hạn mức tài khoản / lỗi thực thi".to_string()) })
        }
    }

    let runner = FailingRunner;
    let clock = Arc::new(MockClock::new(Utc::now()));

    // Lần 1: Task 1 thất bại -> consecutive_failures = 1
    process_overnight_queues(&pool, &*clock, Utc::now(), &event_bus, Some(&runner))
        .await
        .unwrap();
    let cfg: OvernightQueueConfig =
        sqlx::query_as("SELECT * FROM m_overnight_queue_config WHERE workspace_id = 'ws-1'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(cfg.status, "running");
    assert_eq!(cfg.consecutive_failures, 1);

    // Lần 2: Task 2 thất bại -> chạm giới hạn 2 thất bại liên tiếp -> status chuyển thành 'stopped' (DỪNG AN TOÀN)
    process_overnight_queues(&pool, &*clock, Utc::now(), &event_bus, Some(&runner))
        .await
        .unwrap();
    let cfg: OvernightQueueConfig =
        sqlx::query_as("SELECT * FROM m_overnight_queue_config WHERE workspace_id = 'ws-1'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(cfg.status, "stopped");
    assert_eq!(cfg.consecutive_failures, 2);

    // Task 3 còn lại trong hàng đợi vẫn giữ nguyên trạng thái 'queued', không bị chạy
    let qi3_status: String =
        sqlx::query_scalar("SELECT status FROM m_overnight_queue_item WHERE id = 'qi-3'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(qi3_status, "queued");
}

#[tokio::test]
async fn test_ac3_restart_resumes_without_duplicating_completed_tasks() {
    let pool = setup_test_db().await;
    let event_bus = Arc::new(EventBus::new(100));

    insert_test_task(&pool, "t-done", "Task Done", "done").await;
    insert_test_task(&pool, "t-next", "Task Next", "backlog").await;

    sqlx::query(
        r#"
        INSERT INTO m_overnight_queue_config (
            workspace_id, start_time, timezone, max_concurrency,
            consecutive_failure_limit, status, consecutive_failures, updated_at
        ) VALUES ('ws-1', '23:00', 'UTC', 1, 3, 'running', 0, '2026-01-01T00:00:00Z')
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Item 1 đã hoàn thành trước đó
    sqlx::query("INSERT INTO m_overnight_queue_item (id, workspace_id, task_id, order_index, status, created_at, updated_at) VALUES ('qi-done', 'ws-1', 't-done', 1, 'completed', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')").execute(&pool).await.unwrap();
    // Item 2 đang chờ chạy
    sqlx::query("INSERT INTO m_overnight_queue_item (id, workspace_id, task_id, order_index, status, created_at, updated_at) VALUES ('qi-next', 'ws-1', 't-next', 2, 'queued', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')").execute(&pool).await.unwrap();

    let clock = Arc::new(MockClock::new(Utc::now()));
    process_overnight_queues(&pool, &*clock, Utc::now(), &event_bus, None)
        .await
        .unwrap();

    // Item 2 được chạy hoàn thành, Item 1 vẫn giữ nguyên completed
    let qi_next_status: String =
        sqlx::query_scalar("SELECT status FROM m_overnight_queue_item WHERE id = 'qi-next'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(qi_next_status, "completed");

    let qi_done_status: String =
        sqlx::query_scalar("SELECT status FROM m_overnight_queue_item WHERE id = 'qi-done'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(qi_done_status, "completed");
}

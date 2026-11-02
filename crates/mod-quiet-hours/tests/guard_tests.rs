//! Kiểm thử đơn vị cho Guard của module `quiet-hours`.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Test đơn vị cho guard kiểm tra nhiều luồng dữ liệu và dùng unwrap trong test"
)]

use chrono::{TimeZone, Utc};
use dandori_module_api::{GuardDecision, SystemSource, TransitionActor, TransitionGuard};
use dandori_types::{ProjectId, Task, TaskId, TaskStatus};
use mod_quiet_hours::guard::{MockClock, QuietHoursGuard};
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;

async fn setup_test_db() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    // Migration cốt lõi
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

    // Migration của quiet-hours
    sqlx::query(include_str!("../migrations/0001_quiet_hours.sql"))
        .execute(&pool)
        .await
        .unwrap();

    // Dữ liệu mẫu
    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES ('ws-1', 'WS 1', '2026-01-01T00:00:00Z')")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES ('prj-1', 'ws-1', 'Project 1', '/tmp', 'main', '2026-01-01T00:00:00Z')"
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

fn make_task(project_id: &str, status: TaskStatus) -> Task {
    Task {
        id: TaskId::new(),
        project_id: project_id.parse::<ProjectId>().unwrap(),
        title: "Test Task".to_string(),
        description: "Test".to_string(),
        status: status.as_str().to_string(),
        lead_agent: "lead".to_string(),
        worker_agent: "worker".to_string(),
        workdir_path: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[tokio::test]
async fn test_scheduler_in_quiet_window_denied_with_retry_after() {
    let pool = setup_test_db().await;

    // Cấu hình khung cấm: Thứ 2 (1), từ 09:00 đến 18:00 UTC (áp dụng toàn workspace ws-1)
    sqlx::query(
        r#"
        INSERT INTO m_quiet_hours_window (
            id, workspace_id, project_id, name, days_of_week, start_time,
            end_time, timezone, block_manual_runs, enabled, created_at, updated_at
        ) VALUES (
            'w-1', 'ws-1', NULL, 'Giờ làm việc', '1', '09:00',
            '18:00', 'UTC', 0, 1, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Giả lập thời gian: Thứ 2 (2026-11-02) lúc 10:00 UTC (nằm trong khung 09:00 - 18:00)
    let mon_10am = Utc.with_ymd_and_hms(2026, 11, 2, 10, 0, 0).unwrap();
    let clock = Arc::new(MockClock::new(mon_10am));
    let guard = QuietHoursGuard::new(pool.clone(), clock.clone());

    let task = make_task("00000000-0000-0000-0000-000000000001", TaskStatus::Backlog);
    // Cập nhật project id khớp với prj-1
    sqlx::query("UPDATE project SET id = ? WHERE id = 'prj-1'")
        .bind(task.project_id.to_string())
        .execute(&pool)
        .await
        .unwrap();

    // 1. Tác nhân scheduler:calendar -> Bị Deny kèm retry_after = 18:00
    let actor_sched = TransitionActor::System(SystemSource::Scheduler("calendar".to_string()));
    let decision = guard
        .check(
            &task,
            TaskStatus::Backlog,
            TaskStatus::Running,
            &actor_sched,
        )
        .await
        .unwrap();

    let expected_end = Utc.with_ymd_and_hms(2026, 11, 2, 18, 0, 0).unwrap();
    match decision {
        GuardDecision::Deny {
            reason,
            retry_after,
        } => {
            assert!(reason.contains("bộ lập lịch (calendar)"));
            assert_eq!(retry_after, Some(expected_end));
        }
        other => panic!("Kỳ vọng Deny nhưng nhận được: {other:?}"),
    }

    // 2. Chuyển đồng hồ ra ngoài khung: Thứ 2 lúc 18:01 UTC -> Cho phép
    clock.set(Utc.with_ymd_and_hms(2026, 11, 2, 18, 1, 0).unwrap());
    let decision = guard
        .check(
            &task,
            TaskStatus::Backlog,
            TaskStatus::Running,
            &actor_sched,
        )
        .await
        .unwrap();
    assert_eq!(decision, GuardDecision::Allow);
}

#[tokio::test]
async fn test_manual_run_allowed_by_default_and_denied_when_configured() {
    let pool = setup_test_db().await;

    // Cấu hình khung cấm: Thứ 2 (1), từ 09:00 đến 18:00 UTC, block_manual_runs = 0
    sqlx::query(
        r#"
        INSERT INTO m_quiet_hours_window (
            id, workspace_id, project_id, name, days_of_week, start_time,
            end_time, timezone, block_manual_runs, enabled, created_at, updated_at
        ) VALUES (
            'w-1', 'ws-1', NULL, 'Giờ làm việc', '1', '09:00',
            '18:00', 'UTC', 0, 1, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let mon_10am = Utc.with_ymd_and_hms(2026, 11, 2, 10, 0, 0).unwrap();
    let clock = Arc::new(MockClock::new(mon_10am));
    let guard = QuietHoursGuard::new(pool.clone(), clock);

    let task = make_task("00000000-0000-0000-0000-000000000002", TaskStatus::Backlog);
    sqlx::query("UPDATE project SET id = ? WHERE id = 'prj-1'")
        .bind(task.project_id.to_string())
        .execute(&pool)
        .await
        .unwrap();

    let actor_user = TransitionActor::User("user-1".to_string());

    // AC 3: Chạy tay mặc định KHÔNG bị chặn
    let decision = guard
        .check(&task, TaskStatus::Backlog, TaskStatus::Running, &actor_user)
        .await
        .unwrap();
    assert_eq!(decision, GuardDecision::Allow);

    // Bật tùy chọn block_manual_runs = 1
    sqlx::query("UPDATE m_quiet_hours_window SET block_manual_runs = 1 WHERE id = 'w-1'")
        .execute(&pool)
        .await
        .unwrap();

    // AC 3: Chạy tay BỊ CHẶN khi tùy chọn bật
    let decision = guard
        .check(&task, TaskStatus::Backlog, TaskStatus::Running, &actor_user)
        .await
        .unwrap();
    match decision {
        GuardDecision::Deny {
            reason,
            retry_after,
        } => {
            assert!(reason.contains("chạy thủ công"));
            let expected_end = Utc.with_ymd_and_hms(2026, 11, 2, 18, 0, 0).unwrap();
            assert_eq!(retry_after, Some(expected_end));
        }
        other => panic!("Kỳ vọng Deny nhưng nhận được: {other:?}"),
    }
}

#[tokio::test]
async fn test_disabled_window_allows_everything() {
    let pool = setup_test_db().await;

    // Window bị tắt (enabled = 0)
    sqlx::query(
        r#"
        INSERT INTO m_quiet_hours_window (
            id, workspace_id, project_id, name, days_of_week, start_time,
            end_time, timezone, block_manual_runs, enabled, created_at, updated_at
        ) VALUES (
            'w-1', 'ws-1', NULL, 'Giờ làm việc', '1', '09:00',
            '18:00', 'UTC', 1, 0, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let mon_10am = Utc.with_ymd_and_hms(2026, 11, 2, 10, 0, 0).unwrap();
    let clock = Arc::new(MockClock::new(mon_10am));
    let guard = QuietHoursGuard::new(pool.clone(), clock);

    let task = make_task("00000000-0000-0000-0000-000000000003", TaskStatus::Backlog);
    sqlx::query("UPDATE project SET id = ? WHERE id = 'prj-1'")
        .bind(task.project_id.to_string())
        .execute(&pool)
        .await
        .unwrap();

    let actor_sched = TransitionActor::System(SystemSource::Scheduler("calendar".to_string()));
    let decision = guard
        .check(
            &task,
            TaskStatus::Backlog,
            TaskStatus::Running,
            &actor_sched,
        )
        .await
        .unwrap();
    assert_eq!(decision, GuardDecision::Allow);
}

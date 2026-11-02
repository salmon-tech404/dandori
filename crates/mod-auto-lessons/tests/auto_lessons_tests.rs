//! Kiểm thử đơn vị cho crate `mod-auto-lessons` (P3-11).

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử đơn vị mod-auto-lessons có các bài test dài"
)]

use chrono::Utc;
use dandori_module_api::Module;
use dandori_types::{Envelope, EventPayload, EventScope, ProjectId, TaskId, WorkspaceId};
use mod_auto_lessons::AutoLessonsModule;
use mod_auto_lessons::redaction::{redact_multiline_private_keys, redact_secrets};
use mod_auto_lessons::worker::{handle_task_status_changed, is_task_already_processed};
use sqlx::SqlitePool;

async fn setup_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let core_schema = include_str!("../../../crates/dandori-db/migrations/0001_core_schema.sql");
    sqlx::query(core_schema).execute(&pool).await.unwrap();

    let shared_mem_schema =
        include_str!("../../../crates/mod-shared-memory/migrations/0001_shared_memory_schema.sql");
    sqlx::query(shared_mem_schema).execute(&pool).await.unwrap();

    let module = AutoLessonsModule::new();
    for migration in module.migrations() {
        sqlx::query(migration.sql).execute(&pool).await.unwrap();
    }

    pool
}

#[tokio::test]
async fn test_manifest_and_migrations() {
    let module = AutoLessonsModule::new();
    let manifest = module.manifest();
    assert_eq!(manifest.id, "auto-lessons");
    assert_eq!(manifest.version, "0.1.0");
    assert!(manifest.requires.contains(&"shared-memory".to_string()));

    let pool = setup_db().await;
    let count_config: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM m_auto_lessons_config")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count_config.0, 0);

    let count_run: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM m_auto_lessons_run")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count_run.0, 0);
}

#[test]
fn test_secret_redaction_multiline_private_key_and_tokens() {
    let key_type = "RSA";
    let fake_private_key = format!(
        "-----BEGIN {} PRIVATE KEY-----\nMIIEowIBAAKCAQEA0m...\n-----END {} PRIVATE KEY-----",
        key_type, key_type
    );
    let sample_text = format!(
        "Lỗi kết nối xảy ra với khóa: {}\nToken: sk-1234567890abcdef1234567890\nHoàn thành.",
        fake_private_key
    );

    let redacted_key = redact_multiline_private_keys(&sample_text);
    assert!(!redacted_key.contains("MIIEowIBAAKCAQEA0m"));
    assert!(redacted_key.contains("[REDACTED_PRIVATE_KEY]"));

    let full_redacted = redact_secrets(&sample_text);
    assert!(!full_redacted.contains("sk-1234567890abcdef1234567890"));
    assert!(full_redacted.contains("[REDACTED_SECRET]"));
}

#[tokio::test]
async fn test_skip_trivial_task() {
    let pool = setup_db().await;
    let now = Utc::now().to_rfc3339();

    let ws_id = WorkspaceId::new();
    let prj_id = ProjectId::new();
    let t_id = TaskId::new();

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS AL', ?)")
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'P AL', '.', 'main', ?)")
        .bind(prj_id.to_string())
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    // Task rất ngắn (dưới 10 ký tự)
    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES (?, ?, 'abc', '', 'backlog', 'lead', 'worker', ?, ?)")
        .bind(t_id.to_string())
        .bind(prj_id.to_string())
        .bind(&now)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let env = Envelope::new(
        Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::TaskStatusChanged {
            task_id: t_id,
            old_status: "running".to_string(),
            new_status: "done".to_string(),
        },
    );

    handle_task_status_changed(&pool, None, &env, &t_id, "done").await;

    // Phải ghi nhận status = 'skipped' và KHÔNG tạo mục nhớ nào trong m_shared_memory_entry
    let run_row: (String, Option<String>) =
        sqlx::query_as("SELECT status, reason FROM m_auto_lessons_run WHERE task_id = ?")
            .bind(t_id.to_string())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(run_row.0, "skipped");
    assert!(run_row.1.unwrap_or_default().contains("quá ngắn"));

    let memory_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM m_shared_memory_entry")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(memory_count.0, 0, "Task quá ngắn không được tạo bài học");
}

#[tokio::test]
async fn test_anti_loop_and_idempotency() {
    let pool = setup_db().await;
    let now = Utc::now().to_rfc3339();

    let ws_id = WorkspaceId::new();
    let prj_id = ProjectId::new();
    let t_id = TaskId::new();

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS AL', ?)")
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'P AL', '.', 'main', ?)")
        .bind(prj_id.to_string())
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES (?, ?, 'Triển khai tính năng đăng nhập SSO', 'Cần kiểm tra OAuth provider', 'backlog', 'lead', 'worker', ?, ?)")
        .bind(t_id.to_string())
        .bind(prj_id.to_string())
        .bind(&now)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let env = Envelope::new(
        Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::TaskStatusChanged {
            task_id: t_id,
            old_status: "running".to_string(),
            new_status: "done".to_string(),
        },
    );

    // Kích hoạt lần 1
    handle_task_status_changed(&pool, None, &env, &t_id, "done").await;

    assert!(is_task_already_processed(&pool, &t_id.to_string()).await);

    let count_1: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM m_shared_memory_entry")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count_1.0, 1);

    // Kích hoạt lần 2 trên cùng task (chống lặp)
    handle_task_status_changed(&pool, None, &env, &t_id, "done").await;

    let count_2: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM m_shared_memory_entry")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count_2.0, 1, "Chống lặp: Không được tạo thêm bài học");

    let runs_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM m_auto_lessons_run WHERE task_id = ?")
            .bind(t_id.to_string())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        runs_count.0, 1,
        "Tối đa đúng một lần chạy bài học cho mỗi task"
    );
}

#[tokio::test]
async fn test_task_done_creates_pending_lesson_with_redacted_secrets() {
    let pool = setup_db().await;
    let now = Utc::now().to_rfc3339();

    let ws_id = WorkspaceId::new();
    let prj_id = ProjectId::new();
    let t_id = TaskId::new();

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS AL', ?)")
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'P AL', '.', 'main', ?)")
        .bind(prj_id.to_string())
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let key_type = "RSA";
    let fake_key = format!(
        "-----BEGIN {} PRIVATE KEY-----\nMIIEowIBAAKCAQEA0m...\n-----END {} PRIVATE KEY-----",
        key_type, key_type
    );
    let desc = format!(
        "Đã khắc phục lỗi token sk-9999999999abcdef8888888888 và {}",
        fake_key
    );

    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES (?, ?, 'Sửa lỗi rò rỉ khóa xác thực', ?, 'backlog', 'lead', 'worker', ?, ?)")
        .bind(t_id.to_string())
        .bind(prj_id.to_string())
        .bind(&desc)
        .bind(&now)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let env = Envelope::new(
        Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::TaskStatusChanged {
            task_id: t_id,
            old_status: "running".to_string(),
            new_status: "done".to_string(),
        },
    );

    handle_task_status_changed(&pool, None, &env, &t_id, "done").await;

    // Kiểm tra mục bài học tạo ra
    let memory: (String, String, String, String) = sqlx::query_as(
        "SELECT category, content, status, source_type FROM m_shared_memory_entry WHERE project_id = ?",
    )
    .bind(prj_id.to_string())
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(memory.0, "lesson", "Loại mục nhớ phải là lesson");
    assert_eq!(
        memory.2, "pending",
        "Mục nhớ phải ở trạng thái pending (chống tự duyệt)"
    );
    assert_eq!(memory.3, "agent", "Nguồn phải là agent");

    // Khóa và token bí mật phải được che
    assert!(!memory.1.contains("MIIEowIBAAKCAQEA0m"));
    assert!(!memory.1.contains("sk-9999999999abcdef8888888888"));
    assert!(memory.1.contains("[REDACTED_PRIVATE_KEY]"));
    assert!(memory.1.contains("[REDACTED_SECRET]"));
}

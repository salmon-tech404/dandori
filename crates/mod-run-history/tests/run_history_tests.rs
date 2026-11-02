//! Kiểm thử đơn vị và tích hợp cho crate `mod-run-history`.

#![expect(clippy::expect_used, reason = "Kiểm thử cho phép expect")]

use dandori_module_api::Module;
use mod_run_history::RunHistoryModule;
use mod_run_history::worker::cleanup_expired_batch;
use sqlx::SqlitePool;

#[tokio::test]
async fn test_manifest_and_migrations() {
    let module = RunHistoryModule::new();
    let manifest = module.manifest();
    assert_eq!(manifest.id, "run-history");
    assert_eq!(manifest.group, dandori_module_api::ModuleGroup::Dashboard);
    assert_eq!(manifest.ui.contributes.len(), 1);
    let first_ui = manifest
        .ui
        .contributes
        .first()
        .expect("Phải có ít nhất 1 UI contribute");
    assert_eq!(first_ui.slot, "board.task-panel.tab");

    let migrations = module.migrations();
    assert_eq!(migrations.len(), 1);
    let first_migration = migrations.first().expect("Phải có 1 migration");
    assert!(
        first_migration
            .sql
            .contains("CREATE TABLE IF NOT EXISTS m_run_history_note")
    );
}

async fn init_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Tạo pool memory");
    let core_schema = include_str!("../../../crates/dandori-db/migrations/0001_core_schema.sql");
    sqlx::query(core_schema)
        .execute(&pool)
        .await
        .expect("Chạy core schema");

    let module = RunHistoryModule::new();
    for m in module.migrations() {
        sqlx::query(m.sql)
            .execute(&pool)
            .await
            .expect("Chạy migration");
    }
    pool
}

async fn seed_test_hierarchy(pool: &SqlitePool, old_date: &str, recent_date: &str) {
    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES ('ws-1', 'WS', ?)")
        .bind(recent_date)
        .execute(pool)
        .await
        .expect("Tạo workspace");

    sqlx::query(
        "INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) \
         VALUES ('proj-1', 'ws-1', 'Proj', '/tmp', 'main', ?)",
    )
    .bind(recent_date)
    .execute(pool)
    .await
    .expect("Tạo project");

    sqlx::query(
        "INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) \
         VALUES ('task-closed', 'proj-1', 'Closed', 'desc', 'done', 'lead', 'worker', ?, ?)",
    )
    .bind(old_date)
    .bind(old_date)
    .execute(pool)
    .await
    .expect("Tạo task closed");

    sqlx::query(
        "INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) \
         VALUES ('task-open', 'proj-1', 'Open', 'desc', 'running', 'lead', 'worker', ?, ?)",
    )
    .bind(old_date)
    .bind(old_date)
    .execute(pool)
    .await
    .expect("Tạo task open");
}

async fn seed_test_runs_and_events(pool: &SqlitePool, old_date: &str, recent_date: &str) {
    sqlx::query(
        "INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) \
         VALUES ('run-old-closed', 'task-closed', 'claude', 'acc-1', 'success', ?, ?, 0)",
    )
    .bind(old_date)
    .bind(old_date)
    .execute(pool)
    .await
    .expect("Tạo run old closed");

    sqlx::query(
        "INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) \
         VALUES ('run-old-open', 'task-open', 'claude', 'acc-1', 'running', ?, NULL, NULL)",
    )
    .bind(old_date)
    .execute(pool)
    .await
    .expect("Tạo run old open");

    sqlx::query(
        "INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) \
         VALUES ('run-new-closed', 'task-closed', 'claude', 'acc-1', 'success', ?, ?, 0)",
    )
    .bind(recent_date)
    .bind(recent_date)
    .execute(pool)
    .await
    .expect("Tạo run new closed");

    sqlx::query(
        "INSERT INTO run_event (id, run_id, seq, event_type, data, created_at) \
         VALUES ('ev-old', 'run-old-closed', 1, 'log', 'old data', ?)",
    )
    .bind(old_date)
    .execute(pool)
    .await
    .expect("Tạo event old");

    sqlx::query(
        "INSERT INTO run_event (id, run_id, seq, event_type, data, created_at) \
         VALUES ('ev-new', 'run-new-closed', 1, 'log', 'new data', ?)",
    )
    .bind(recent_date)
    .execute(pool)
    .await
    .expect("Tạo event new");
}

#[tokio::test]
async fn test_cleanup_expired_batch_threshold_and_open_task_protection() {
    let pool = init_test_db().await;
    let now = chrono::Utc::now();
    let old_date = (now - chrono::Duration::days(100)).to_rfc3339();
    let recent_date = (now - chrono::Duration::days(2)).to_rfc3339();

    seed_test_hierarchy(&pool, &old_date, &recent_date).await;
    seed_test_runs_and_events(&pool, &old_date, &recent_date).await;

    let (deleted_events, deleted_runs) = cleanup_expired_batch(&pool, 90, 100)
        .await
        .expect("Dọn dẹp thành công");
    assert_eq!(deleted_events, 1, "Phải xóa đúng 1 event quá hạn");
    assert_eq!(
        deleted_runs, 1,
        "Phải xóa đúng 1 run quá hạn của task đã đóng"
    );

    let remaining_runs: Vec<String> = sqlx::query_scalar("SELECT id FROM run ORDER BY id ASC")
        .fetch_all(&pool)
        .await
        .expect("Truy vấn run còn lại");
    assert_eq!(remaining_runs, vec!["run-new-closed", "run-old-open"]);

    let remaining_events: Vec<String> = sqlx::query_scalar("SELECT id FROM run_event")
        .fetch_all(&pool)
        .await
        .expect("Truy vấn event còn lại");
    assert_eq!(remaining_events, vec!["ev-new"]);
}

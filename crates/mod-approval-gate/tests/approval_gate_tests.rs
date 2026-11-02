//! Kiểm thử đơn vị cho crate `mod-approval-gate`.

#![expect(
    clippy::too_many_lines,
    reason = "Kiểm thử đơn vị mod-approval-gate có hàm thiết lập dữ liệu dài"
)]

use chrono::{Duration, Utc};
use dandori_module_api::{EventBus, Module};
use mod_approval_gate::ApprovalGateModule;
use mod_approval_gate::worker::process_expired_requests_once;
use sqlx::SqlitePool;
use std::sync::Arc;

#[tokio::test]
async fn test_manifest_and_migrations() {
    let module = ApprovalGateModule::new();
    let manifest = module.manifest();
    assert_eq!(manifest.id, "approval-gate");
    assert_eq!(manifest.version, "0.1.0");

    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    for migration in module.migrations() {
        sqlx::query(migration.sql).execute(&pool).await.unwrap();
    }

    // Kiểm tra bảng đã tồn tại
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM m_approval_gate_request")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 0);
}

#[tokio::test]
async fn test_process_expired_requests() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let core_schema = include_str!("../../../crates/dandori-db/migrations/0001_core_schema.sql");
    sqlx::query(core_schema).execute(&pool).await.unwrap();

    let module = ApprovalGateModule::new();
    for migration in module.migrations() {
        sqlx::query(migration.sql).execute(&pool).await.unwrap();
    }

    let bus = Arc::new(EventBus::new(50));

    // Tạo project và task
    let ws_id = uuid::Uuid::new_v4().to_string();
    let prj_id = uuid::Uuid::new_v4().to_string();
    let task_id = uuid::Uuid::new_v4().to_string();
    let user_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query("INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, 'admin', 'hash', 'admin', ?)")
        .bind(&user_id).bind(&now).execute(&pool).await.unwrap();

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS', ?)")
        .bind(&ws_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'PRJ', '.', 'main', ?)")
        .bind(&prj_id).bind(&ws_id).bind(&now).execute(&pool).await.unwrap();

    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES (?, ?, 'Task 1', 'desc', 'needs_human', 'lead', 'worker', ?, ?)")
        .bind(&task_id).bind(&prj_id).bind(&now).bind(&now).execute(&pool).await.unwrap();

    // Chèn 1 request đã quá hạn và 1 request chưa quá hạn
    let past = (Utc::now() - Duration::hours(1)).to_rfc3339();
    let future = (Utc::now() + Duration::hours(1)).to_rfc3339();

    sqlx::query(
        "INSERT INTO m_approval_gate_request \
         (id, workspace_id, project_id, task_id, kind, from_status, target_status, requester_id, status, created_at, expires_at) \
         VALUES ('req-expired', ?, ?, ?, 'transition', 'in_review', 'ready_to_merge', 'u1', 'pending', ?, ?)",
    )
    .bind(&ws_id).bind(&prj_id).bind(&task_id).bind(&past).bind(&past).execute(&pool).await.unwrap();

    sqlx::query(
        "INSERT INTO m_approval_gate_request \
         (id, workspace_id, project_id, task_id, kind, from_status, target_status, requester_id, status, created_at, expires_at) \
         VALUES ('req-pending', ?, ?, ?, 'transition', 'in_review', 'ready_to_merge', 'u1', 'pending', ?, ?)",
    )
    .bind(&ws_id).bind(&prj_id).bind(&task_id).bind(&now).bind(&future).execute(&pool).await.unwrap();

    let processed = process_expired_requests_once(&pool, &bus).await.unwrap();
    assert_eq!(processed, 1);

    // Kiểm tra trạng thái req-expired thành expired
    let status: (String,) =
        sqlx::query_as("SELECT status FROM m_approval_gate_request WHERE id = 'req-expired'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status.0, "expired");

    // Kiểm tra task đã được hoàn tác về in_review
    let task_status: (String,) = sqlx::query_as("SELECT status FROM task WHERE id = ?")
        .bind(&task_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(task_status.0, "in_review");

    // req-pending vẫn pending
    let status_pending: (String,) =
        sqlx::query_as("SELECT status FROM m_approval_gate_request WHERE id = 'req-pending'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status_pending.0, "pending");
}

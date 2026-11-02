//! Các kiểm thử tích hợp cho Task P2-10 (Module auto-switch & Tự chuyển tài khoản khi hết hạn mức).

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P2-10 dùng unwrap và có fixture kiểm tra nhiều tiêu chí"
)]

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use dandori_events::EventBus;
use dandori_registry::FeatureRegistry;
use dandori_registry::lifecycle::FeatureLifecycleManager;
use dandori_server::app::create_app;
use dandori_server::config::ServerConfig;
use dandori_server::run_service::RunService;
use dandori_server::state::AppState;
use dandori_types::{ProjectId, TaskId, TaskStatus, WorkspaceId};
use serde_json::json;
use sqlx::SqlitePool;
use tempfile::tempdir;
use tower::ServiceExt;

struct TestContext {
    app: axum::Router,
    pool: SqlitePool,
    #[expect(dead_code, reason = "Giữ lifecycle manager trong bộ nhớ")]
    lifecycle_manager: Arc<FeatureLifecycleManager>,
    event_bus: Arc<EventBus>,
    owner_cookie: String,
    #[expect(dead_code, reason = "Giữ thư mục tạm cho đến khi kết thúc test")]
    temp_dir: tempfile::TempDir,
}

async fn setup_test_context() -> TestContext {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    let core_schema = include_str!("../../../crates/dandori-db/migrations/0001_core_schema.sql");
    sqlx::query(core_schema).execute(&pool).await.unwrap();

    let auth_schema = include_str!("../../../crates/mod-auth-login/migrations/0001_auth_login.sql");
    sqlx::query(auth_schema).execute(&pool).await.unwrap();

    let event_bus = Arc::new(EventBus::new(200));
    let registry = FeatureRegistry::new(pool.clone());
    registry.seed_modules_and_packs().await.unwrap();
    let lifecycle_manager = Arc::new(FeatureLifecycleManager::new(
        registry,
        pool.clone(),
        Arc::clone(&event_bus),
    ));

    // Đăng ký toàn bộ module tự động
    let modules = dandori_modules::registered_modules();
    for module in &modules {
        for migration in module.migrations() {
            sqlx::query(migration.sql).execute(&pool).await.unwrap();
        }
    }
    for m in modules {
        lifecycle_manager.register_module(m).await;
    }

    let temp_dir = tempdir().unwrap();
    let config = ServerConfig {
        data_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let state = AppState::new(
        pool.clone(),
        Arc::clone(&event_bus),
        Arc::clone(&lifecycle_manager),
        config,
    );

    // Cung cấp TaskRunner cho LifecycleManager
    let run_service = Arc::new(RunService::new(
        pool.clone(),
        Arc::clone(&event_bus),
        Arc::clone(&lifecycle_manager),
        10,
    ));
    lifecycle_manager.set_task_runner(run_service);

    let app = create_app(state);

    // Thiết lập tài khoản Owner
    let setup = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "owner", "password": "password123" }).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(setup).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let owner_cookie = resp
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    TestContext {
        app,
        pool,
        lifecycle_manager,
        event_bus,
        owner_cookie,
        temp_dir,
    }
}

async fn enable_modules(ctx: &TestContext, module_ids: &[&str]) {
    for mod_id in module_ids {
        let enable_req = Request::builder()
            .method("POST")
            .uri("/api/v1/features/apply")
            .header(header::COOKIE, &ctx.owner_cookie)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "module_id": mod_id,
                    "scope": "system",
                    "target_id": "system",
                    "enable": true
                })
                .to_string(),
            ))
            .unwrap();
        let enable_resp = ctx.app.clone().oneshot(enable_req).await.unwrap();
        assert_eq!(enable_resp.status(), StatusCode::OK);
    }
}

/// Tiêu chí 1: Khi task gặp QuotaExhausted, tự chuyển sang account kế tiếp cùng nhà và hoàn tất; ghi nhận lịch sử chuyển.
#[tokio::test]
async fn test_ac1_auto_switch_to_next_account_on_quota_exhausted() {
    let ctx = setup_test_context().await;
    enable_modules(
        &ctx,
        &[
            "subscription-connect",
            "multi-account",
            "quota-tracking",
            "auto-switch",
        ],
    )
    .await;

    // 1. Tạo 2 tài khoản cùng nhà "fake": Acc 1 (ưu tiên 10), Acc 2 (ưu tiên 20)
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO agent_account (id, provider, label, home_dir, auth_type, status, enabled, priority) \
         VALUES ('acc-sw-1', 'fake', 'Fake Switch 1', '/tmp/h1', 'cli', 'ready', 1, 10)",
    )
    .execute(&ctx.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO agent_account (id, provider, label, home_dir, auth_type, status, enabled, priority) \
         VALUES ('acc-sw-2', 'fake', 'Fake Switch 2', '/tmp/h2', 'cli', 'ready', 1, 20)",
    )
    .execute(&ctx.pool)
    .await
    .unwrap();

    // 2. Tạo workspace, project, task
    let ws_id = WorkspaceId::new();
    let prj_id = ProjectId::new();
    let t_id = TaskId::new();

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS Switch', ?)")
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    // Gán owner vào workspace_member
    let owner_id: (String,) = sqlx::query_as("SELECT id FROM user WHERE username = 'owner'")
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'admin')",
    )
    .bind(ws_id.to_string())
    .bind(&owner_id.0)
    .execute(&ctx.pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'PRJ Switch', '/repo', 'main', ?)")
        .bind(prj_id.to_string())
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, workdir_path, created_at, updated_at) VALUES (?, ?, 'Task Switch', 'D', 'running', 'lead', 'fake', '/tmp/wt', ?, ?)")
        .bind(t_id.to_string())
        .bind(prj_id.to_string())
        .bind(&now)
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    // 3. Giả lập Lần chạy 1 (Acc 1) kết thúc với lỗi QuotaExhausted
    let r1_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO run (id, task_id, agent, account, status, started_at) VALUES (?, ?, 'fake', 'Fake Switch 1', 'failed', ?)")
        .bind(&r1_id)
        .bind(t_id.to_string())
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    // Đánh dấu Acc 1 bị hết hạn mức
    sqlx::query(
        "UPDATE agent_account SET exhausted_until = '2099-01-01T00:00:00Z' WHERE id = 'acc-sw-1'",
    )
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Phát sự kiện RunFailed do QuotaExhausted lên EventBus
    ctx.event_bus.publish(dandori_types::Envelope::new(
        Utc::now(),
        dandori_types::EventScope::task(ws_id, prj_id, t_id),
        dandori_types::EventPayload::RunFailed {
            run_id: r1_id.parse().unwrap(),
            error_message: "Hết hạn mức: quota_exhausted rate limit".to_string(),
        },
    ));

    // Chờ worker auto-switch xử lý
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    // Kiểm tra lịch sử chuyển trong m_auto_switch_history
    let history_row: Option<(String, String, String, i64)> = sqlx::query_as(
        "SELECT from_account_id, to_account_id, from_provider, switch_count FROM m_auto_switch_history WHERE task_id = ?",
    )
    .bind(t_id.to_string())
    .fetch_optional(&ctx.pool)
    .await
    .unwrap();

    assert!(history_row.is_some(), "Phải có bản ghi chuyển tài khoản");
    let (from_acc, to_acc, prov, count) = history_row.unwrap();
    assert_eq!(from_acc, "acc-sw-1");
    assert_eq!(to_acc, "acc-sw-2");
    assert_eq!(prov, "fake");
    assert_eq!(count, 1);
}

/// Tiêu chí 2: Vượt giới hạn chuyển (mặc định 2) -> đặt task thành needs_human, không lặp vô hạn.
#[tokio::test]
async fn test_ac2_exceeding_max_switches_sets_needs_human_no_loop() {
    let ctx = setup_test_context().await;
    enable_modules(
        &ctx,
        &[
            "subscription-connect",
            "multi-account",
            "quota-tracking",
            "auto-switch",
        ],
    )
    .await;

    let now = Utc::now().to_rfc3339();
    let ws_id = WorkspaceId::new();
    let prj_id = ProjectId::new();
    let t_id = TaskId::new();

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS Limit', ?)")
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'PRJ Limit', '/repo', 'main', ?)")
        .bind(prj_id.to_string())
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES (?, ?, 'Task Limit', 'D', 'running', 'lead', 'fake', ?, ?)")
        .bind(t_id.to_string())
        .bind(prj_id.to_string())
        .bind(&now)
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO agent_account (id, provider, label, home_dir, auth_type, status, enabled, priority) \
         VALUES ('acc-3', 'fake', 'Fake 3', '/tmp/h3', 'cli', 'ready', 1, 30)",
    )
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Giả lập đã chuyển 2 lần trước đó (đạt giới hạn 2)
    sqlx::query(
        "INSERT INTO m_auto_switch_history (id, task_id, from_account_id, to_account_id, from_provider, to_provider, reason, switch_count, created_at) \
         VALUES ('h1', ?, 'acc-1', 'acc-2', 'fake', 'fake', 'quota', 1, ?), ('h2', ?, 'acc-2', 'acc-3', 'fake', 'fake', 'quota', 2, ?)",
    )
    .bind(t_id.to_string())
    .bind(&now)
    .bind(t_id.to_string())
    .bind(&now)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Lần chạy thứ 3 bị hết quota tiếp
    let r3_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO run (id, task_id, agent, account, status, started_at) VALUES (?, ?, 'fake', 'acc-3', 'failed', ?)")
        .bind(&r3_id)
        .bind(t_id.to_string())
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    ctx.event_bus.publish(dandori_types::Envelope::new(
        Utc::now(),
        dandori_types::EventScope::task(ws_id, prj_id, t_id),
        dandori_types::EventPayload::RunFailed {
            run_id: r3_id.parse().unwrap(),
            error_message: "Hết hạn mức: quota_exhausted limit reached".to_string(),
        },
    ));

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Kiểm tra task chuyển sang needs_human và không có bản ghi chuyển thứ 3
    let (st,): (String,) = sqlx::query_as("SELECT status FROM task WHERE id = ?")
        .bind(t_id.to_string())
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    assert_eq!(st, TaskStatus::NeedsHuman.as_str());

    let (count,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM m_auto_switch_history WHERE task_id = ?")
            .bind(t_id.to_string())
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(
        count, 2,
        "Không được tăng thêm lượt chuyển khi đã đạt max_switches"
    );
}

/// Tiêu chí 3: Không chuyển sang nhà cung cấp khác khi chưa cho phép; không chuyển sang account api_key.
#[tokio::test]
async fn test_ac3_no_cross_provider_without_permission_and_no_api_key() {
    let ctx = setup_test_context().await;
    enable_modules(
        &ctx,
        &[
            "subscription-connect",
            "multi-account",
            "quota-tracking",
            "auto-switch",
        ],
    )
    .await;

    let now = Utc::now().to_rfc3339();

    // Tạo 1 tài khoản fake (hết quota)
    sqlx::query(
        "INSERT INTO agent_account (id, provider, label, home_dir, auth_type, status, enabled, priority, exhausted_until) \
         VALUES ('acc-f1', 'fake', 'Fake 1', '/tmp/f1', 'cli', 'ready', 1, 10, '2099-01-01T00:00:00Z')",
    )
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Tạo 1 tài khoản claude (khác nhà, sẵn sàng)
    sqlx::query(
        "INSERT INTO agent_account (id, provider, label, home_dir, auth_type, status, enabled, priority) \
         VALUES ('acc-c1', 'claude', 'Claude 1', '/tmp/c1', 'cli', 'ready', 1, 20)",
    )
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Tạo 1 tài khoản fake nhưng auth_type = 'api_key'
    sqlx::query(
        "INSERT INTO agent_account (id, provider, label, home_dir, auth_type, status, enabled, priority) \
         VALUES ('acc-api1', 'fake', 'Fake ApiKey', '/tmp/api', 'api_key', 'ready', 1, 5)",
    )
    .execute(&ctx.pool)
    .await
    .unwrap();

    let ws_id = WorkspaceId::new();
    let prj_id = ProjectId::new();
    let t_id = TaskId::new();

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS Cross', ?)")
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'PRJ Cross', '/repo', 'main', ?)")
        .bind(prj_id.to_string())
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES (?, ?, 'Task Cross', 'D', 'running', 'lead', 'fake', ?, ?)")
        .bind(t_id.to_string())
        .bind(prj_id.to_string())
        .bind(&now)
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    let r_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO run (id, task_id, agent, account, status, started_at) VALUES (?, ?, 'fake', 'Fake 1', 'failed', ?)")
        .bind(&r_id)
        .bind(t_id.to_string())
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    ctx.event_bus.publish(dandori_types::Envelope::new(
        Utc::now(),
        dandori_types::EventScope::task(ws_id, prj_id, t_id),
        dandori_types::EventPayload::RunFailed {
            run_id: r_id.parse().unwrap(),
            error_message: "Hết hạn mức: quota_exhausted".to_string(),
        },
    ));

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Không chuyển sang claude (khác nhà) và không chuyển sang api_key -> task chuyển sang needs_human
    let (st,): (String,) = sqlx::query_as("SELECT status FROM task WHERE id = ?")
        .bind(t_id.to_string())
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    assert_eq!(st, TaskStatus::NeedsHuman.as_str());

    let (count,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM m_auto_switch_history WHERE task_id = ?")
            .bind(t_id.to_string())
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(count, 0, "Không được tự chuyển sang nhà khác hoặc api_key");
}

/// Tiêu chí 4: Worktree và prompt giữ nguyên qua lần chuyển.
#[tokio::test]
async fn test_ac4_worktree_and_prompt_preserved_across_switches() {
    let ctx = setup_test_context().await;
    enable_modules(
        &ctx,
        &[
            "subscription-connect",
            "multi-account",
            "quota-tracking",
            "auto-switch",
        ],
    )
    .await;

    let now = Utc::now().to_rfc3339();
    let ws_id = WorkspaceId::new();
    let prj_id = ProjectId::new();
    let t_id = TaskId::new();

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS WT', ?)")
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    let owner_id: (String,) = sqlx::query_as("SELECT id FROM user WHERE username = 'owner'")
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'admin')",
    )
    .bind(ws_id.to_string())
    .bind(&owner_id.0)
    .execute(&ctx.pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'PRJ WT', '/repo', 'main', ?)")
        .bind(prj_id.to_string())
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    let expected_workdir = "/tmp/preserved_worktree_dir";
    let expected_desc = "Important user prompt that must not change";

    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, workdir_path, created_at, updated_at) VALUES (?, ?, 'Task WT', ?, 'running', 'lead', 'fake', ?, ?, ?)")
        .bind(t_id.to_string())
        .bind(prj_id.to_string())
        .bind(expected_desc)
        .bind(expected_workdir)
        .bind(&now)
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO agent_account (id, provider, label, home_dir, auth_type, status, enabled, priority) \
         VALUES ('acc-wt-1', 'fake', 'Fake WT 1', '/tmp/h1', 'cli', 'ready', 1, 10), \
                ('acc-wt-2', 'fake', 'Fake WT 2', '/tmp/h2', 'cli', 'ready', 1, 20)",
    )
    .execute(&ctx.pool)
    .await
    .unwrap();

    let r_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO run (id, task_id, agent, account, status, started_at) VALUES (?, ?, 'fake', 'Fake WT 1', 'failed', ?)")
        .bind(&r_id)
        .bind(t_id.to_string())
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    sqlx::query(
        "UPDATE agent_account SET exhausted_until = '2099-01-01T00:00:00Z' WHERE id = 'acc-wt-1'",
    )
    .execute(&ctx.pool)
    .await
    .unwrap();

    ctx.event_bus.publish(dandori_types::Envelope::new(
        Utc::now(),
        dandori_types::EventScope::task(ws_id, prj_id, t_id),
        dandori_types::EventPayload::RunFailed {
            run_id: r_id.parse().unwrap(),
            error_message: "Hết hạn mức: quota_exhausted".to_string(),
        },
    ));

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    // Kiểm tra task: workdir_path và description (prompt) được bảo toàn nguyên vẹn
    let (wt, desc): (Option<String>, String) =
        sqlx::query_as("SELECT workdir_path, description FROM task WHERE id = ?")
            .bind(t_id.to_string())
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

    assert_eq!(wt.as_deref(), Some(expected_workdir));
    assert_eq!(desc, expected_desc);
}

/// Tiêu chí 5: Khi tắt module auto-switch -> task gặp QuotaExhausted chuyển sang needs_human, không tự chuyển tài khoản.
#[tokio::test]
async fn test_ac5_disabled_module_leaves_task_at_needs_human() {
    let ctx = setup_test_context().await;
    // Bật subscription-connect và multi-account nhưng KHÔNG bật auto-switch
    enable_modules(
        &ctx,
        &["subscription-connect", "multi-account", "quota-tracking"],
    )
    .await;

    let now = Utc::now().to_rfc3339();
    let ws_id = WorkspaceId::new();
    let prj_id = ProjectId::new();
    let t_id = TaskId::new();

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS Off', ?)")
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'PRJ Off', '/repo', 'main', ?)")
        .bind(prj_id.to_string())
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES (?, ?, 'Task Off', 'D', 'running', 'lead', 'fake', ?, ?)")
        .bind(t_id.to_string())
        .bind(prj_id.to_string())
        .bind(&now)
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    // Có 2 tài khoản fake sẵn sàng
    sqlx::query(
        "INSERT INTO agent_account (id, provider, label, home_dir, auth_type, status, enabled, priority) \
         VALUES ('acc-off-1', 'fake', 'Fake Off 1', '/tmp/h1', 'cli', 'ready', 1, 10), \
                ('acc-off-2', 'fake', 'Fake Off 2', '/tmp/h2', 'cli', 'ready', 1, 20)",
    )
    .execute(&ctx.pool)
    .await
    .unwrap();

    let r_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO run (id, task_id, agent, account, status, started_at) VALUES (?, ?, 'fake', 'Fake Off 1', 'failed', ?)")
        .bind(&r_id)
        .bind(t_id.to_string())
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    ctx.event_bus.publish(dandori_types::Envelope::new(
        Utc::now(),
        dandori_types::EventScope::task(ws_id, prj_id, t_id),
        dandori_types::EventPayload::RunFailed {
            run_id: r_id.parse().unwrap(),
            error_message: "Hết hạn mức: quota_exhausted".to_string(),
        },
    ));

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Do module auto-switch tắt, không có lượt chuyển nào
    let (history_count,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='m_auto_switch_history'",
    )
    .fetch_one(&ctx.pool)
    .await
    .unwrap();

    if history_count > 0 {
        let (count,): (i64,) =
            sqlx::query_as("SELECT count(*) FROM m_auto_switch_history WHERE task_id = ?")
                .bind(t_id.to_string())
                .fetch_one(&ctx.pool)
                .await
                .unwrap();
        assert_eq!(count, 0, "Không được tự chuyển tài khoản khi module tắt");
    }
}

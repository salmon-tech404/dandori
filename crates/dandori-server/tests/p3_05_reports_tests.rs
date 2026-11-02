//! Các bài kiểm thử tích hợp cho Task P3-05: Module `reports`.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P3-05 dùng unwrap và kiểm tra nhiều tiêu chí nghiệm thu"
)]

use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use dandori_events::EventBus;
use dandori_module_api::TaskRunner;
use dandori_registry::FeatureRegistry;
use dandori_registry::lifecycle::FeatureLifecycleManager;
use dandori_server::app::create_app;
use dandori_server::config::ServerConfig;
use dandori_server::run_service::RunService;
use dandori_server::state::AppState;
use dandori_types::{ProjectId, WorkspaceId};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tempfile::tempdir;
use tower::ServiceExt;

struct TestContext {
    app: axum::Router,
    pool: SqlitePool,
    #[expect(dead_code, reason = "Giữ lifecycle_manager trong bộ nhớ")]
    lifecycle_manager: Arc<FeatureLifecycleManager>,
    #[expect(dead_code, reason = "Giữ event_bus trong bộ nhớ")]
    event_bus: Arc<EventBus>,
    owner_cookie: String,
    member_cookie: String,
    outsider_cookie: String,
    workspace_id: String,
    project_id: String,
    #[expect(dead_code, reason = "Giữ run_service cho bài test")]
    run_service: Arc<RunService>,
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
    let run_service = Arc::new(RunService::new(
        pool.clone(),
        Arc::clone(&event_bus),
        Arc::clone(&lifecycle_manager),
        10,
    ));
    let fake_adapter = Arc::new(dandori_agent_adapters::FakeAdapter::new(
        std::path::PathBuf::from("fake-agent"),
    ));
    run_service
        .register_adapter("fake", fake_adapter.clone())
        .await;

    let runner: Arc<dyn TaskRunner> = run_service.clone();
    lifecycle_manager.set_task_runner(runner);

    let state = AppState::new_with_run_service(
        pool.clone(),
        Arc::clone(&event_bus),
        Arc::clone(&lifecycle_manager),
        config,
        run_service.clone(),
    );
    let app = create_app(state);

    // 1. Setup Owner (Admin)
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
        .split(';')
        .next()
        .unwrap()
        .to_string();

    // 2. Tạo Workspace và Project
    let ws_id = WorkspaceId::new();
    let prj_id = ProjectId::new();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS Reports', ?)")
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let owner_user_id: String = sqlx::query_scalar("SELECT id FROM user WHERE username = 'owner'")
        .fetch_one(&pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'admin')",
    )
    .bind(ws_id.to_string())
    .bind(&owner_user_id)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'Prj Reports', '/tmp/repo', 'main', ?)"
    )
    .bind(prj_id.to_string())
    .bind(ws_id.to_string())
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    // 3. Setup Member (role = 'member')
    let member_user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, 'member', 'hash', 'user', ?)")
        .bind(&member_user_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'member')",
    )
    .bind(ws_id.to_string())
    .bind(&member_user_id)
    .execute(&pool)
    .await
    .unwrap();
    let member_token = "member_token_reports";
    let member_token_hash = dandori_server::auth::hash_token(member_token);
    let expires = (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339();
    sqlx::query(
        "INSERT INTO session (id, user_id, expires_at, token_hash) VALUES ('sess_mem_rep', ?, ?, ?)",
    )
    .bind(&member_user_id)
    .bind(&expires)
    .bind(&member_token_hash)
    .execute(&pool)
    .await
    .unwrap();
    let member_cookie = format!("dandori_session={member_token}");

    // 4. Setup Outsider (not in workspace)
    let outsider_user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, 'outsider', 'hash', 'user', ?)")
        .bind(&outsider_user_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();
    let outsider_token = "outsider_token_reports";
    let outsider_token_hash = dandori_server::auth::hash_token(outsider_token);
    sqlx::query("INSERT INTO session (id, user_id, expires_at, token_hash) VALUES ('sess_out_rep', ?, ?, ?)",)
        .bind(&outsider_user_id)
        .bind(&expires)
        .bind(&outsider_token_hash)
        .execute(&pool)
        .await
        .unwrap();
    let outsider_cookie = format!("dandori_session={outsider_token}");

    TestContext {
        app,
        pool,
        lifecycle_manager,
        event_bus,
        owner_cookie,
        member_cookie,
        outsider_cookie,
        workspace_id: ws_id.to_string(),
        project_id: prj_id.to_string(),
        run_service,
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
                    "enable": true,
                    "risk_acknowledged": true
                })
                .to_string(),
            ))
            .unwrap();
        let resp = ctx.app.clone().oneshot(enable_req).await.unwrap();
        assert!(
            resp.status().is_success(),
            "Không thể kích hoạt module {mod_id}"
        );
    }
}

#[tokio::test]
async fn test_ac1_and_ac5_current_report_and_bola() {
    let ctx = setup_test_context().await;
    enable_modules(&ctx, &["run-history", "reports"]).await;

    // Tạo 1 task đã hoàn thành và 1 run
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES ('t-1', ?, 'Task 1', 'desc', 'done', 'lead', 'worker', ?, ?)"
    )
    .bind(&ctx.project_id)
    .bind(&now)
    .bind(&now)
    .execute(&ctx.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) VALUES ('r-1', 't-1', 'claude', 'acc-1', 'success', ?, ?, 0)"
    )
    .bind(&now)
    .bind(&now)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // 1. Outsider truy vấn báo cáo -> 403 Forbidden (BOLA)
    let outsider_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/reports/workspaces/{}/reports/current?period_type=daily",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.outsider_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(outsider_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 2. Member truy vấn báo cáo -> 200 OK
    let member_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/reports/workspaces/{}/reports/current?period_type=daily",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(member_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(body.get("report").is_some());
    assert!(body.get("markdown").is_some());
    assert_eq!(body["report"]["counts"]["completed"], 1);
    assert_eq!(body["report"]["counts"]["total"], 1);
}

#[tokio::test]
async fn test_ac3_and_ac6_config_authorization_and_snapshots() {
    let ctx = setup_test_context().await;
    enable_modules(&ctx, &["run-history", "reports"]).await;

    // 1. Member cố cập nhật cấu hình -> 403 Forbidden (yêu cầu Admin/Owner)
    let member_update = Request::builder()
        .method("PUT")
        .uri(format!(
            "/api/v1/m/reports/workspaces/{}/reports/config",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "auto_generate": true,
                "timezone": "+07:00",
                "daily_time": "22:00"
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(member_update).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 2. Owner cập nhật cấu hình hợp lệ -> 200 OK
    let owner_update = Request::builder()
        .method("PUT")
        .uri(format!(
            "/api/v1/m/reports/workspaces/{}/reports/config",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "auto_generate": true,
                "timezone": "+07:00",
                "daily_time": "22:00"
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(owner_update).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. Member truy vấn snapshots -> 200 OK
    let get_snapshots = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/reports/workspaces/{}/reports/snapshots",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(get_snapshots).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 4. Outsider truy vấn snapshots -> 403 Forbidden
    let outsider_snapshots = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/reports/workspaces/{}/reports/snapshots",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.outsider_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(outsider_snapshots).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

//! Các bài kiểm thử tích hợp cho Task P3-04: Module `overnight-queue`.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P3-04 dùng unwrap và kiểm tra nhiều tiêu chí nghiệm thu"
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

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS Overnight', ?)")
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
        "INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'Prj Overnight', '/tmp/repo', 'main', ?)"
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
    let member_token = "member_token_overnight";
    let member_token_hash = dandori_server::auth::hash_token(member_token);
    let expires = (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339();
    sqlx::query(
        "INSERT INTO session (id, user_id, expires_at, token_hash) VALUES ('sess_mem_on', ?, ?, ?)",
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
    let outsider_token = "outsider_token_overnight";
    let outsider_token_hash = dandori_server::auth::hash_token(outsider_token);
    sqlx::query(
        "INSERT INTO session (id, user_id, expires_at, token_hash) VALUES ('sess_out_on', ?, ?, ?)",
    )
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
async fn test_ac1_queue_enqueue_reorder_and_bola() {
    let ctx = setup_test_context().await;
    enable_modules(&ctx, &["calendar", "quiet-hours", "overnight-queue"]).await;

    // Tạo 2 task trong project
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES ('t-1', ?, 'Task 1', 'desc 1', 'backlog', 'lead', 'worker', ?, ?)"
    )
    .bind(&ctx.project_id)
    .bind(&now)
    .bind(&now)
    .execute(&ctx.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES ('t-2', ?, 'Task 2', 'desc 2', 'backlog', 'lead', 'worker', ?, ?)"
    )
    .bind(&ctx.project_id)
    .bind(&now)
    .bind(&now)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // 1. Outsider thử enqueue task -> 403 Forbidden (BOLA)
    let outsider_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/overnight-queue/workspaces/{}/queue/items",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.outsider_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "task_ids": ["t-1", "t-2"] }).to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(outsider_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 2. Member enqueue tasks hợp lệ -> 200 OK
    let member_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/overnight-queue/workspaces/{}/queue/items",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "task_ids": ["t-1", "t-2"] }).to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(member_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. Member lấy chi tiết queue
    let get_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/overnight-queue/workspaces/{}/queue",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["task_id"], "t-1");
    assert_eq!(items[1]["task_id"], "t-2");

    let item1_id = items[0]["id"].as_str().unwrap().to_string();
    let item2_id = items[1]["id"].as_str().unwrap().to_string();

    // 4. Member đảo thứ tự: item2 lên trước item1
    let reorder_req = Request::builder()
        .method("PUT")
        .uri(format!(
            "/api/v1/m/overnight-queue/workspaces/{}/queue/reorder",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "ordered_item_ids": [item2_id, item1_id] }).to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(reorder_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Kiểm tra thứ tự sau khi đảo
    let get_req2 = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/overnight-queue/workspaces/{}/queue",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp2 = ctx.app.clone().oneshot(get_req2).await.unwrap();
    let bytes2 = to_bytes(resp2.into_body(), usize::MAX).await.unwrap();
    let body2: Value = serde_json::from_slice(&bytes2).unwrap();
    let reordered_items = body2["items"].as_array().unwrap();
    assert_eq!(reordered_items[0]["task_id"], "t-2");
    assert_eq!(reordered_items[1]["task_id"], "t-1");
}

#[tokio::test]
async fn test_ac2_safe_stop_and_pause_resume() {
    let ctx = setup_test_context().await;
    enable_modules(&ctx, &["calendar", "quiet-hours", "overnight-queue"]).await;

    // 1. Member thường cố cập nhật cấu hình -> 403 Forbidden (yêu cầu Admin/Owner)
    let member_update_cfg = Request::builder()
        .method("PUT")
        .uri(format!(
            "/api/v1/m/overnight-queue/workspaces/{}/queue",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "start_time": "22:00",
                "max_concurrency": 3,
                "consecutive_failure_limit": 2
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(member_update_cfg).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 2. Owner cập nhật cấu hình hợp lệ -> 200 OK
    let owner_update_cfg = Request::builder()
        .method("PUT")
        .uri(format!(
            "/api/v1/m/overnight-queue/workspaces/{}/queue",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "start_time": "22:00",
                "max_concurrency": 3,
                "consecutive_failure_limit": 2
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(owner_update_cfg).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. Member thử pause queue -> 403 Forbidden
    let member_pause = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/overnight-queue/workspaces/{}/queue/pause",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(member_pause).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 4. Owner pause queue -> 200 OK
    let pause_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/overnight-queue/workspaces/{}/queue/pause",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(pause_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 5. Owner resume queue -> 200 OK
    let resume_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/overnight-queue/workspaces/{}/queue/resume",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(resume_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 6. Owner start queue now -> 200 OK
    let start_now_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/overnight-queue/workspaces/{}/queue/start-now",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(start_now_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_ac3_morning_summary() {
    let ctx = setup_test_context().await;
    enable_modules(&ctx, &["calendar", "quiet-hours", "overnight-queue"]).await;

    // 1. Outsider truy vấn summary -> 403 Forbidden
    let out_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/overnight-queue/workspaces/{}/queue/summary",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.outsider_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(out_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 2. Member truy vấn summary -> 200 OK
    let mem_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/overnight-queue/workspaces/{}/queue/summary",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(mem_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(body.get("summary").is_some());
}

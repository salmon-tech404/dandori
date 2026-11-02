//! Các bài kiểm thử tích hợp cho Task P2-12: Module `cost-tracking`.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P2-12 dùng unwrap và kiểm tra nhiều tiêu chí"
)]

use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use dandori_db::repo::{ProjectRepo, TaskRepo};
use dandori_events::{EventBus, EventFilter};
use dandori_module_api::TaskRunner;
use dandori_registry::FeatureRegistry;
use dandori_registry::lifecycle::FeatureLifecycleManager;
use dandori_server::app::create_app;
use dandori_server::config::ServerConfig;
use dandori_server::run_service::RunService;
use dandori_server::state::AppState;
use dandori_types::{Project, ProjectId, Task, TaskId, WorkspaceId};
use serde_json::{Value, json};
use sqlx::{Row, SqlitePool};
use tempfile::tempdir;
use tower::ServiceExt;

struct TestContext {
    app: axum::Router,
    pool: SqlitePool,
    #[expect(dead_code, reason = "Giữ lifecycle manager trong bộ nhớ")]
    lifecycle_manager: Arc<FeatureLifecycleManager>,
    event_bus: Arc<EventBus>,
    owner_cookie: String,
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
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "Bật module {mod_id} phải thành công"
        );
    }
}

async fn create_test_project_and_task(ctx: &TestContext) -> (String, String, String) {
    let ws_id = WorkspaceId::new();
    let ws_str = ws_id.to_string();
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    let owner_user = sqlx::query("SELECT id FROM user WHERE username = 'owner'")
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    let user_id_str: String = owner_user.get("id");

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS 1', ?)")
        .bind(&ws_str)
        .bind(&now_str)
        .execute(&ctx.pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'admin')",
    )
    .bind(&ws_str)
    .bind(&user_id_str)
    .execute(&ctx.pool)
    .await
    .unwrap();

    let prj_id = ProjectId::new();
    let project = Project {
        id: prj_id,
        workspace_id: ws_id,
        name: "Test Project Cost Tracking".to_string(),
        repo_path: ".".to_string(),
        default_branch: "main".to_string(),
        created_at: now,
    };
    ProjectRepo::new(&ctx.pool)
        .create_project(&project)
        .await
        .unwrap();

    let task_id = TaskId::new();
    let task = Task {
        id: task_id,
        project_id: prj_id,
        title: "Test Task Cost Tracking".to_string(),
        description: "Test task".to_string(),
        status: "todo".to_string(),
        lead_agent: "lead".to_string(),
        worker_agent: "worker".to_string(),
        workdir_path: None,
        created_at: now,
        updated_at: now,
    };
    TaskRepo::new(&ctx.pool).create_task(&task).await.unwrap();

    (ws_str, prj_id.to_string(), task_id.to_string())
}

#[tokio::test]
async fn test_ac1_token_accumulation_and_missing_runs() {
    let ctx = setup_test_context().await;
    enable_modules(&ctx, &["live-status", "cost-tracking"]).await;

    let (_ws_id, prj_id, task_id) = create_test_project_and_task(&ctx).await;

    // Run 1: có số liệu (1000 input, 500 output)
    let run1_id = "run-001";
    let req1 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/cost-tracking/runs/{run1_id}/record-tokens"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "task_id": task_id,
                "project_id": prj_id,
                "agent_id": "agent-1",
                "account_id": "acc-1",
                "provider": "anthropic",
                "model": "claude-3-5-sonnet",
                "has_token_data": true,
                "input_tokens": 1000,
                "output_tokens": 500
            })
            .to_string(),
        ))
        .unwrap();
    let resp1 = ctx.app.clone().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);

    // Run 2: có số liệu (2000 input, 1000 output)
    let run2_id = "run-002";
    let req2 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/cost-tracking/runs/{run2_id}/record-tokens"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "task_id": task_id,
                "project_id": prj_id,
                "agent_id": "agent-1",
                "account_id": "acc-1",
                "provider": "anthropic",
                "model": "claude-3-5-sonnet",
                "has_token_data": true,
                "input_tokens": 2000,
                "output_tokens": 1000
            })
            .to_string(),
        ))
        .unwrap();
    let resp2 = ctx.app.clone().oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);

    // Run 3: KHÔNG có số liệu (CLI không báo)
    let run3_id = "run-003";
    let req3 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/cost-tracking/runs/{run3_id}/record-tokens"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "task_id": task_id,
                "project_id": prj_id,
                "agent_id": "agent-2",
                "account_id": "acc-2",
                "provider": "openai",
                "model": "o3-mini",
                "has_token_data": false,
                "input_tokens": null,
                "output_tokens": null
            })
            .to_string(),
        ))
        .unwrap();
    let resp3 = ctx.app.clone().oneshot(req3).await.unwrap();
    assert_eq!(resp3.status(), StatusCode::OK);

    // Truy vấn chi phí task
    let cost_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/cost-tracking/tasks/{task_id}/cost"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let cost_resp = ctx.app.clone().oneshot(cost_req).await.unwrap();
    assert_eq!(cost_resp.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(cost_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let summary: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(summary["task_id"], task_id);
    assert_eq!(summary["has_token_data"], true);
    assert_eq!(summary["total_input_tokens"], 3000);
    assert_eq!(summary["total_output_tokens"], 1500);
    assert_eq!(summary["total_tokens"], 4500);
    assert_eq!(summary["missing_runs_count"], 1);
    assert!(
        summary["total_cost"].is_null(),
        "Chưa nhập bảng giá thì tổng chi phí phải là null"
    );
}

#[tokio::test]
async fn test_ac2_pricing_table_and_total_cost() {
    let ctx = setup_test_context().await;
    enable_modules(&ctx, &["live-status", "cost-tracking"]).await;

    let (_ws_id, prj_id, task_id) = create_test_project_and_task(&ctx).await;

    // Ghi 1 lần chạy 1000 input tokens và 500 output tokens
    let run1_id = "run-pricing-01";
    let req1 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/cost-tracking/runs/{run1_id}/record-tokens"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "task_id": task_id,
                "project_id": prj_id,
                "agent_id": "agent-1",
                "account_id": "acc-1",
                "provider": "anthropic",
                "model": "claude-3-5-sonnet",
                "has_token_data": true,
                "input_tokens": 1000,
                "output_tokens": 500
            })
            .to_string(),
        ))
        .unwrap();
    let resp1 = ctx.app.clone().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);

    // Trước khi có bảng giá: total_cost là null
    let cost_req1 = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/cost-tracking/tasks/{task_id}/cost"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let cost_resp1 = ctx.app.clone().oneshot(cost_req1).await.unwrap();
    let body_bytes1 = axum::body::to_bytes(cost_resp1.into_body(), usize::MAX)
        .await
        .unwrap();
    let summary1: Value = serde_json::from_slice(&body_bytes1).unwrap();
    assert!(summary1["total_cost"].is_null());

    // Thiết lập bảng giá cho anthropic claude-3-5-sonnet: input: $0.000003, output: $0.000015
    let pricing_req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/m/cost-tracking/projects/{prj_id}/pricing"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "anthropic",
                "model": "claude-3-5-sonnet",
                "input_token_price": 3.0,
                "output_token_price": 15.0
            })
            .to_string(),
        ))
        .unwrap();
    let pricing_resp = ctx.app.clone().oneshot(pricing_req).await.unwrap();
    assert_eq!(pricing_resp.status(), StatusCode::OK);

    // Sau khi có bảng giá: total_cost phải = 1000*0.000003 + 500*0.000015 = 0.003 + 0.0075 = 0.0105
    let cost_req2 = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/cost-tracking/tasks/{task_id}/cost"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let cost_resp2 = ctx.app.clone().oneshot(cost_req2).await.unwrap();
    let body_bytes2 = axum::body::to_bytes(cost_resp2.into_body(), usize::MAX)
        .await
        .unwrap();
    let summary2: Value = serde_json::from_slice(&body_bytes2).unwrap();

    let cost = summary2["total_cost"]
        .as_f64()
        .expect("total_cost phải là số thực");
    assert!(
        (cost - 0.0105).abs() < 1e-6,
        "Cost tính toán phải đúng: expected 0.0105, got {cost}"
    );
}

#[tokio::test]
async fn test_ac3_task_budget_crossing_single_notification() {
    let ctx = setup_test_context().await;
    enable_modules(&ctx, &["live-status", "cost-tracking"]).await;

    let (_ws_id, prj_id, task_id) = create_test_project_and_task(&ctx).await;

    // Đặt bảng giá
    let pricing_req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/m/cost-tracking/projects/{prj_id}/pricing"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "openai",
                "model": "gpt-4o",
                "input_token_price": 5.0,
                "output_token_price": 15.0
            })
            .to_string(),
        ))
        .unwrap();
    let pricing_resp = ctx.app.clone().oneshot(pricing_req).await.unwrap();
    assert_eq!(pricing_resp.status(), StatusCode::OK);

    // Đặt ngân sách cho task là $0.05
    let budget_req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/m/cost-tracking/tasks/{task_id}/budget"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "max_budget": 0.05
            })
            .to_string(),
        ))
        .unwrap();
    let budget_resp = ctx.app.clone().oneshot(budget_req).await.unwrap();
    assert_eq!(budget_resp.status(), StatusCode::OK);

    // Đăng ký nhận sự kiện từ event_bus
    let mut rx = ctx.event_bus.subscribe(EventFilter::all());

    // Run 1: Chi phí = 1000 * 0.000005 + 1000 * 0.000015 = 0.02 (< 0.05) -> Chưa vượt ngân sách
    let run1_id = "run-budget-01";
    let req1 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/cost-tracking/runs/{run1_id}/record-tokens"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "task_id": task_id,
                "project_id": prj_id,
                "agent_id": "agent-1",
                "account_id": "acc-1",
                "provider": "openai",
                "model": "gpt-4o",
                "has_token_data": true,
                "input_tokens": 1000,
                "output_tokens": 1000
            })
            .to_string(),
        ))
        .unwrap();
    let resp1 = ctx.app.clone().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);

    // Kiểm tra không có notification
    let maybe_ev = tokio::time::timeout(Duration::from_millis(50), rx.recv()).await;
    assert!(
        maybe_ev.is_err(),
        "Chưa vượt ngân sách không được phát thông báo"
    );

    // Run 2: Chi phí thêm = 2000 * 0.000005 + 2500 * 0.000015 = 0.01 + 0.0375 = 0.0475
    // Tổng chi phí = 0.02 + 0.0475 = 0.0675 > 0.05 -> Vượt ngân sách! Phát 1 thông báo
    let run2_id = "run-budget-02";
    let req2 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/cost-tracking/runs/{run2_id}/record-tokens"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "task_id": task_id,
                "project_id": prj_id,
                "agent_id": "agent-1",
                "account_id": "acc-1",
                "provider": "openai",
                "model": "gpt-4o",
                "has_token_data": true,
                "input_tokens": 2000,
                "output_tokens": 2500
            })
            .to_string(),
        ))
        .unwrap();
    let resp2 = ctx.app.clone().oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);

    let event = tokio::time::timeout(Duration::from_millis(200), rx.recv())
        .await
        .expect("Phải nhận được thông báo vượt ngân sách")
        .expect("Sự kiện hợp lệ");
    assert_eq!(
        event.payload.kind(),
        dandori_types::EventKind::NotificationRequested,
        "Sự kiện phải là NotificationRequested"
    );

    // Run 3: Vượt ngân sách tiếp -> Không được phát thêm thông báo (chỉ phát 1 lần cho lần vượt)
    let run3_id = "run-budget-03";
    let req3 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/cost-tracking/runs/{run3_id}/record-tokens"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "task_id": task_id,
                "project_id": prj_id,
                "agent_id": "agent-1",
                "account_id": "acc-1",
                "provider": "openai",
                "model": "gpt-4o",
                "has_token_data": true,
                "input_tokens": 1000,
                "output_tokens": 1000
            })
            .to_string(),
        ))
        .unwrap();
    let resp3 = ctx.app.clone().oneshot(req3).await.unwrap();
    assert_eq!(resp3.status(), StatusCode::OK);

    let maybe_ev2 = tokio::time::timeout(Duration::from_millis(50), rx.recv()).await;
    assert!(
        maybe_ev2.is_err(),
        "Không được phát lặp lại thông báo vượt ngân sách"
    );
}

#[tokio::test]
async fn test_ac4_bola_workspace_isolation() {
    let ctx = setup_test_context().await;
    enable_modules(&ctx, &["live-status", "cost-tracking"]).await;

    let (_ws_id, prj_id, task_id) = create_test_project_and_task(&ctx).await;

    // Tạo user2 "stranger" không thuộc workspace
    let stranger_id = dandori_types::UserId::new();
    let stranger_str = stranger_id.to_string();
    let now_str = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, 'stranger', 'hash', 'user', ?)"
    )
    .bind(&stranger_str).bind(&now_str)
    .execute(&ctx.pool).await.unwrap();

    let token = "stranger_session_token";
    let token_hash = dandori_db::hash_session_token(token);
    let expires_at = (Utc::now() + chrono::Duration::hours(24)).to_rfc3339();
    sqlx::query("INSERT INTO session (id, user_id, expires_at, token_hash) VALUES (?, ?, ?, ?)")
        .bind(dandori_types::SessionId::new().to_string())
        .bind(&stranger_str)
        .bind(expires_at)
        .bind(token_hash)
        .execute(&ctx.pool)
        .await
        .unwrap();

    let stranger_cookie = format!("dandori_session={token}");

    // Stranger cố truy vấn bảng giá của project -> 403 Forbidden
    let pricing_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/cost-tracking/projects/{prj_id}/pricing"))
        .header(header::COOKIE, &stranger_cookie)
        .body(Body::empty())
        .unwrap();
    let pricing_resp = ctx.app.clone().oneshot(pricing_req).await.unwrap();
    assert_eq!(pricing_resp.status(), StatusCode::FORBIDDEN);

    // Stranger cố truy vấn chi phí của task -> 403 Forbidden
    let cost_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/cost-tracking/tasks/{task_id}/cost"))
        .header(header::COOKIE, &stranger_cookie)
        .body(Body::empty())
        .unwrap();
    let cost_resp = ctx.app.clone().oneshot(cost_req).await.unwrap();
    assert_eq!(cost_resp.status(), StatusCode::FORBIDDEN);
}

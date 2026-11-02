//! Các bài kiểm thử tích hợp cho Task P3-07: Module `eta`.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P3-07 dùng unwrap và kiểm tra nhiều tiêu chí nghiệm thu"
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
    owner_cookie: String,
    member_cookie: String,
    outsider_cookie: String,
    workspace_id: String,
    project_id: String,
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

    let now = chrono::Utc::now().to_rfc3339();

    // 2. Setup Member & Outsider
    let member_user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, 'member', 'hash', 'user', ?)")
        .bind(&member_user_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let member_token = "member_token_eta";
    let member_token_hash = dandori_server::auth::hash_token(member_token);
    let expires = (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339();
    sqlx::query(
        "INSERT INTO session (id, user_id, expires_at, token_hash) VALUES ('sess_mem_eta', ?, ?, ?)",
    )
    .bind(&member_user_id)
    .bind(&expires)
    .bind(&member_token_hash)
    .execute(&pool)
    .await
    .unwrap();
    let member_cookie = format!("dandori_session={member_token}");

    let outsider_user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, 'outsider', 'hash', 'user', ?)")
        .bind(&outsider_user_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();
    let outsider_token = "outsider_token_eta";
    let outsider_token_hash = dandori_server::auth::hash_token(outsider_token);
    sqlx::query(
        "INSERT INTO session (id, user_id, expires_at, token_hash) VALUES ('sess_out_eta', ?, ?, ?)",
    )
    .bind(&outsider_user_id)
    .bind(&expires)
    .bind(&outsider_token_hash)
    .execute(&pool)
    .await
    .unwrap();
    let outsider_cookie = format!("dandori_session={outsider_token}");

    // 3. Tạo Workspace và Project
    let ws_id = WorkspaceId::new();
    let prj_id = ProjectId::new();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS ETA', ?)")
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let owner_user_id: String = sqlx::query_scalar("SELECT id FROM user WHERE username = 'owner'")
        .fetch_one(&pool)
        .await
        .unwrap();

    let member_user_id: String =
        sqlx::query_scalar("SELECT id FROM user WHERE username = 'member'")
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
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'member')",
    )
    .bind(ws_id.to_string())
    .bind(&member_user_id)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'Prj ETA', '/tmp/repo', 'main', ?)"
    )
    .bind(prj_id.to_string())
    .bind(ws_id.to_string())
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    // Kích hoạt module eta cho workspace
    let apply_req = Request::builder()
        .method("POST")
        .uri("/api/v1/features/apply")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, &owner_cookie)
        .body(Body::from(
            json!({
                "module_id": "eta",
                "scope": "workspace",
                "target_id": ws_id.to_string(),
                "enable": true
            })
            .to_string(),
        ))
        .unwrap();
    let apply_resp = app.clone().oneshot(apply_req).await.unwrap();
    assert_eq!(apply_resp.status(), StatusCode::OK);

    TestContext {
        app,
        pool,
        owner_cookie,
        member_cookie,
        outsider_cookie,
        workspace_id: ws_id.to_string(),
        project_id: prj_id.to_string(),
        temp_dir,
    }
}

#[tokio::test]
async fn test_ac1_and_ac5_manifest_and_authorization() {
    let ctx = setup_test_context().await;

    // AC1: Kiểm tra manifest trong registry
    let req = Request::builder()
        .uri("/api/v1/features/modules")
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();

    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let features = body["modules"].as_array().expect("Mảng module");

    let eta_mod = features
        .iter()
        .find(|f| f["id"] == "eta")
        .expect("Phải tìm thấy module eta trong registry");

    assert_eq!(eta_mod["name"], "Dự tính thời gian (ETA)");
    assert_eq!(eta_mod["risk"], "low");
    let requires = eta_mod["requires"].as_array().expect("Mảng requires");
    assert!(requires.iter().any(|r| r == "run-history"));

    // Tạo một task để test endpoint
    let task_id = "t-eta-auth";
    sqlx::query(
        "INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES (?, ?, 'Test Auth', 'Desc', 'todo', 'lead', 'claude', '2026-11-02T00:00:00Z', '2026-11-02T00:00:00Z')"
    )
    .bind(task_id)
    .bind(&ctx.project_id)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // AC5 BOLA: Người ngoài workspace bị từ chối 403 Forbidden
    let req_out = Request::builder()
        .uri(format!(
            "/api/v1/m/eta/workspaces/{}/eta/task/{}",
            ctx.workspace_id, task_id
        ))
        .header(header::COOKIE, &ctx.outsider_cookie)
        .body(Body::empty())
        .unwrap();
    let resp_out = ctx.app.clone().oneshot(req_out).await.unwrap();
    assert_eq!(resp_out.status(), StatusCode::FORBIDDEN);

    // Thành viên workspace truy cập thành công 200 OK
    let req_mem = Request::builder()
        .uri(format!(
            "/api/v1/m/eta/workspaces/{}/eta/task/{}",
            ctx.workspace_id, task_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp_mem = ctx.app.clone().oneshot(req_mem).await.unwrap();
    assert_eq!(resp_mem.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_ac2_and_ac3_under_threshold_and_estimation_accuracy() {
    let ctx = setup_test_context().await;

    let task_id = "t-eta-calc";
    sqlx::query(
        "INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES (?, ?, 'Task Calc', 'Desc', 'todo', 'lead', 'claude', '2026-11-02T00:00:00Z', '2026-11-02T00:00:00Z')"
    )
    .bind(task_id)
    .bind(&ctx.project_id)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // 1. Khi chưa có lần chạy nào hoàn tất -> chưa đủ dữ liệu (has_estimate = false)
    let req_empty = Request::builder()
        .uri(format!(
            "/api/v1/m/eta/workspaces/{}/eta/task/{}",
            ctx.workspace_id, task_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp_empty = ctx.app.clone().oneshot(req_empty).await.unwrap();
    assert_eq!(resp_empty.status(), StatusCode::OK);
    let bytes_empty = to_bytes(resp_empty.into_body(), usize::MAX).await.unwrap();
    let val_empty: Value = serde_json::from_slice(&bytes_empty).unwrap();
    assert_eq!(val_empty["eta"]["has_estimate"], false);
    assert_eq!(val_empty["eta"]["sample_count"], 0);

    // 2. Chèn 5 lần chạy hoàn tất cùng (prj, role, provider) với thời lượng: 60s, 120s, 180s, 240s, 300s
    for i in 1..=5 {
        let t_id = format!("t-seed-{i}");
        let r_id = format!("r-seed-{i}");
        let start = "2026-11-02T10:00:00Z";
        let end = format!("2026-11-02T10:0{}:00Z", i);

        sqlx::query(
            "INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES (?, ?, 'Seed', 'Desc', 'done', 'lead', 'claude', '2026-11-02T00:00:00Z', '2026-11-02T00:00:00Z')"
        )
        .bind(&t_id)
        .bind(&ctx.project_id)
        .execute(&ctx.pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) VALUES (?, ?, 'claude', 'acc-1', 'success', ?, ?, 0)"
        )
        .bind(&r_id)
        .bind(&t_id)
        .bind(start)
        .bind(&end)
        .execute(&ctx.pool)
        .await
        .unwrap();
    }

    // 3. Tạo một run đang chạy cho task hiện tại
    let active_run_id = "r-active-1";
    sqlx::query(
        "INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) VALUES (?, ?, 'claude', 'acc-1', 'running', '2026-11-02T11:00:00Z', NULL, NULL)"
    )
    .bind(active_run_id)
    .bind(task_id)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // 4. Giờ truy vấn task ETA -> đã có đủ 5 mẫu, has_estimate = true
    let req_est = Request::builder()
        .uri(format!(
            "/api/v1/m/eta/workspaces/{}/eta/task/{}",
            ctx.workspace_id, task_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp_est = ctx.app.clone().oneshot(req_est).await.unwrap();
    assert_eq!(resp_est.status(), StatusCode::OK);
    let bytes_est = to_bytes(resp_est.into_body(), usize::MAX).await.unwrap();
    let val_est: Value = serde_json::from_slice(&bytes_est).unwrap();

    assert_eq!(val_est["eta"]["has_estimate"], true);
    assert_eq!(val_est["eta"]["sample_count"], 5);
    assert_eq!(val_est["eta"]["median_secs"], 180.0);
    assert_eq!(val_est["eta"]["p80_secs"], 252.0);
    assert_eq!(val_est["eta"]["confidence"], "low");

    // 5. Ghi nhận hoàn tất tác vụ với thời lượng thực tế 190s -> kiểm tra đối soát sai số
    let rec_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/eta/workspaces/{}/eta/record-completion",
            ctx.workspace_id
        ))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::from(
            json!({
                "run_id": active_run_id,
                "actual_duration_secs": 190.0
            })
            .to_string(),
        ))
        .unwrap();
    let rec_resp = ctx.app.clone().oneshot(rec_req).await.unwrap();
    assert_eq!(rec_resp.status(), StatusCode::OK);

    // 6. Kiểm tra thống kê độ chính xác
    let acc_req = Request::builder()
        .uri(format!(
            "/api/v1/m/eta/workspaces/{}/eta/accuracy?project_id={}",
            ctx.workspace_id, ctx.project_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();
    let acc_resp = ctx.app.clone().oneshot(acc_req).await.unwrap();
    assert_eq!(acc_resp.status(), StatusCode::OK);
    let acc_bytes = to_bytes(acc_resp.into_body(), usize::MAX).await.unwrap();
    let acc_val: Value = serde_json::from_slice(&acc_bytes).unwrap();

    assert_eq!(acc_val["accuracy"]["total_predictions"], 1);
    assert_eq!(acc_val["accuracy"]["completed_predictions"], 1);
    // Sai số tuyệt đối |190.0 - 180.0| = 10.0
    assert_eq!(acc_val["accuracy"]["median_absolute_error_secs"], 10.0);
}

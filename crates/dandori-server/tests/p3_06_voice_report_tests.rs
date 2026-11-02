//! Các bài kiểm thử tích hợp cho Task P3-06: Module `voice-report`.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P3-06 dùng unwrap và kiểm tra nhiều tiêu chí nghiệm thu"
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
    owner_cookie: String,
    workspace_id: String,
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

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS Voice', ?)")
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
        "INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'Prj Voice', '/tmp/repo', 'main', ?)"
    )
    .bind(prj_id.to_string())
    .bind(ws_id.to_string())
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    TestContext {
        app,
        owner_cookie,
        workspace_id: ws_id.to_string(),
        temp_dir,
    }
}

#[tokio::test]
async fn test_voice_report_manifest_and_registry() {
    let ctx = setup_test_context().await;

    // AC1: Lấy danh sách module và kiểm tra manifest của voice-report
    let req = Request::builder()
        .uri("/api/v1/features/modules")
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();

    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let features = body["modules"].as_array().expect("Mảng danh sách module");

    let voice_mod = features
        .iter()
        .find(|f| f["id"] == "voice-report")
        .expect("Phải tìm thấy module voice-report trong registry");

    assert_eq!(voice_mod["name"], "Giọng đọc báo cáo");
    assert_eq!(voice_mod["risk"], "low");
    let requires = voice_mod["requires"].as_array().expect("Mảng requires");
    assert!(
        requires.iter().any(|r| r == "reports"),
        "voice-report phải khai báo requires reports"
    );
}

#[tokio::test]
async fn test_voice_report_enable_with_dependency() {
    let ctx = setup_test_context().await;

    // AC2: Lập kế hoạch bật voice-report -> phải bao gồm phụ thuộc reports
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/features/plan-enable")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::from(
            json!({
                "module_id": "voice-report",
                "scope": "workspace",
                "target_id": ctx.workspace_id
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let plan_resp: Value = serde_json::from_slice(&bytes).unwrap();

    let will_enable = plan_resp["plan"]["will_enable_with"]
        .as_array()
        .expect("Mảng will_enable_with");
    assert!(
        will_enable.iter().any(|m| m == "reports"),
        "Kế hoạch bật voice-report phải kéo theo module reports"
    );
    assert!(
        will_enable.iter().any(|m| m == "voice-report"),
        "Kế hoạch phải chứa chính voice-report"
    );

    // Áp dụng bật reports trước qua apply_feature
    let apply_req1 = Request::builder()
        .method("POST")
        .uri("/api/v1/features/apply")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::from(
            json!({
                "module_id": "reports",
                "scope": "workspace",
                "target_id": ctx.workspace_id,
                "enable": true
            })
            .to_string(),
        ))
        .unwrap();

    let apply_resp1 = ctx.app.clone().oneshot(apply_req1).await.unwrap();
    assert_eq!(apply_resp1.status(), StatusCode::OK);

    // Áp dụng bật voice-report qua apply_feature
    let apply_req2 = Request::builder()
        .method("POST")
        .uri("/api/v1/features/apply")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::from(
            json!({
                "module_id": "voice-report",
                "scope": "workspace",
                "target_id": ctx.workspace_id,
                "enable": true
            })
            .to_string(),
        ))
        .unwrap();

    let apply_resp2 = ctx.app.clone().oneshot(apply_req2).await.unwrap();
    assert_eq!(apply_resp2.status(), StatusCode::OK);

    // Tra cứu effective state
    let eff_req = Request::builder()
        .uri(format!(
            "/api/v1/features/effective?module_id=voice-report&workspace={}",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();

    let eff_resp = ctx.app.clone().oneshot(eff_req).await.unwrap();
    assert_eq!(eff_resp.status(), StatusCode::OK);
    let eff_bytes = to_bytes(eff_resp.into_body(), usize::MAX).await.unwrap();
    let eff_state: Value = serde_json::from_slice(&eff_bytes).unwrap();
    assert_eq!(eff_state["effective"]["enabled"], true);
}

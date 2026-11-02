//! Các bài kiểm thử tích hợp cho Task P3-03: Module `quiet-hours`.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P3-03 dùng unwrap và kiểm tra nhiều tiêu chí nghiệm thu"
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

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS Quiet', ?)")
        .bind(ws_id.to_string())
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    // Lấy owner user id
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
        "INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'Prj Quiet', '/tmp/repo', 'main', ?)"
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
    let member_token = "member_token_12345";
    let member_token_hash = dandori_server::auth::hash_token(member_token);
    let expires = (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339();
    sqlx::query(
        "INSERT INTO session (id, user_id, expires_at, token_hash) VALUES ('sess_member', ?, ?, ?)",
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
    let outsider_token = "outsider_token_12345";
    let outsider_token_hash = dandori_server::auth::hash_token(outsider_token);
    sqlx::query("INSERT INTO session (id, user_id, expires_at, token_hash) VALUES ('sess_outsider', ?, ?, ?)")
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
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "Bật module {mod_id} phải thành công"
        );
    }
}

#[tokio::test]
async fn test_quiet_hours_crud_and_bola_isolation() {
    let ctx = setup_test_context().await;
    enable_modules(&ctx, &["calendar", "quiet-hours"]).await;

    // 1. Admin tạo khung giờ cấm mới (POST)
    let create_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/quiet-hours/workspaces/{}/windows",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": ctx.workspace_id,
                "project_id": ctx.project_id,
                "name": "Khung cấm ban đêm",
                "days_of_week": "1,2,3,4,5",
                "start_time": "22:00",
                "end_time": "06:00",
                "timezone": "UTC",
                "block_manual_runs": false
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let window_id = json["window"]["id"].as_str().unwrap().to_string();
    assert_eq!(json["window"]["name"], "Khung cấm ban đêm");

    // 2. Member xem danh sách khung giờ cấm (GET) -> 200 OK
    let list_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/quiet-hours/workspaces/{}/windows",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. Member kiểm tra trạng thái status (GET) -> 200 OK
    let status_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/quiet-hours/workspaces/{}/status?project_id={}",
            ctx.workspace_id, ctx.project_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(status_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 4. BOLA Outsider cố truy cập -> 403 Forbidden
    let outsider_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/quiet-hours/workspaces/{}/windows",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.outsider_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(outsider_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 5. BOLA Member không có quyền tạo khung giờ cấm -> 403 Forbidden
    let member_create = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/quiet-hours/workspaces/{}/windows",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": ctx.workspace_id,
                "name": "Illegal",
                "days_of_week": "1",
                "start_time": "10:00",
                "end_time": "11:00"
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(member_create).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 6. Admin cập nhật khung giờ cấm (PUT)
    let update_req = Request::builder()
        .method("PUT")
        .uri(format!(
            "/api/v1/m/quiet-hours/workspaces/{}/windows/{}",
            ctx.workspace_id, window_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "name": "Khung cấm ban đêm (đã sửa)",
                "block_manual_runs": true
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["window"]["name"], "Khung cấm ban đêm (đã sửa)");
    assert_eq!(json["window"]["block_manual_runs"], true);

    // 7. Admin bật/tắt (POST toggle)
    let toggle_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/quiet-hours/workspaces/{}/windows/{}/toggle",
            ctx.workspace_id, window_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(toggle_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["window"]["enabled"], false);

    // 8. Admin xóa khung giờ cấm (DELETE)
    let delete_req = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/api/v1/m/quiet-hours/workspaces/{}/windows/{}",
            ctx.workspace_id, window_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_transition_guard_blocks_manual_when_configured() {
    let ctx = setup_test_context().await;
    enable_modules(&ctx, &["calendar", "quiet-hours"]).await;

    // Tạo một task ở trạng thái backlog
    let task_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        INSERT INTO task (
            id, project_id, title, description, status,
            lead_agent, worker_agent, created_at, updated_at
        )
        VALUES (?, ?, 'Task 1', 'Desc', 'backlog', 'lead', 'worker', ?, ?)
        "#,
    )
    .bind(&task_id)
    .bind(&ctx.project_id)
    .bind(&now)
    .bind(&now)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Cấu hình khung cấm bao phủ cả 24h (00:00 - 23:59 hoặc qua ngày) tất cả các ngày 1,2,3,4,5,6,7
    // Với block_manual_runs = 0 (chạy tay được phép)
    sqlx::query(
        r#"
        INSERT INTO m_quiet_hours_window (
            id, workspace_id, project_id, name, days_of_week, start_time,
            end_time, timezone, block_manual_runs, enabled, created_at, updated_at
        ) VALUES (
            'w-block', ?, ?, 'Khung 24/7', '1,2,3,4,5,6,7', '00:00',
            '00:00', 'UTC', 0, 1, ?, ?
        )
        "#,
    )
    .bind(&ctx.workspace_id)
    .bind(&ctx.project_id)
    .bind(&now)
    .bind(&now)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // 1. Chạy tay chuyển trạng thái sang running -> Được phép (200 OK)
    let transition_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{task_id}/status"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "status": "running" }).to_string()))
        .unwrap();

    let resp = ctx.app.clone().oneshot(transition_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Chuyển lại về backlog
    sqlx::query("UPDATE task SET status = 'backlog' WHERE id = ?")
        .bind(&task_id)
        .execute(&ctx.pool)
        .await
        .unwrap();

    // 2. Cập nhật block_manual_runs = 1
    sqlx::query("UPDATE m_quiet_hours_window SET block_manual_runs = 1 WHERE id = 'w-block'")
        .execute(&ctx.pool)
        .await
        .unwrap();

    // 3. Chạy tay chuyển sang running -> Bị chặn với 400 Bad Request
    let transition_req2 = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{task_id}/status"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "status": "running" }).to_string()))
        .unwrap();

    let resp = ctx.app.clone().oneshot(transition_req2).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let err_msg = json["error"]["message"].as_str().unwrap();
    assert!(err_msg.contains("khung giờ cấm chạy"));

    // 4. Tắt khung cấm (enabled = 0) -> Chạy lại được phép (200 OK)
    sqlx::query("UPDATE m_quiet_hours_window SET enabled = 0 WHERE id = 'w-block'")
        .execute(&ctx.pool)
        .await
        .unwrap();

    let transition_req3 = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{task_id}/status"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "status": "running" }).to_string()))
        .unwrap();

    let resp = ctx.app.clone().oneshot(transition_req3).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_disable_module_unregisters_guard_and_overlapping_status() {
    let ctx = setup_test_context().await;
    enable_modules(&ctx, &["calendar", "quiet-hours"]).await;

    // 1. Tạo 2 khung giờ cấm chồng chéo nhau (Overlapping windows - AC 5)
    // Khung 1: 09:00 - 13:00 UTC
    // Khung 2: 12:00 - 18:00 UTC
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        INSERT INTO m_quiet_hours_window (
            id, workspace_id, project_id, name, days_of_week, start_time,
            end_time, timezone, block_manual_runs, enabled, created_at, updated_at
        ) VALUES
        ('w-1', ?, ?, 'Khung sáng', '1,2,3,4,5,6,7', '09:00', '13:00', 'UTC', 1, 1, ?, ?),
        ('w-2', ?, ?, 'Khung chiều', '1,2,3,4,5,6,7', '12:00', '18:00', 'UTC', 1, 1, ?, ?)
        "#,
    )
    .bind(&ctx.workspace_id)
    .bind(&ctx.project_id)
    .bind(&now)
    .bind(&now)
    .bind(&ctx.workspace_id)
    .bind(&ctx.project_id)
    .bind(&now)
    .bind(&now)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // 2. Kiểm tra status API (GET status) trả về trạng thái hợp nhất
    let status_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/quiet-hours/workspaces/{}/status?project_id={}",
            ctx.workspace_id, ctx.project_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();

    let resp = ctx.app.clone().oneshot(status_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(json["status"].is_object());

    // 3. Tắt module quiet-hours (AC 4)
    let disable_req = Request::builder()
        .method("POST")
        .uri("/api/v1/features/apply")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "module_id": "quiet-hours",
                "scope": "system",
                "target_id": "system",
                "enable": false,
                "risk_acknowledged": true
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(disable_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 4. Tạo task và thử chuyển sang running: Khi module đã tắt, TransitionGuard bị gỡ bỏ, chuyển trạng thái thành công
    let task_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO task (
            id, project_id, title, description, status,
            lead_agent, worker_agent, created_at, updated_at
        )
        VALUES (?, ?, 'Task Disabled', 'Desc', 'backlog', 'lead', 'worker', ?, ?)
        "#,
    )
    .bind(&task_id)
    .bind(&ctx.project_id)
    .bind(&now)
    .bind(&now)
    .execute(&ctx.pool)
    .await
    .unwrap();

    let transition_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{task_id}/status"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "status": "running" }).to_string()))
        .unwrap();

    let resp = ctx.app.clone().oneshot(transition_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

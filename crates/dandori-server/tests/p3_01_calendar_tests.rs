//! Các bài kiểm thử tích hợp cho Task P3-01: Module `calendar`.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P3-01 dùng unwrap và kiểm tra nhiều tiêu chí nghiệm thu"
)]

use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
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
        .to_string();

    // 2. Setup Member & Outsider
    let now = Utc::now().to_rfc3339();
    let member_user_id = "user_member";
    let outsider_user_id = "user_outsider";

    sqlx::query("INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, 'member', 'hash', 'user', ?)")
        .bind(member_user_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, 'outsider', 'hash', 'user', ?)")
        .bind(outsider_user_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let member_token = "member_token_12345";
    let outsider_token = "outsider_token_12345";
    let member_token_hash = dandori_server::auth::hash_token(member_token);
    let outsider_token_hash = dandori_server::auth::hash_token(outsider_token);
    let expires = (Utc::now() + chrono::Duration::days(1)).to_rfc3339();

    sqlx::query(
        "INSERT INTO session (id, user_id, expires_at, token_hash) VALUES ('sess_member', ?, ?, ?)",
    )
    .bind(member_user_id)
    .bind(&expires)
    .bind(&member_token_hash)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO session (id, user_id, expires_at, token_hash) VALUES ('sess_outsider', ?, ?, ?)")
        .bind(outsider_user_id)
        .bind(&expires)
        .bind(&outsider_token_hash)
        .execute(&pool)
        .await
        .unwrap();

    let member_cookie = format!("dandori_session={member_token}");
    let outsider_cookie = format!("dandori_session={outsider_token}");

    // 3. Setup Workspace & Project
    let ws_id = WorkspaceId::new().to_string();
    let prj_id = ProjectId::new().to_string();

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS 1', ?)")
        .bind(&ws_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'PRJ 1', 'd:/repo', 'main', ?)")
        .bind(&prj_id)
        .bind(&ws_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let owner_user = sqlx::query("SELECT id FROM user WHERE username = 'owner'")
        .fetch_one(&pool)
        .await
        .unwrap();
    let owner_user_id: String = sqlx::Row::get(&owner_user, "id");

    // Owner is admin of WS 1
    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'admin')",
    )
    .bind(&ws_id)
    .bind(&owner_user_id)
    .execute(&pool)
    .await
    .unwrap();

    // Member is member of WS 1
    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'member')",
    )
    .bind(&ws_id)
    .bind(member_user_id)
    .execute(&pool)
    .await
    .unwrap();

    // Outsider is NOT member of WS 1

    TestContext {
        app,
        pool,
        lifecycle_manager,
        event_bus,
        owner_cookie,
        member_cookie,
        outsider_cookie,
        workspace_id: ws_id,
        project_id: prj_id,
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
async fn test_p3_01_ac6_bola_authorization_and_crud() {
    let ctx = setup_test_context().await;
    enable_modules(&ctx, &["subscription-connect", "calendar"]).await;

    let payload = json!({
        "project_id": ctx.project_id,
        "title": "Kiểm thử Lịch 1",
        "schedule_type": "cron",
        "schedule_expr": "0 * * * *",
        "timezone": "UTC",
        "task_template": {
            "title": "Nhiệm vụ tự động",
            "description": "Mô tả tự động",
            "lead_agent": "claude",
            "worker_agent": "codestral"
        },
        "auto_dispatch": true,
        "missed_policy": "skip"
    });

    // 1. Outsider không thuộc Workspace tạo lịch -> 403 Forbidden
    let req_outsider = Request::builder()
        .method("POST")
        .uri("/api/v1/m/calendar/schedules")
        .header(header::COOKIE, &ctx.outsider_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req_outsider).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "Outsider không có quyền tạo lịch"
    );

    // 2. Member bình thường tạo lịch -> 403 Forbidden (Yêu cầu Workspace Admin)
    let req_member = Request::builder()
        .method("POST")
        .uri("/api/v1/m/calendar/schedules")
        .header(header::COOKIE, &ctx.member_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req_member).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "Member thường không được phép quản trị lịch"
    );

    // 3. Admin tạo lịch -> 201 Created
    let req_admin = Request::builder()
        .method("POST")
        .uri("/api/v1/m/calendar/schedules")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req_admin).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::CREATED,
        "Admin tạo lịch thành công"
    );

    let body_bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let res_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    let created = &res_json["schedule"];
    assert_eq!(created["title"], "Kiểm thử Lịch 1");
    assert!(created["next_fire"].is_string());
    let sched_id = created["id"].as_str().unwrap().to_string();

    // 4. Member có quyền XEM danh sách lịch theo workspace -> 200 OK
    let req_list_member = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/calendar/schedules?workspace_id={}",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(req_list_member).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "Member xem được danh sách lịch trong workspace"
    );

    // Outsider xem danh sách lịch -> 403 Forbidden
    let req_list_outsider = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/calendar/schedules?workspace_id={}",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.outsider_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(req_list_outsider).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "Outsider bị chặn xem lịch"
    );

    // 5. Member thử Tạm dừng lịch -> 403 Forbidden
    let req_pause_member = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/m/calendar/schedules/{}/pause", sched_id))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(req_pause_member).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "Member không được tạm dừng lịch"
    );

    // 6. Admin Tạm dừng lịch -> 200 OK
    let req_pause_admin = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/m/calendar/schedules/{}/pause", sched_id))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(req_pause_admin).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let paused_bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let paused: Value = serde_json::from_slice(&paused_bytes).unwrap();
    assert_eq!(paused["enabled"], false);

    // 7. Admin Tiếp tục lịch -> 200 OK
    let req_resume_admin = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/m/calendar/schedules/{}/resume", sched_id))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(req_resume_admin).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let resumed_bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let resumed: Value = serde_json::from_slice(&resumed_bytes).unwrap();
    assert_eq!(resumed["enabled"], true);

    // 8. Admin Fire Now -> 200 OK
    let req_fire_admin = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/calendar/schedules/{}/fire-now",
            sched_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(req_fire_admin).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Kiểm tra task được tạo trong database
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM task WHERE project_id = ?")
        .bind(&ctx.project_id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    assert_eq!(count, 1, "Fire-now phải tạo đúng 1 task backlog");

    // 9. Admin Xóa lịch -> 200 OK
    let req_del_admin = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/m/calendar/schedules/{}", sched_id))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(req_del_admin).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_p3_01_ac4_disable_and_enable_lifecycle() {
    let ctx = setup_test_context().await;
    enable_modules(&ctx, &["subscription-connect", "calendar"]).await;

    // Tắt module calendar
    let disable_req = Request::builder()
        .method("POST")
        .uri("/api/v1/features/apply")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "module_id": "calendar",
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

    // Khi tắt module calendar, API route trả về 403 FORBIDDEN (module_disabled)
    let req_list = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/calendar/schedules?workspace_id={}",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(req_list).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Bật lại module calendar
    let re_enable_req = Request::builder()
        .method("POST")
        .uri("/api/v1/features/apply")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "module_id": "calendar",
                "scope": "system",
                "target_id": "system",
                "enable": true,
                "risk_acknowledged": true
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(re_enable_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Sau khi bật lại, API hoạt động trở lại bình thường -> 200 OK
    let req_list_again = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/calendar/schedules?workspace_id={}",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(req_list_again).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

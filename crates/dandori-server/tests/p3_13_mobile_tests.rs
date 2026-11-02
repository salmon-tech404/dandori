//! Các bài kiểm thử tích hợp cho Task P3-13: Module `mobile` (Mobile Companion & Approval Gate).

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    clippy::indexing_slicing,
    reason = "Kiểm thử tích hợp P3-13 dùng unwrap và kiểm tra toàn bộ tiêu chí AC1-AC5"
)]

use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use dandori_events::EventBus;
use dandori_registry::FeatureRegistry;
use dandori_registry::lifecycle::FeatureLifecycleManager;
use dandori_server::app::create_app;
use dandori_server::config::ServerConfig;
use dandori_server::run_service::RunService;
use dandori_server::state::AppState;
use dandori_types::{ProjectId, TaskId, WorkspaceId};
use serde_json::{Value, json};
use sqlx::{Row, SqlitePool};
use tempfile::tempdir;
use tower::ServiceExt;

struct TestContext {
    app: axum::Router,
    pool: SqlitePool,
    owner_cookie: String,
    outsider_cookie: String,
    workspace_a_id: String,
    workspace_b_id: String,
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
    let workspace_a_id = WorkspaceId::new().to_string();
    let workspace_b_id = WorkspaceId::new().to_string();

    // 2. Setup Outsider User trực tiếp
    let outsider_user_id = "user_outsider_mobile_99";
    sqlx::query("INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, 'outsider', 'hash', 'user', ?)")
        .bind(outsider_user_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let outsider_token = "outsider_token_mobile_99";
    let outsider_token_hash = dandori_server::auth::hash_token(outsider_token);
    let expires = (chrono::Utc::now() + chrono::Duration::days(1)).to_rfc3339();

    sqlx::query(
        "INSERT INTO session (id, user_id, expires_at, token_hash) VALUES ('sess_outsider_mob', ?, ?, ?)",
    )
    .bind(outsider_user_id)
    .bind(&expires)
    .bind(&outsider_token_hash)
    .execute(&pool)
    .await
    .unwrap();
    let outsider_cookie = format!("dandori_session={outsider_token}");

    // 3. Tạo Workspace A (Owner làm admin) và Workspace B (Outsider làm admin)
    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'Workspace A', ?)")
        .bind(&workspace_a_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'Workspace B', ?)")
        .bind(&workspace_b_id)
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
    .bind(&workspace_a_id)
    .bind(&owner_user_id)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'admin')",
    )
    .bind(&workspace_b_id)
    .bind(outsider_user_id)
    .execute(&pool)
    .await
    .unwrap();

    // 4. Kích hoạt module mobile và notifications qua features/apply API
    let enable_mobile = Request::builder()
        .method("POST")
        .uri("/api/v1/features/apply")
        .header(header::COOKIE, &owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "module_id": "mobile",
                "scope": "workspace",
                "target_id": workspace_a_id,
                "enable": true,
                "risk_acknowledged": true
            })
            .to_string(),
        ))
        .unwrap();
    let resp_mob = app.clone().oneshot(enable_mobile).await.unwrap();
    assert_eq!(resp_mob.status(), StatusCode::OK);

    let enable_notif = Request::builder()
        .method("POST")
        .uri("/api/v1/features/apply")
        .header(header::COOKIE, &owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "module_id": "notifications",
                "scope": "workspace",
                "target_id": workspace_a_id,
                "enable": true,
                "risk_acknowledged": true
            })
            .to_string(),
        ))
        .unwrap();
    let resp_notif = app.clone().oneshot(enable_notif).await.unwrap();
    assert_eq!(resp_notif.status(), StatusCode::OK);

    TestContext {
        app,
        pool,
        owner_cookie,
        outsider_cookie,
        workspace_a_id,
        workspace_b_id,
        temp_dir,
    }
}

/// Helper tạo thiết bị di động mới trong workspace và trả về (device_id, raw_token)
async fn create_test_device(ctx: &TestContext, ws_id: &str, device_name: &str) -> (String, String) {
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/m/mobile/workspaces/{ws_id}/devices"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "name": device_name, "read_only": false }).to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let device_id = body["device"]["id"].as_str().unwrap().to_string();
    let token = body["token"].as_str().unwrap().to_string();
    (device_id, token)
}

#[tokio::test]
async fn test_ac1_token_scoping_and_forbidden_routes() {
    let ctx = setup_test_context().await;
    let (_dev_id, token) = create_test_device(&ctx, &ctx.workspace_a_id, "iPhone AC1").await;
    assert!(token.starts_with("mob_"));

    // 1. Gọi route trong phạm vi hẹp: /api/v1/m/mobile/status -> 200 OK
    let req_status = Request::builder()
        .method("GET")
        .uri("/api/v1/m/mobile/status")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp_status = ctx.app.clone().oneshot(req_status).await.unwrap();
    assert_eq!(resp_status.status(), StatusCode::OK);

    // 2. Gọi route trong phạm vi hẹp: /api/v1/m/mobile/approvals -> 200 OK
    let req_appr = Request::builder()
        .method("GET")
        .uri("/api/v1/m/mobile/approvals")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp_appr = ctx.app.clone().oneshot(req_appr).await.unwrap();
    assert_eq!(resp_appr.status(), StatusCode::OK);

    // 3. Gọi route module khác: /api/v1/m/notifications/... -> 403 Forbidden (forbidden_scope)
    let req_notif = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/notifications/workspaces/{}/notifications/channels",
            ctx.workspace_a_id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp_notif = ctx.app.clone().oneshot(req_notif).await.unwrap();
    assert_eq!(resp_notif.status(), StatusCode::FORBIDDEN);
    let bytes = to_bytes(resp_notif.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["error"]["code"], "forbidden_scope");

    // 4. Gọi route quản trị bên trong mobile bằng mobile token -> bị từ chối (401/403)
    let req_admin = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/mobile/workspaces/{}/devices",
            ctx.workspace_a_id
        ))
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp_admin = ctx.app.clone().oneshot(req_admin).await.unwrap();
    assert!(
        resp_admin.status() == StatusCode::FORBIDDEN
            || resp_admin.status() == StatusCode::UNAUTHORIZED,
        "Mobile token không được phép gọi route quản trị thiết bị"
    );

    // 5. Gọi route lõi /api/v1/workspaces bằng mobile token -> 401 Unauthorized (vì không có web session)
    let req_core = Request::builder()
        .method("GET")
        .uri("/api/v1/workspaces")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp_core = ctx.app.clone().oneshot(req_core).await.unwrap();
    assert_eq!(resp_core.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_ac2_expired_and_revoked_tokens_and_hash_storage() {
    let ctx = setup_test_context().await;
    let (dev_id, token) = create_test_device(&ctx, &ctx.workspace_a_id, "iPhone AC2").await;

    // 1. Kiểm tra lưu băm SHA-256 trong DB: không có raw token trong cột
    let row = sqlx::query("SELECT token_hash FROM m_mobile_device WHERE id = ?")
        .bind(&dev_id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    let db_hash: String = row.get("token_hash");
    assert_ne!(db_hash, token);
    let expected_hash = dandori_db::hash_session_token(&token);
    assert_eq!(db_hash, expected_hash);

    // 2. Thu hồi thiết bị
    let req_revoke = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/mobile/workspaces/{}/devices/{dev_id}/revoke",
            ctx.workspace_a_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp_revoke = ctx.app.clone().oneshot(req_revoke).await.unwrap();
    assert_eq!(resp_revoke.status(), StatusCode::OK);

    // 3. Sau khi thu hồi, gọi /status bằng token đó bị từ chối 401 Unauthorized
    let req_revoked_call = Request::builder()
        .method("GET")
        .uri("/api/v1/m/mobile/status")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp_revoked_call = ctx.app.clone().oneshot(req_revoked_call).await.unwrap();
    assert_eq!(resp_revoked_call.status(), StatusCode::UNAUTHORIZED);

    // 4. Kiểm tra token hết hạn: tạo thiết bị mới và đặt expires_at về quá khứ
    let (dev2_id, token2) = create_test_device(&ctx, &ctx.workspace_a_id, "iPhone Expired").await;
    let past = (chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
    sqlx::query("UPDATE m_mobile_device SET expires_at = ? WHERE id = ?")
        .bind(&past)
        .bind(&dev2_id)
        .execute(&ctx.pool)
        .await
        .unwrap();

    let req_expired = Request::builder()
        .method("GET")
        .uri("/api/v1/m/mobile/status")
        .header(header::AUTHORIZATION, format!("Bearer {token2}"))
        .body(Body::empty())
        .unwrap();
    let resp_expired = ctx.app.clone().oneshot(req_expired).await.unwrap();
    assert_eq!(resp_expired.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_ac3_origin_allowlist_check() {
    let ctx = setup_test_context().await;
    let (_dev_id, token) = create_test_device(&ctx, &ctx.workspace_a_id, "iPhone AC3").await;

    // 1. Origin lạ không có trong allowlist bị từ chối 403 Forbidden
    let req_bad_origin = Request::builder()
        .method("GET")
        .uri("/api/v1/m/mobile/status")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::ORIGIN, "https://malicious-external-site.com")
        .body(Body::empty())
        .unwrap();
    let resp_bad_origin = ctx.app.clone().oneshot(req_bad_origin).await.unwrap();
    assert_eq!(resp_bad_origin.status(), StatusCode::FORBIDDEN);

    // 2. Cập nhật cấu hình allowed_origins của workspace A
    let req_update_cfg = Request::builder()
        .method("PUT")
        .uri(format!(
            "/api/v1/m/mobile/workspaces/{}/config",
            ctx.workspace_a_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "allowed_origins": ["https://vpn.dandori.internal", "https://*.tunnel.mycompany.com"],
                "read_only": false
            })
            .to_string(),
        ))
        .unwrap();
    let resp_update_cfg = ctx.app.clone().oneshot(req_update_cfg).await.unwrap();
    assert_eq!(resp_update_cfg.status(), StatusCode::OK);

    // 3. Gọi lại với Origin đã được phép -> 200 OK
    let req_good_origin = Request::builder()
        .method("GET")
        .uri("/api/v1/m/mobile/status")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::ORIGIN, "https://vpn.dandori.internal")
        .body(Body::empty())
        .unwrap();
    let resp_good_origin = ctx.app.clone().oneshot(req_good_origin).await.unwrap();
    assert_eq!(resp_good_origin.status(), StatusCode::OK);

    // 4. Gọi lại với Origin khớp wildcard subdomain -> 200 OK
    let req_wildcard_origin = Request::builder()
        .method("GET")
        .uri("/api/v1/m/mobile/status")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::ORIGIN, "https://my-phone.tunnel.mycompany.com")
        .body(Body::empty())
        .unwrap();
    let resp_wildcard = ctx.app.clone().oneshot(req_wildcard_origin).await.unwrap();
    assert_eq!(resp_wildcard.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_ac4_workspace_isolation() {
    let ctx = setup_test_context().await;

    // 1. Outsider không thuộc Workspace A gọi danh sách thiết bị của Workspace A -> 403 Forbidden
    let req_outsider_devices = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/mobile/workspaces/{}/devices",
            ctx.workspace_a_id
        ))
        .header(header::COOKIE, &ctx.outsider_cookie)
        .body(Body::empty())
        .unwrap();
    let resp_outsider = ctx.app.clone().oneshot(req_outsider_devices).await.unwrap();
    assert_eq!(resp_outsider.status(), StatusCode::FORBIDDEN);

    // 2. Thiết bị A thuộc workspace A
    let (_dev_a, token_a) = create_test_device(&ctx, &ctx.workspace_a_id, "Device WS-A").await;

    // Tạo một dự án và task trong workspace B
    let proj_b_id = ProjectId::new().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'Project B', '/tmp/repo_b', 'main', ?)")
        .bind(&proj_b_id)
        .bind(&ctx.workspace_b_id)
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    let task_b_id = TaskId::new().to_string();
    sqlx::query("INSERT INTO task (id, project_id, title, description, lead_agent, worker_agent, status, created_at, updated_at) VALUES (?, ?, 'Task in WS B', 'Need approval', 'lead', 'worker', 'needs_human', ?, ?)")
        .bind(&task_b_id)
        .bind(&proj_b_id)
        .bind(&now)
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    // Thiết bị A gửi quyết định duyệt task thuộc workspace B -> 403 Forbidden
    let req_cross = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/m/mobile/approvals/{task_b_id}/decision"))
        .header(header::AUTHORIZATION, format!("Bearer {token_a}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "action": "approve" }).to_string()))
        .unwrap();

    let resp_cross = ctx.app.clone().oneshot(req_cross).await.unwrap();
    assert_eq!(resp_cross.status(), StatusCode::FORBIDDEN);
}

#[test]
fn test_ac5_default_config_loopback_only() {
    let default_cfg = ServerConfig::default();
    assert_eq!(
        default_cfg.host, "127.0.0.1",
        "ServerConfig mặc định bắt buộc là 127.0.0.1 (RS-SEC-006)"
    );
    assert_ne!(
        default_cfg.host, "0.0.0.0",
        "ServerConfig không được phép bind ra 0.0.0.0 mặc định"
    );
}

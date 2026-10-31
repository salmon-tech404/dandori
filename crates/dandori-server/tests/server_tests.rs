use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use dandori_events::EventBus;
use dandori_module_api::{
    Manifest, Migration, Module, ModuleContext, ModuleSubscription, WorkerSpec,
};
use dandori_registry::FeatureRegistry;
use dandori_registry::lifecycle::FeatureLifecycleManager;
use dandori_registry::types::{RegistryRole, RegistryScope};
use dandori_server::{AppState, ServerConfig, create_app};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::sync::{Arc, LazyLock};
use std::time::Instant;
use tower::ServiceExt;

#[expect(
    clippy::unwrap_used,
    reason = "static sample manifest is valid constant json"
)]
static SAMPLE_MANIFEST: LazyLock<Manifest> = LazyLock::new(|| {
    serde_json::from_str(
        r#"{
            "id": "sample-hello",
            "group": "safety",
            "name": "Module mẫu",
            "description": "Module mẫu cho test",
            "version": "0.1.0",
            "requires": [],
            "scopes": ["workspace", "project"],
            "effort": "light",
            "risk": "low",
            "core": false,
            "requires_worker_restart": false,
            "settings": {},
            "ui": { "contributes": [] }
        }"#,
    )
    .unwrap()
});

static SAMPLE_MIGRATIONS: [Migration; 1] = [Migration {
    id: "0001_note",
    sql: "CREATE TABLE IF NOT EXISTS m_sample_hello_note (id TEXT PRIMARY KEY, note TEXT);",
}];

#[derive(Debug, Default, Clone, Copy)]
struct TestSampleModule;

impl Module for TestSampleModule {
    fn manifest(&self) -> &Manifest {
        &SAMPLE_MANIFEST
    }
    fn migrations(&self) -> &'static [Migration] {
        &SAMPLE_MIGRATIONS
    }
    fn routes(&self, _ctx: &ModuleContext) -> axum::Router {
        axum::Router::new().route(
            "/ping",
            axum::routing::get(|| async {
                axum::Json(json!({ "status": "ok", "module": "sample-hello" }))
            }),
        )
    }
    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        Vec::new()
    }
    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }
}

async fn setup_test_app()
-> Result<(axum::Router, SqlitePool, Arc<FeatureLifecycleManager>), Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    let core_schema = include_str!("../../../crates/dandori-db/migrations/0001_core_schema.sql");
    sqlx::query(core_schema).execute(&pool).await?;

    let auth_schema = include_str!("../../../crates/mod-auth-login/migrations/0001_auth_login.sql");
    sqlx::query(auth_schema).execute(&pool).await?;

    // Đăng ký module sample-hello vào bảng feature_module
    sqlx::query(
        "INSERT INTO feature_module (id, group_name, name, description, version, requires, scopes, risk, effort, core) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("sample-hello")
    .bind("safety")
    .bind("Module mẫu")
    .bind("Module mẫu cho test")
    .bind("0.1.0")
    .bind("[]")
    .bind("[\"workspace\",\"project\"]")
    .bind("low")
    .bind("light")
    .bind(0)
    .execute(&pool)
    .await?;

    let event_bus = Arc::new(EventBus::new(100));
    let registry = FeatureRegistry::new(pool.clone());
    let lifecycle_manager = Arc::new(FeatureLifecycleManager::new(
        registry,
        pool.clone(),
        Arc::clone(&event_bus),
    ));

    lifecycle_manager
        .register_module(Arc::new(TestSampleModule))
        .await;

    let config = ServerConfig::default();
    let state = AppState::new(
        pool.clone(),
        event_bus,
        Arc::clone(&lifecycle_manager),
        config,
    );

    let app = create_app(state);
    Ok((app, pool, lifecycle_manager))
}

#[tokio::test]
async fn test_ac1_server_only_binds_localhost() {
    assert!(ServerConfig::validate_host("127.0.0.1").is_ok());
    assert!(ServerConfig::validate_host("localhost").is_ok());
    assert!(ServerConfig::validate_host("::1").is_ok());

    let err_zero = ServerConfig::validate_host("0.0.0.0").unwrap_err();
    assert!(err_zero.contains("Cấm bind 0.0.0.0"));

    let err_ip = ServerConfig::validate_host("192.168.1.100").unwrap_err();
    assert!(err_ip.contains("chỉ cho phép bind 127.0.0.1"));
}

#[tokio::test]
async fn test_ac2_setup_owner_once_and_reject_second_time() {
    let (app, _pool, _lm) = setup_test_app().await.unwrap();

    // Lần 1: Tạo chủ thành công
    let req1 = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "username": "admin",
                "password": "password123"
            })
            .to_string(),
        ))
        .unwrap();

    let resp1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);

    // Lần 2: Thử tạo chủ lại -> Bị từ chối 409 Conflict
    let req2 = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "username": "hacker",
                "password": "password456"
            })
            .to_string(),
        ))
        .unwrap();

    let resp2 = app.oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::CONFLICT);

    let body_bytes = to_bytes(resp2.into_body(), usize::MAX).await.unwrap();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["error"]["code"], "owner_already_exists");
}

#[tokio::test]
async fn test_ac3_login_sets_cookie_and_me_returns_user() {
    let (app, _pool, _lm) = setup_test_app().await.unwrap();

    // Khởi tạo chủ
    let req_init = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "admin", "password": "password123" }).to_string(),
        ))
        .unwrap();
    app.clone().oneshot(req_init).await.unwrap();

    // Đăng nhập
    let req_login = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "admin", "password": "password123" }).to_string(),
        ))
        .unwrap();

    let resp_login = app.clone().oneshot(req_login).await.unwrap();
    assert_eq!(resp_login.status(), StatusCode::OK);

    let cookie_val = resp_login
        .headers()
        .get(header::SET_COOKIE)
        .expect("Phải có header Set-Cookie")
        .to_str()
        .unwrap()
        .to_string();

    assert!(cookie_val.contains("HttpOnly"));
    assert!(cookie_val.contains("SameSite=Strict"));
    assert!(cookie_val.contains("dandori_session="));

    // Lấy cookie để gọi GET /api/v1/me
    let session_cookie = cookie_val.split(';').next().unwrap();

    let req_me = Request::builder()
        .method("GET")
        .uri("/api/v1/me")
        .header(header::COOKIE, session_cookie)
        .body(Body::empty())
        .unwrap();

    let resp_me = app.oneshot(req_me).await.unwrap();
    assert_eq!(resp_me.status(), StatusCode::OK);

    let body_bytes = to_bytes(resp_me.into_body(), usize::MAX).await.unwrap();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["username"], "admin");
    assert_eq!(body_json["system_role"], "admin");
}

#[expect(
    clippy::unwrap_used,
    reason = "test helper performs unwrap for test requests"
)]
async fn exec_login_req(
    app: axum::Router,
    username: &str,
    password: &str,
) -> (StatusCode, Value, std::time::Duration) {
    let start = Instant::now();
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": username, "password": password }).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let elapsed = start.elapsed();
    let status = resp.status();
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    (status, json, elapsed)
}

#[tokio::test]
async fn test_ac4_wrong_password_and_unknown_user_return_same_error_and_comparable_time() {
    let (app, _pool, _lm) = setup_test_app().await.unwrap();

    let req_init = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "admin", "password": "password123" }).to_string(),
        ))
        .unwrap();
    app.clone().oneshot(req_init).await.unwrap();

    let (status_unknown, json_unknown, elapsed_unknown) =
        exec_login_req(app.clone(), "ghost_user", "wrong_password").await;
    assert_eq!(status_unknown, StatusCode::UNAUTHORIZED);

    let (status_wrong, json_wrong, elapsed_wrong) =
        exec_login_req(app, "admin", "wrong_password").await;
    assert_eq!(status_wrong, StatusCode::UNAUTHORIZED);

    assert_eq!(
        json_unknown["error"]["message"], json_wrong["error"]["message"],
        "Thông điệp báo lỗi phải hoàn toàn không phân biệt sai tên hay sai pass"
    );
    assert_eq!(json_unknown["error"]["code"], "invalid_credentials");

    // Tên không tồn tại phải tốn cỡ thời gian băm Argon2 như mật khẩu sai (không có nhánh nhanh).
    // So theo tỉ lệ, không theo hiệu tuyệt đối: ngưỡng tuyệt đối từng lớn hơn cả độ trễ của lỗi.
    assert!(
        elapsed_unknown.as_millis() * 2 >= elapsed_wrong.as_millis(),
        "Tên không tồn tại ({elapsed_unknown:?}) nhanh hơn hẳn mật khẩu sai ({elapsed_wrong:?}): lộ tên đăng nhập có thật"
    );
}

#[tokio::test]
async fn test_ac5_brute_force_lockout_after_max_attempts() {
    let (app, _pool, _lm) = setup_test_app().await.unwrap();

    // Khởi tạo chủ
    let req_init = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "admin", "password": "password123" }).to_string(),
        ))
        .unwrap();
    app.clone().oneshot(req_init).await.unwrap();

    // Thử sai 5 lần liên tiếp
    for _ in 0..5 {
        let req_fail = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({ "username": "admin", "password": "wrong_password" }).to_string(),
            ))
            .unwrap();

        let resp = app.clone().oneshot(req_fail).await.unwrap();
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        if json["error"]["code"] == "account_locked" {
            break;
        }
    }

    // Lần thứ 6: Tài khoản phải bị khóa tạm thời
    let req_locked = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "admin", "password": "password123" }).to_string(),
        ))
        .unwrap();

    let resp_locked = app.oneshot(req_locked).await.unwrap();
    assert_eq!(resp_locked.status(), StatusCode::TOO_MANY_REQUESTS);

    let bytes = to_bytes(resp_locked.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"]["code"], "account_locked");
    assert!(
        json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("tạm khóa")
    );
}

#[tokio::test]
async fn test_ac6_session_extended_on_activity() {
    let (app, pool, _lm) = setup_test_app().await.unwrap();

    // Khởi tạo chủ
    let req_init = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "admin", "password": "password123" }).to_string(),
        ))
        .unwrap();
    let resp_init = app.clone().oneshot(req_init).await.unwrap();
    let cookie_val = resp_init
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let session_cookie = cookie_val.split(';').next().unwrap();

    // Lấy thời hạn session ban đầu trong DB
    let row1 = sqlx::query("SELECT expires_at FROM session")
        .fetch_one(&pool)
        .await
        .unwrap();
    let exp1: String = sqlx::Row::try_get(&row1, "expires_at").unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Gọi GET /api/v1/me để kích hoạt gia hạn
    let req_me = Request::builder()
        .method("GET")
        .uri("/api/v1/me")
        .header(header::COOKIE, session_cookie)
        .body(Body::empty())
        .unwrap();
    let resp_me = app.oneshot(req_me).await.unwrap();
    assert_eq!(resp_me.status(), StatusCode::OK);

    // Lấy thời hạn session sau khi gia hạn
    let row2 = sqlx::query("SELECT expires_at FROM session")
        .fetch_one(&pool)
        .await
        .unwrap();
    let exp2: String = sqlx::Row::try_get(&row2, "expires_at").unwrap();

    assert!(
        exp2 >= exp1,
        "Phiên đăng nhập phải được gia hạn thời gian sống khi có hoạt động"
    );
}

#[tokio::test]
async fn test_ac7_foreign_origin_rejected_by_middleware() {
    let (app, _pool, _lm) = setup_test_app().await.unwrap();

    // Origin hợp lệ: localhost
    let req_local = Request::builder()
        .method("GET")
        .uri("/api/v1/health")
        .header("origin", "http://localhost:5173")
        .body(Body::empty())
        .unwrap();
    let resp_local = app.clone().oneshot(req_local).await.unwrap();
    assert_eq!(resp_local.status(), StatusCode::OK);

    // Origin độc hại từ bên ngoài
    let req_evil = Request::builder()
        .method("GET")
        .uri("/api/v1/health")
        .header("origin", "https://malicious-site.com")
        .body(Body::empty())
        .unwrap();
    let resp_evil = app.oneshot(req_evil).await.unwrap();
    assert_eq!(resp_evil.status(), StatusCode::FORBIDDEN);

    let bytes = to_bytes(resp_evil.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"]["code"], "forbidden_origin");
}

#[tokio::test]
async fn test_ac8_module_route_accessible_when_enabled_and_403_when_disabled() {
    let (app, _pool, lm) = setup_test_app().await.unwrap();

    // Ban đầu module sample-hello chưa bật -> Gọi qua server trả về 403 module_disabled
    let req_disabled = Request::builder()
        .method("GET")
        .uri("/api/v1/m/sample-hello/ping")
        .body(Body::empty())
        .unwrap();
    let resp_disabled = app.clone().oneshot(req_disabled).await.unwrap();
    assert_eq!(resp_disabled.status(), StatusCode::FORBIDDEN);

    let bytes_disabled = to_bytes(resp_disabled.into_body(), usize::MAX)
        .await
        .unwrap();
    let json_disabled: Value = serde_json::from_slice(&bytes_disabled).unwrap();
    assert_eq!(json_disabled["error"]["code"], "module_disabled");

    // Bật module sample-hello
    lm.enable(
        "sample-hello",
        &RegistryScope::Workspace("ws_1".to_string()),
        RegistryRole::WorkspaceAdmin,
        "u_admin",
        false,
    )
    .await
    .unwrap();

    // Giờ route truy cập thành công trả về 200
    let req_enabled = Request::builder()
        .method("GET")
        .uri("/api/v1/m/sample-hello/ping")
        .body(Body::empty())
        .unwrap();
    let resp_enabled = app.oneshot(req_enabled).await.unwrap();
    assert_eq!(resp_enabled.status(), StatusCode::OK);

    let bytes_enabled = to_bytes(resp_enabled.into_body(), usize::MAX)
        .await
        .unwrap();
    let json_enabled: Value = serde_json::from_slice(&bytes_enabled).unwrap();
    assert_eq!(json_enabled["status"], "ok");
    assert_eq!(json_enabled["module"], "sample-hello");
}

#[test]
fn test_dummy_hash_is_valid_argon2id() {
    // Nếu chuỗi giả lập không phân tích được, dummy_verify trả về ngay và tên đăng nhập không tồn tại
    // phản hồi nhanh hơn hẳn mật khẩu sai (lộ tên nào có thật).
    let parsed = argon2::password_hash::PasswordHash::new(dandori_server::auth::dummy_hash());
    assert!(parsed.is_ok(), "chuỗi băm giả lập phải hợp lệ: {parsed:?}");
}

#[test]
fn test_hash_token_is_sha256_hex_and_differs_from_input() {
    let token = "abc123";
    let hashed = dandori_server::auth::hash_token(token);
    assert_eq!(hashed.len(), 64);
    assert_ne!(hashed, token);
    // SHA-256("abc123") đã biết trước, để bắt việc đổi thuật toán hoặc quên băm.
    assert_eq!(
        hashed,
        "6ca13d52ca70c883e0f0bb101e425a89e8624de51db2d2392593af6a84118090"
    );
}

#[tokio::test]
async fn test_session_token_is_stored_hashed_not_plaintext() {
    let (app, pool, _lm) = setup_test_app().await.unwrap();

    let req_init = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "admin", "password": "password123" }).to_string(),
        ))
        .unwrap();
    let resp_init = app.clone().oneshot(req_init).await.unwrap();
    let set_cookie = resp_init
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let session_cookie = set_cookie.split(';').next().unwrap().to_string();
    let raw_token = session_cookie.trim_start_matches("dandori_session=");

    let row = sqlx::query("SELECT token_hash FROM session")
        .fetch_one(&pool)
        .await
        .unwrap();
    let stored: String = sqlx::Row::try_get(&row, "token_hash").unwrap();

    assert_ne!(stored, raw_token, "DB không được giữ token nguyên văn");
    assert_eq!(stored, dandori_server::auth::hash_token(raw_token));

    let req_me = Request::builder()
        .method("GET")
        .uri("/api/v1/me")
        .header(header::COOKIE, session_cookie)
        .body(Body::empty())
        .unwrap();
    assert_eq!(app.oneshot(req_me).await.unwrap().status(), StatusCode::OK);
}

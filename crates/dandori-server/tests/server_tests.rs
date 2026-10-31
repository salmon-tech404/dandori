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

/// Module thử có một worker chạy đến khi bị hủy, để kiểm tra tắt êm thật sự dừng worker.
#[derive(Debug)]
struct TestWorkerModule;

impl Module for TestWorkerModule {
    fn manifest(&self) -> &Manifest {
        &SAMPLE_MANIFEST
    }
    fn migrations(&self) -> &'static [Migration] {
        &SAMPLE_MIGRATIONS
    }
    fn routes(&self, _ctx: &ModuleContext) -> axum::Router {
        axum::Router::new()
    }
    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        vec![WorkerSpec::new("test-waiter", |token| {
            Box::pin(async move { token.cancelled().await })
                as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        })]
    }
    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }
}

async fn setup_test_app()
-> Result<(axum::Router, SqlitePool, Arc<FeatureLifecycleManager>), Box<dyn std::error::Error>> {
    setup_test_app_with(Arc::new(TestSampleModule)).await
}

async fn setup_test_app_with(
    module: Arc<dyn Module>,
) -> Result<(axum::Router, SqlitePool, Arc<FeatureLifecycleManager>), Box<dyn std::error::Error>> {
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

    lifecycle_manager.register_module(module).await;

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
        exp2 > exp1,
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
#[expect(
    clippy::too_many_lines,
    reason = "kịch bản kiểm thử tích hợp truy cập route module có phiên làm việc"
)]
async fn test_ac8_module_route_accessible_when_enabled_and_403_when_disabled() {
    let (app, _pool, lm) = setup_test_app().await.unwrap();

    // Thiết lập phiên đăng nhập trước khi gọi route module (T10 AC4 / T14B)
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

    // Ban đầu module sample-hello chưa bật -> Gọi qua server có phiên trả về 403 module_disabled
    let req_disabled = Request::builder()
        .method("GET")
        .uri("/api/v1/m/sample-hello/ping")
        .header(header::COOKIE, &session_cookie)
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

    // Giờ route truy cập thành công có phiên trả về 200
    let req_enabled = Request::builder()
        .method("GET")
        .uri("/api/v1/m/sample-hello/ping")
        .header(header::COOKIE, &session_cookie)
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

#[tokio::test]
async fn test_source_based_lockout_and_no_unbounded_username_rows() {
    let (app, pool, _lm) = setup_test_app().await.unwrap();

    // 4 lần thử đăng nhập thất bại đầu tiên với các tên ngẫu nhiên khác nhau từ cùng một IP nguồn: trả về 401
    let ip = "198.51.100.42";
    for i in 1..=4 {
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .header("x-forwarded-for", ip)
            .body(Body::from(
                json!({ "username": format!("unknown_user_{i}"), "password": "wrong_password" })
                    .to_string(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // Lần thứ 5 từ cùng IP nguồn này vượt ngưỡng MAX_FAILED_ATTEMPTS (5) và bị khóa (429 TOO_MANY_REQUESTS)
    let req5 = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-for", ip)
        .body(Body::from(
            json!({ "username": "another_random_user", "password": "wrong_password" }).to_string(),
        ))
        .unwrap();
    let resp5 = app.clone().oneshot(req5).await.unwrap();
    assert_eq!(
        resp5.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "IP nguồn phải bị khóa tạm thời sau 5 lần thử thất bại"
    );

    // Kiểm tra không sinh bản ghi vô hạn cho các tên đăng nhập lạ:
    // Bảng m_auth_login_attempt không được chứa khóa dạng user:unknown_user_*
    let rows = sqlx::query("SELECT key FROM m_auth_login_attempt WHERE key LIKE 'unknown_user_%' OR key LIKE 'user:unknown_user_%'")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert!(
        rows.is_empty(),
        "Không được tạo bản ghi riêng cho từng tên lạ ngẫu nhiên, chỉ khóa theo nguồn"
    );
}

#[tokio::test]
async fn test_setup_owner_concurrency_race() {
    let (app, pool, _lm) = setup_test_app().await.unwrap();

    let mut handles = Vec::new();
    for i in 0..10 {
        let app_clone = app.clone();
        handles.push(tokio::spawn(async move {
            let req = Request::builder()
                .method("POST")
                .uri("/api/v1/auth/setup-owner")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "username": format!("owner_{i}"), "password": "password123" })
                        .to_string(),
                ))
                .unwrap();
            let resp = app_clone.oneshot(req).await.unwrap();
            resp.status()
        }));
    }

    let mut success_count = 0;
    let mut conflict_count = 0;
    for h in handles {
        match h.await.unwrap() {
            StatusCode::OK => success_count += 1,
            StatusCode::CONFLICT => conflict_count += 1,
            other => panic!("Trạng thái không mong muốn: {other}"),
        }
    }

    assert_eq!(success_count, 1, "Chính xác 1 cuộc gọi tạo chủ thành công");
    assert_eq!(
        conflict_count, 9,
        "9 cuộc gọi còn lại phải trả về 409 Conflict"
    );

    let row = sqlx::query("SELECT COUNT(*) as count FROM user")
        .fetch_one(&pool)
        .await
        .unwrap();
    let user_count: i64 = sqlx::Row::try_get(&row, "count").unwrap();
    assert_eq!(
        user_count, 1,
        "Chỉ duy nhất 1 tài khoản chủ tồn tại trong DB"
    );
}

#[tokio::test]
async fn test_atomic_failed_attempt_counter() {
    let (_app, pool, _lm) = setup_test_app().await.unwrap();

    let mut handles = Vec::new();
    for _ in 0..10 {
        let pool_clone = pool.clone();
        handles.push(tokio::spawn(async move {
            dandori_server::auth::record_failed_attempt(&pool_clone, "test_atomic_key").await
        }));
    }

    for h in handles {
        h.await.unwrap().unwrap();
    }

    let count = dandori_server::auth::get_failed_count(&pool, "test_atomic_key")
        .await
        .unwrap();
    assert_eq!(
        count, 10,
        "10 lần thử ghi nhận đồng thời phải đạt chính xác bộ đếm 10"
    );
}

#[tokio::test]
async fn test_unauthenticated_request_to_module_route_returns_401() {
    let (app, _pool, lm) = setup_test_app().await.unwrap();

    // 1. Khi module đang TẮT: không có cookie -> phải trả về 401 UNAUTHORIZED (chặn trước khi kiểm tra 403 module_disabled)
    let req_disabled = Request::builder()
        .method("GET")
        .uri("/api/v1/m/sample-hello/ping")
        .body(Body::empty())
        .unwrap();
    let resp_disabled = app.clone().oneshot(req_disabled).await.unwrap();
    assert_eq!(
        resp_disabled.status(),
        StatusCode::UNAUTHORIZED,
        "Không có cookie phải trả về 401 ngay cả khi module đang tắt"
    );

    // 2. Bật module sample-hello
    lm.enable(
        "sample-hello",
        &RegistryScope::System,
        RegistryRole::SystemOwner,
        "admin",
        false,
    )
    .await
    .unwrap();

    // 3. Khi module đang BẬT: không có cookie -> vẫn phải trả về 401 UNAUTHORIZED
    let req_enabled = Request::builder()
        .method("GET")
        .uri("/api/v1/m/sample-hello/ping")
        .body(Body::empty())
        .unwrap();
    let resp_enabled = app.clone().oneshot(req_enabled).await.unwrap();
    assert_eq!(
        resp_enabled.status(),
        StatusCode::UNAUTHORIZED,
        "Không có cookie phải trả về 401 dù module đã bật"
    );
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "kiểm tra ma trận bảo mật danh sách trắng toàn bộ route của máy chủ"
)]
async fn test_every_route_requires_auth_except_whitelist() {
    let (app, _pool, _lm) = setup_test_app().await.unwrap();

    // 1. Danh sách route công khai (whitelist): chỉ duy nhất 3 route này được phép gọi không cần session
    let public_routes = [
        ("GET", "/api/v1/health"),
        ("POST", "/api/v1/auth/setup-owner"),
        ("POST", "/api/v1/auth/login"),
    ];

    for (method, uri) in public_routes {
        let req = Request::builder()
            .method(method)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({}).to_string()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_ne!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "Route công khai {method} {uri} không được yêu cầu cookie xác thực (trả về 401)"
        );
    }

    // 2. Mọi route còn lại (lõi và module) đều phải yêu cầu session và trả về 401 nếu thiếu cookie
    let protected_routes = [
        ("GET", "/api/v1/me"),
        ("POST", "/api/v1/auth/logout"),
        ("POST", "/api/v1/auth/users"),
        ("GET", "/api/v1/workspaces"),
        ("POST", "/api/v1/workspaces"),
        ("GET", "/api/v1/workspaces/ws-test-1"),
        ("PATCH", "/api/v1/workspaces/ws-test-1"),
        ("DELETE", "/api/v1/workspaces/ws-test-1"),
        ("GET", "/api/v1/workspaces/ws-test-1/members"),
        ("POST", "/api/v1/workspaces/ws-test-1/members"),
        ("DELETE", "/api/v1/workspaces/ws-test-1/members/u-test-1"),
        ("GET", "/api/v1/workspaces/ws-test-1/projects"),
        ("POST", "/api/v1/workspaces/ws-test-1/projects"),
        ("GET", "/api/v1/projects/p-test-1"),
        ("PATCH", "/api/v1/projects/p-test-1"),
        ("DELETE", "/api/v1/projects/p-test-1"),
        ("GET", "/api/v1/projects/p-test-1/tasks"),
        ("POST", "/api/v1/projects/p-test-1/tasks"),
        ("GET", "/api/v1/tasks/t-test-1"),
        ("PATCH", "/api/v1/tasks/t-test-1"),
        ("DELETE", "/api/v1/tasks/t-test-1"),
        ("POST", "/api/v1/tasks/t-test-1/status"),
        ("GET", "/api/v1/features/effective"),
        ("GET", "/api/v1/features/modules"),
        ("GET", "/api/v1/features/packs"),
        ("POST", "/api/v1/features/plan-enable"),
        ("POST", "/api/v1/features/apply"),
        ("POST", "/api/v1/features/plan-disable"),
        ("GET", "/api/v1/features/audit"),
        ("GET", "/api/v1/m/sample-hello/ping"),
    ];

    for (method, uri) in protected_routes {
        let req = Request::builder()
            .method(method)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({}).to_string()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "Route bảo vệ {method} {uri} phải trả về 401 khi không có cookie phiên"
        );
    }
}

#[tokio::test]
async fn test_real_server_tcp_bind_port_zero_and_health_check() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = ServerConfig::new("127.0.0.1", 0, temp_dir.path().to_path_buf()).unwrap();
    let cancel_token = tokio_util::sync::CancellationToken::new();

    let (addr, handle, _lm) = dandori_server::start_background_server(config, cancel_token.clone())
        .await
        .unwrap();

    let url = format!("http://{addr}/api/v1/health");
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["status"], "ok");

    cancel_token.cancel();
    handle.await.unwrap().unwrap();
}

#[tokio::test]
async fn test_real_server_graceful_shutdown_and_worker_cleanup() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = ServerConfig::new("127.0.0.1", 0, temp_dir.path().to_path_buf()).unwrap();
    let cancel_token = tokio_util::sync::CancellationToken::new();

    let (addr, handle, lm) = dandori_server::start_background_server(config, cancel_token.clone())
        .await
        .unwrap();

    // Xác nhận máy chủ đang phục vụ trên TCP
    let url = format!("http://{addr}/api/v1/health");
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    // Kích hoạt hủy token tắt êm
    cancel_token.cancel();

    // Tác vụ máy chủ phải kết thúc trong thời hạn (deadline 5s)
    let shutdown_res = tokio::time::timeout(std::time::Duration::from_secs(5), handle).await;
    assert!(shutdown_res.is_ok(), "Máy chủ phải tắt êm trong thời hạn");
    shutdown_res.unwrap().unwrap().unwrap();

    // Số tác vụ worker nền đang hoạt động phải bằng 0 sau khi tắt
    let active_workers = lm.active_task_count().await;
    assert_eq!(
        active_workers, 0,
        "Mọi worker nền phải được dừng khi tắt máy chủ"
    );
}

#[tokio::test]
async fn test_graceful_shutdown_stops_running_module_workers() {
    let (app, _pool, lm) = setup_test_app_with(Arc::new(TestWorkerModule))
        .await
        .unwrap();
    lm.enable(
        "sample-hello",
        &RegistryScope::Workspace("ws_1".to_string()),
        RegistryRole::WorkspaceAdmin,
        "u_admin",
        false,
    )
    .await
    .unwrap();
    assert!(
        lm.active_task_count().await >= 1,
        "worker phải đang chạy trước khi tắt máy chủ"
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let token = tokio_util::sync::CancellationToken::new();
    let handle = tokio::spawn(dandori_server::serve_app(
        listener,
        app,
        Arc::clone(&lm),
        Some(token.clone()),
    ));

    token.cancel();
    let result = tokio::time::timeout(std::time::Duration::from_secs(5), handle).await;
    result.unwrap().unwrap().unwrap();

    assert_eq!(
        lm.active_task_count().await,
        0,
        "tắt êm phải dừng mọi worker đang chạy"
    );
}

#[tokio::test]
async fn test_shutdown_deadline_closes_hung_connection() {
    use tokio::io::AsyncWriteExt;

    let (app, _pool, lm) = setup_test_app().await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let token = tokio_util::sync::CancellationToken::new();
    let handle = tokio::spawn(dandori_server::serve_app_with_deadline(
        listener,
        app,
        Arc::clone(&lm),
        Some(token.clone()),
        std::time::Duration::from_millis(300),
    ));

    // Kết nối treo: gửi dở dòng đầu của yêu cầu rồi im lặng, máy chủ coi là yêu cầu đang chạy.
    let mut hung = tokio::net::TcpStream::connect(addr).await.unwrap();
    hung.write_all(
        b"GET /api/v1/health HTTP/1.1
Host: x
",
    )
    .await
    .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    token.cancel();
    let result = tokio::time::timeout(std::time::Duration::from_secs(3), handle).await;
    assert!(
        result.is_ok(),
        "kết nối treo không được giữ máy chủ sống quá thời hạn tắt"
    );
    drop(hung);
}

#[tokio::test]
async fn test_failed_attempts_restart_after_lock_expires() {
    use dandori_server::auth::{check_lockout, get_failed_count, record_failed_attempt};

    let (_app, pool, _lm) = setup_test_app().await.unwrap();
    for _ in 0..5 {
        record_failed_attempt(&pool, "user:bob").await.unwrap();
    }
    assert!(check_lockout(&pool, "user:bob").await.unwrap().is_some());

    // Giả lập hết hạn khóa bằng cách lùi thời điểm mở khóa về quá khứ.
    let past = (chrono::Utc::now() - chrono::Duration::minutes(1)).to_rfc3339();
    sqlx::query("UPDATE m_auth_login_attempt SET locked_until = ? WHERE key = ?")
        .bind(past)
        .bind("user:bob")
        .execute(&pool)
        .await
        .unwrap();
    assert!(check_lockout(&pool, "user:bob").await.unwrap().is_none());

    // Sau khi hết khóa, một lần gõ sai mới chỉ tính là lần thứ nhất, không khóa lại ngay.
    let relock = record_failed_attempt(&pool, "user:bob").await.unwrap();
    assert!(
        relock.is_none(),
        "một lần sai sau khi hết khóa không được khóa lại"
    );
    assert_eq!(get_failed_count(&pool, "user:bob").await.unwrap(), 1);
}

#[tokio::test]
async fn test_real_server_ignores_forwarded_for_header_for_lockout() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = ServerConfig::new("127.0.0.1", 0, temp_dir.path().to_path_buf()).unwrap();
    let token = tokio_util::sync::CancellationToken::new();
    let (addr, handle, _lm) = dandori_server::start_background_server(config, token.clone())
        .await
        .unwrap();

    // Mỗi lần giả một IP khác qua X-Forwarded-For: máy chủ thật phải dùng địa chỉ kết nối TCP,
    // nên 5 lần sai vẫn dồn vào cùng một nguồn và bị khóa.
    let client = reqwest::Client::new();
    let url = format!("http://{addr}/api/v1/auth/login");
    let mut last_status = 0;
    for i in 0..5 {
        let resp = client
            .post(&url)
            .header("content-type", "application/json")
            .header("x-forwarded-for", format!("203.0.113.{i}"))
            .body(json!({ "username": "ghost", "password": "wrongpass" }).to_string())
            .send()
            .await
            .unwrap();
        last_status = resp.status().as_u16();
    }
    assert_eq!(
        last_status, 429,
        "không được lách khóa theo nguồn bằng header X-Forwarded-For"
    );

    token.cancel();
    handle.await.unwrap().unwrap();
}

#[test]
fn test_config_from_env_defaults_and_validation() {
    let none = |_: &str| None;
    let defaults = ServerConfig::from_lookup(none).unwrap();
    assert_eq!(defaults.host, "127.0.0.1");
    assert_eq!(defaults.port, 3000);

    let custom = ServerConfig::from_lookup(|k| match k {
        "DANDORI_PORT" => Some("4100".to_string()),
        _ => None,
    })
    .unwrap();
    assert_eq!(custom.port, 4100);

    let open_host = ServerConfig::from_lookup(|k| match k {
        "DANDORI_HOST" => Some("0.0.0.0".to_string()),
        _ => None,
    });
    assert!(open_host.is_err(), "0.0.0.0 phải bị từ chối");

    let bad_port = ServerConfig::from_lookup(|k| match k {
        "DANDORI_PORT" => Some("abc".to_string()),
        _ => None,
    });
    assert!(bad_port.is_err(), "cổng không phải số phải bị từ chối");
}

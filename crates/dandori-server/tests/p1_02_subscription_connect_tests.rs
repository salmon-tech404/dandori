//! Các kiểm thử tích hợp cho Task P1-02 (Module subscription-connect & Quản lý tài khoản CLI).

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P1-02 dùng unwrap và có fixture dài"
)]

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use dandori_events::EventBus;
use dandori_registry::FeatureRegistry;
use dandori_registry::lifecycle::FeatureLifecycleManager;
use dandori_server::app::create_app;
use dandori_server::config::ServerConfig;
use dandori_server::state::AppState;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tempfile::tempdir;
use tower::ServiceExt;

struct TestContext {
    app: axum::Router,
    pool: SqlitePool,
    lifecycle_manager: Arc<FeatureLifecycleManager>,
    cookie: String,
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
    for m in modules {
        lifecycle_manager.register_module(m).await;
    }

    let temp_dir = tempdir().unwrap();
    let config = ServerConfig {
        data_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let state = AppState::new(
        pool.clone(),
        Arc::clone(&event_bus),
        Arc::clone(&lifecycle_manager),
        config,
    );

    let app = create_app(state);

    // Tạo owner và đăng nhập lấy phiên
    let req_init = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "admin", "password": "password123" }).to_string(),
        ))
        .unwrap();
    let resp_init = app.clone().oneshot(req_init).await.unwrap();
    assert_eq!(resp_init.status(), StatusCode::OK);

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
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let cookie = cookie_val.split(';').next().unwrap().to_string();

    TestContext {
        app,
        pool,
        lifecycle_manager,
        cookie,
        temp_dir,
    }
}

#[tokio::test]
async fn test_auth_enforcement_and_module_gating() {
    let ctx = setup_test_context().await;

    // 1. Không có cookie phiên -> 401 Unauthorized
    let req_unauth = Request::builder()
        .method("GET")
        .uri("/api/v1/m/subscription-connect/accounts")
        .body(Body::empty())
        .unwrap();
    let resp_unauth = ctx.app.clone().oneshot(req_unauth).await.unwrap();
    assert_eq!(resp_unauth.status(), StatusCode::UNAUTHORIZED);

    // 2. Có cookie phiên nhưng module chưa bật -> 403 Forbidden (module_disabled)
    let req_disabled = Request::builder()
        .method("GET")
        .uri("/api/v1/m/subscription-connect/accounts")
        .header(header::COOKIE, &ctx.cookie)
        .body(Body::empty())
        .unwrap();
    let resp_disabled = ctx.app.clone().oneshot(req_disabled).await.unwrap();
    assert_eq!(resp_disabled.status(), StatusCode::FORBIDDEN);

    let body_bytes = axum::body::to_bytes(resp_disabled.into_body(), usize::MAX)
        .await
        .unwrap();
    let json_err: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json_err["error"]["code"], "module_disabled");
}

#[tokio::test]
async fn test_crud_accounts_and_isolated_home() {
    let ctx = setup_test_context().await;

    // Kích hoạt module subscription-connect
    let scope = dandori_registry::RegistryScope::System;
    ctx.lifecycle_manager
        .enable(
            "subscription-connect",
            &scope,
            dandori_registry::RegistryRole::SystemOwner,
            "admin",
            true,
        )
        .await
        .unwrap();

    // 1. Thêm tài khoản Claude
    let req_create = Request::builder()
        .method("POST")
        .uri("/api/v1/m/subscription-connect/accounts")
        .header(header::COOKIE, &ctx.cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "claude",
                "label": "Tài khoản Claude Dev",
                "priority": 10
            })
            .to_string(),
        ))
        .unwrap();

    let resp_create = ctx.app.clone().oneshot(req_create).await.unwrap();
    assert_eq!(resp_create.status(), StatusCode::CREATED);

    let create_body = axum::body::to_bytes(resp_create.into_body(), usize::MAX)
        .await
        .unwrap();
    let acc: Value = serde_json::from_slice(&create_body).unwrap();
    let account_id = acc["id"].as_str().unwrap().to_string();
    let home_dir = acc["home_dir"].as_str().unwrap().to_string();

    assert_eq!(acc["provider"], "claude");
    assert_eq!(acc["label"], "Tài khoản Claude Dev");
    assert_eq!(acc["priority"], 10);
    assert_eq!(acc["auth_type"], "cli");

    // Khẳng định thư mục HOME cô lập thực sự được tạo trên ổ đĩa
    let home_path = std::path::PathBuf::from(&home_dir);
    assert!(
        home_path.is_dir(),
        "Thư mục HOME cô lập phải tồn tại trên đĩa: {}",
        home_dir
    );

    // 2. Lấy danh sách tài khoản
    let req_list = Request::builder()
        .method("GET")
        .uri("/api/v1/m/subscription-connect/accounts")
        .header(header::COOKIE, &ctx.cookie)
        .body(Body::empty())
        .unwrap();

    let resp_list = ctx.app.clone().oneshot(req_list).await.unwrap();
    assert_eq!(resp_list.status(), StatusCode::OK);

    let list_body = axum::body::to_bytes(resp_list.into_body(), usize::MAX)
        .await
        .unwrap();
    let list: Vec<Value> = serde_json::from_slice(&list_body).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], account_id);

    // 3. Xem hướng dẫn đăng nhập
    let req_instructions = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/subscription-connect/accounts/{}/login-instructions",
            account_id
        ))
        .header(header::COOKIE, &ctx.cookie)
        .body(Body::empty())
        .unwrap();

    let resp_instructions = ctx.app.clone().oneshot(req_instructions).await.unwrap();
    assert_eq!(resp_instructions.status(), StatusCode::OK);

    let inst_body = axum::body::to_bytes(resp_instructions.into_body(), usize::MAX)
        .await
        .unwrap();
    let inst: Value = serde_json::from_slice(&inst_body).unwrap();
    let cmd = inst["command"].as_str().unwrap();
    assert!(
        cmd.contains(&home_dir),
        "Lệnh đăng nhập phải chứa đường dẫn HOME cô lập"
    );
    assert!(
        cmd.contains("claude auth login"),
        "Lệnh đăng nhập phải chứa lệnh CLI chính thức của Claude"
    );

    // 4. Kiểm tra lại trạng thái tài khoản
    let req_check = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/subscription-connect/accounts/{}/check",
            account_id
        ))
        .header(header::COOKIE, &ctx.cookie)
        .body(Body::empty())
        .unwrap();

    let resp_check = ctx.app.clone().oneshot(req_check).await.unwrap();
    assert_eq!(resp_check.status(), StatusCode::OK);

    // 5. Xóa tài khoản và dọn dẹp HOME
    let req_delete = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/api/v1/m/subscription-connect/accounts/{}",
            account_id
        ))
        .header(header::COOKIE, &ctx.cookie)
        .body(Body::empty())
        .unwrap();

    let resp_delete = ctx.app.clone().oneshot(req_delete).await.unwrap();
    assert_eq!(resp_delete.status(), StatusCode::OK);

    // Khẳng định thư mục HOME đã bị xóa sạch
    assert!(
        !home_path.exists(),
        "Thư mục HOME cô lập phải được dọn dẹp sau khi xóa tài khoản"
    );

    // Khẳng định tài khoản đã bị xóa khỏi cơ sở dữ liệu
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agent_account WHERE id = ?")
        .bind(&account_id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    assert_eq!(
        count, 0,
        "Bản ghi tài khoản phải bị xóa khỏi bảng agent_account"
    );
}

#[tokio::test]
async fn test_regular_user_cannot_manage_agent_accounts() {
    let ctx = setup_test_context().await;
    ctx.lifecycle_manager
        .enable(
            "subscription-connect",
            &dandori_registry::RegistryScope::System,
            dandori_registry::RegistryRole::SystemOwner,
            "admin",
            true,
        )
        .await
        .unwrap();

    let create_user = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/users")
        .header(header::COOKIE, &ctx.cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "mallory", "password": "password123", "system_role": "user" })
                .to_string(),
        ))
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(create_user).await.unwrap().status(),
        StatusCode::OK
    );
    let login = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "mallory", "password": "password123" }).to_string(),
        ))
        .unwrap();
    let mallory = ctx
        .app
        .clone()
        .oneshot(login)
        .await
        .unwrap()
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Tài khoản agent dùng chung toàn hệ thống: người dùng thường không được tạo hay xóa.
    let create = Request::builder()
        .method("POST")
        .uri("/api/v1/m/subscription-connect/accounts")
        .header(header::COOKIE, &mallory)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "provider": "claude", "label": "của mallory" }).to_string(),
        ))
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(create).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );

    // Chủ vẫn tạo được; người dùng thường không xóa được tài khoản đó.
    let owner_create = Request::builder()
        .method("POST")
        .uri("/api/v1/m/subscription-connect/accounts")
        .header(header::COOKIE, &ctx.cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "provider": "claude", "label": "của chủ" }).to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(owner_create).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let acc_id = serde_json::from_slice::<serde_json::Value>(&body).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    for (method, path) in [
        (
            "DELETE",
            format!("/api/v1/m/subscription-connect/accounts/{acc_id}"),
        ),
        (
            "POST",
            format!("/api/v1/m/subscription-connect/accounts/{acc_id}/check"),
        ),
        (
            "GET",
            format!("/api/v1/m/subscription-connect/accounts/{acc_id}/login-instructions"),
        ),
    ] {
        let req = Request::builder()
            .method(method)
            .uri(&path)
            .header(header::COOKIE, &mallory)
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            ctx.app.clone().oneshot(req).await.unwrap().status(),
            StatusCode::FORBIDDEN,
            "{method} {path}"
        );
    }
}

//! Các kiểm thử tích hợp cho Task P2-08 (Module multi-account & Quản lý nhiều tài khoản cùng nhà cung cấp).

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P2-08 dùng unwrap và có fixture kiểm tra nhiều tiêu chí"
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
    #[expect(dead_code, reason = "Giữ lifecycle manager trong bộ nhớ")]
    lifecycle_manager: Arc<FeatureLifecycleManager>,
    owner_cookie: String,
    non_admin_cookie: String,
    #[expect(dead_code, reason = "Giữ thư mục tạm cho đến khi kết thúc test")]
    temp_dir: tempfile::TempDir,
}

async fn login_new_user(app: &axum::Router, owner_cookie: &str, username: &str) -> String {
    let create = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/users")
        .header(header::COOKIE, owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": username, "password": "password123", "system_role": "user" })
                .to_string(),
        ))
        .unwrap();
    assert_eq!(
        app.clone().oneshot(create).await.unwrap().status(),
        StatusCode::OK
    );

    let login = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": username, "password": "password123" }).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(login).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    resp.headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
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
    let state = AppState::new(
        pool.clone(),
        Arc::clone(&event_bus),
        Arc::clone(&lifecycle_manager),
        config,
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

    // Bật module worktree, subscription-connect và multi-account ở cấp hệ thống
    for mod_id in ["worktree", "subscription-connect", "multi-account"] {
        let enable_req = Request::builder()
            .method("POST")
            .uri("/api/v1/features/apply")
            .header(header::COOKIE, &owner_cookie)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "module_id": mod_id,
                    "scope": "system",
                    "target_id": "system",
                    "enable": true
                })
                .to_string(),
            ))
            .unwrap();
        let enable_resp = app.clone().oneshot(enable_req).await.unwrap();
        let status = enable_resp.status();
        let bytes = axum::body::to_bytes(enable_resp.into_body(), 1024 * 64)
            .await
            .unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert_eq!(status, StatusCode::OK, "Failed to enable {mod_id}: {text}");
    }

    // Đăng ký tài khoản Non-Admin (Member) để kiểm tra phân quyền (BOLA)
    let non_admin_cookie = login_new_user(&app, &owner_cookie, "memberuser").await;

    TestContext {
        app,
        pool,
        lifecycle_manager,
        owner_cookie,
        non_admin_cookie,
        temp_dir,
    }
}

/// Tiêu chí 1: Hai account cùng nhà; chọn theo ưu tiên; tắt account ưu tiên cao -> chọn account kế.
#[tokio::test]
async fn test_ac1_multi_account_priority_and_disable_selection() {
    let ctx = setup_test_context().await;

    // Tạo 2 tài khoản cùng nhà "fake": account 1 priority 10, account 2 priority 20
    let req1 = Request::builder()
        .uri("/api/v1/m/subscription-connect/accounts")
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "fake",
                "label": "Fake Account High Priority",
                "priority": 10
            })
            .to_string(),
        ))
        .unwrap();
    let resp1 = ctx.app.clone().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::CREATED);
    let body1: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp1.into_body(), 1024 * 64)
            .await
            .unwrap(),
    )
    .unwrap();
    let acc1_id = body1["id"].as_str().unwrap().to_string();

    let req2 = Request::builder()
        .uri("/api/v1/m/subscription-connect/accounts")
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "fake",
                "label": "Fake Account Lower Priority",
                "priority": 20
            })
            .to_string(),
        ))
        .unwrap();
    let resp2 = ctx.app.clone().oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::CREATED);
    let body2: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp2.into_body(), 1024 * 64)
            .await
            .unwrap(),
    )
    .unwrap();
    let acc2_id = body2["id"].as_str().unwrap().to_string();

    // Cập nhật trạng thái cả hai tài khoản sang "ready" để bộ chọn lõi có thể chọn
    sqlx::query("UPDATE agent_account SET status = 'ready' WHERE id IN (?, ?)")
        .bind(&acc1_id)
        .bind(&acc2_id)
        .execute(&ctx.pool)
        .await
        .unwrap();

    // Kiểm tra bộ chọn lõi: ưu tiên 10 nhỏ hơn 20 nên acc1 phải được chọn
    let repo = dandori_db::repo::AgentAccountRepo::new(&ctx.pool);
    let picked1 = repo
        .find_available_account(Some("fake"), &[])
        .await
        .unwrap()
        .expect("Phải tìm thấy tài khoản khả dụng");
    assert_eq!(picked1.id.to_string(), acc1_id);
    assert_eq!(picked1.priority, 10);

    // Tắt account ưu tiên cao (acc1) qua endpoint PATCH của mod-multi-account
    let patch_req = Request::builder()
        .uri(format!("/api/v1/m/multi-account/accounts/{acc1_id}"))
        .method("PATCH")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "enabled": false }).to_string()))
        .unwrap();
    let patch_resp = ctx.app.clone().oneshot(patch_req).await.unwrap();
    assert_eq!(patch_resp.status(), StatusCode::OK);

    // Kiểm tra bộ chọn lõi sau khi tắt acc1: acc2 phải được chọn thay thế
    let picked2 = repo
        .find_available_account(Some("fake"), &[])
        .await
        .unwrap()
        .expect("Phải tìm thấy tài khoản acc2");
    assert_eq!(picked2.id.to_string(), acc2_id);
    assert_eq!(picked2.priority, 20);

    // Đổi ưu tiên acc2 lên 50, bật lại acc1 với ưu tiên 10
    let patch_req2 = Request::builder()
        .uri(format!("/api/v1/m/multi-account/accounts/{acc2_id}"))
        .method("PATCH")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "priority": 50 }).to_string()))
        .unwrap();
    let patch_resp2 = ctx.app.clone().oneshot(patch_req2).await.unwrap();
    assert_eq!(patch_resp2.status(), StatusCode::OK);

    let patch_req1_reopen = Request::builder()
        .uri(format!("/api/v1/m/multi-account/accounts/{acc1_id}"))
        .method("PATCH")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "enabled": true }).to_string()))
        .unwrap();
    let patch_resp1 = ctx.app.clone().oneshot(patch_req1_reopen).await.unwrap();
    assert_eq!(patch_resp1.status(), StatusCode::OK);

    // acc1 lại được chọn vì 10 < 50
    let picked3 = repo
        .find_available_account(Some("fake"), &[])
        .await
        .unwrap()
        .expect("Phải tìm thấy tài khoản acc1");
    assert_eq!(picked3.id.to_string(), acc1_id);
}

/// Tiêu chí 2: Nhà cung cấp không hỗ trợ cô lập HOME không cho thêm account thứ hai.
#[tokio::test]
async fn test_ac2_reject_second_account_if_home_isolation_not_supported() {
    let ctx = setup_test_context().await;

    // "codex" không hỗ trợ cô lập HOME per docs/agents/*.md
    // Thêm account thứ nhất cho "codex": thành công
    let req1 = Request::builder()
        .uri("/api/v1/m/subscription-connect/accounts")
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "codex",
                "label": "Codex Account 1",
                "priority": 100
            })
            .to_string(),
        ))
        .unwrap();
    let resp1 = ctx.app.clone().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Kiểm tra validate-addition từ mod-multi-account báo allowed = false
    let validate_req = Request::builder()
        .uri("/api/v1/m/multi-account/accounts/validate-addition")
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "provider": "codex" }).to_string()))
        .unwrap();
    let validate_resp = ctx.app.clone().oneshot(validate_req).await.unwrap();
    assert_eq!(validate_resp.status(), StatusCode::OK);
    let val_body: Value = serde_json::from_slice(
        &axum::body::to_bytes(validate_resp.into_body(), 1024 * 64)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(val_body["allowed"], false);
    assert!(
        val_body["reason"]
            .as_str()
            .unwrap()
            .contains("không hỗ trợ cô lập thư mục HOME")
    );

    // Cố gắng thêm account thứ hai cho "codex": PHẢI BỊ CHẶN (400 Bad Request)
    let req2 = Request::builder()
        .uri("/api/v1/m/subscription-connect/accounts")
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "codex",
                "label": "Codex Account 2",
                "priority": 100
            })
            .to_string(),
        ))
        .unwrap();
    let resp2 = ctx.app.clone().oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::BAD_REQUEST);
    let err_body: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp2.into_body(), 1024 * 64)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(
        err_body["error"]
            .as_str()
            .unwrap()
            .contains("không hỗ trợ cô lập thư mục HOME")
    );
}

/// Tiêu chí 3: Mỗi account dùng HOME riêng (tiến trình con nhận đúng đường dẫn khác nhau).
#[tokio::test]
async fn test_ac3_each_account_uses_distinct_isolated_home() {
    let ctx = setup_test_context().await;

    // Tạo 2 tài khoản cho "claude"
    let req1 = Request::builder()
        .uri("/api/v1/m/subscription-connect/accounts")
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "claude",
                "label": "Claude Account Alpha",
                "priority": 10
            })
            .to_string(),
        ))
        .unwrap();
    let resp1 = ctx.app.clone().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::CREATED);
    let b1: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp1.into_body(), 1024 * 64)
            .await
            .unwrap(),
    )
    .unwrap();
    let home1 = b1["home_dir"].as_str().unwrap().to_string();

    let req2 = Request::builder()
        .uri("/api/v1/m/subscription-connect/accounts")
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "claude",
                "label": "Claude Account Beta",
                "priority": 20
            })
            .to_string(),
        ))
        .unwrap();
    let resp2 = ctx.app.clone().oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::CREATED);
    let b2: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp2.into_body(), 1024 * 64)
            .await
            .unwrap(),
    )
    .unwrap();
    let home2 = b2["home_dir"].as_str().unwrap().to_string();

    // Kiểm tra hai đường dẫn HOME là hoàn toàn khác biệt trên hệ thống tập tin
    assert_ne!(home1, home2);
    assert!(std::path::Path::new(&home1).exists());
    assert!(std::path::Path::new(&home2).exists());
}

/// Tiêu chí 4: Hiển thị danh sách nhóm theo nhà cung cấp và thông tin task đang chạy.
#[tokio::test]
async fn test_ac4_grouped_accounts_and_running_task_association() {
    let ctx = setup_test_context().await;

    // Tạo tài khoản cho fake
    let req = Request::builder()
        .uri("/api/v1/m/subscription-connect/accounts")
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "fake",
                "label": "Worker Account 1",
                "priority": 10
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), 1024 * 64)
            .await
            .unwrap(),
    )
    .unwrap();
    let acc_id = body["id"].as_str().unwrap();

    // Tạo workspace, project, task, và run đang running với tài khoản này
    let ws_id = "ws-test-multi";
    let proj_id = "proj-test-multi";
    let task_id = "task-test-multi";
    let run_id = "run-test-multi";
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS', ?)")
        .bind(ws_id)
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'PROJ', '/repo', 'main', ?)")
        .bind(proj_id)
        .bind(ws_id)
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES (?, ?, 'Task Active 1', 'Desc', 'running', 'lead', 'fake', ?, ?)")
        .bind(task_id)
        .bind(proj_id)
        .bind(&now)
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO run (id, task_id, agent, account, status, started_at) VALUES (?, ?, 'fake', 'Worker Account 1', 'running', ?)")
        .bind(run_id)
        .bind(task_id)
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();

    // Lấy danh sách tài khoản nhóm theo nhà cung cấp từ mod-multi-account
    let list_req = Request::builder()
        .uri("/api/v1/m/multi-account/accounts")
        .method("GET")
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let list_resp = ctx.app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: Value = serde_json::from_slice(
        &axum::body::to_bytes(list_resp.into_body(), 1024 * 64)
            .await
            .unwrap(),
    )
    .unwrap();

    let groups = list_body
        .as_array()
        .expect("Phải trả về mảng các nhóm nhà cung cấp");
    let fake_group = groups
        .iter()
        .find(|g| g["provider"].as_str() == Some("fake"))
        .expect("Phải có nhóm nhà cung cấp fake");
    assert_eq!(fake_group["supports_home_isolation"], true);

    let accounts = fake_group["accounts"].as_array().unwrap();
    let target_acc = accounts
        .iter()
        .find(|a| a["id"].as_str() == Some(acc_id))
        .expect("Phải có tài khoản đã tạo");
    let running_task = &target_acc["running_task"];
    assert_eq!(running_task["task_id"], task_id);
    assert_eq!(running_task["task_title"], "Task Active 1");
    assert_eq!(running_task["run_id"], run_id);
}

/// Tiêu chí 5: Phân quyền quản trị (BOLA check) - người dùng thường không thể cập nhật tài khoản.
#[tokio::test]
async fn test_ac5_authorization_bola_check() {
    let ctx = setup_test_context().await;

    // Tạo tài khoản bởi admin
    let req = Request::builder()
        .uri("/api/v1/m/subscription-connect/accounts")
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "fake",
                "label": "Test Auth Account",
                "priority": 50
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), 1024 * 64)
            .await
            .unwrap(),
    )
    .unwrap();
    let acc_id = body["id"].as_str().unwrap();

    // Thử cập nhật với tài khoản non-admin -> PHẢI BỊ CHẶN (403 Forbidden)
    let patch_req = Request::builder()
        .uri(format!("/api/v1/m/multi-account/accounts/{acc_id}"))
        .method("PATCH")
        .header(header::COOKIE, &ctx.non_admin_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "enabled": false }).to_string()))
        .unwrap();
    let patch_resp = ctx.app.clone().oneshot(patch_req).await.unwrap();
    assert_eq!(patch_resp.status(), StatusCode::FORBIDDEN);
}

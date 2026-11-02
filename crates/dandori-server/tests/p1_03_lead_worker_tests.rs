//! Các bài kiểm thử tích hợp cho Task P1-03: Module `lead-worker`.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P1-03 dùng unwrap và có fixture dài"
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
    owner_cookie: String,
    temp_dir: tempfile::TempDir,
}

impl TestContext {
    async fn send_request(
        &self,
        method: &str,
        uri: &str,
        cookie: &str,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut builder = Request::builder().method(method).uri(uri);

        if !cookie.is_empty() {
            builder = builder.header(header::COOKIE, cookie);
        }

        let req_body = if let Some(b) = body {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
            Body::from(b.to_string())
        } else {
            Body::empty()
        };

        let req = builder.body(req_body).unwrap();
        let resp = self.app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap_or(Value::Null);
        (status, body_json)
    }

    async fn set_account_ready(&self, provider: &str) {
        sqlx::query("UPDATE agent_account SET status = 'ready' WHERE provider = ?")
            .bind(provider)
            .execute(&self.pool)
            .await
            .unwrap();
    }

    async fn login_user(&self, username: &str, pass: &str) -> String {
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({ "username": username, "password": pass }).to_string(),
            ))
            .unwrap();
        let resp = self.app.clone().oneshot(req).await.unwrap();
        resp.headers()
            .get(header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
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

    // Đăng ký toàn bộ module tự động và chạy migration
    let modules = dandori_modules::registered_modules();
    for m in &modules {
        for mig in m.migrations() {
            sqlx::query(mig.sql).execute(&pool).await.unwrap();
        }
    }
    for m in modules {
        lifecycle_manager.register_module(m).await;
    }

    let temp_dir = tempdir().unwrap();
    // Tạo thư mục .git giả lập repo hợp lệ
    std::fs::create_dir_all(temp_dir.path().join(".git")).unwrap();

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

    let owner_cookie = resp_login
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Kích hoạt module subscription-connect và lead-worker cho test
    let scope = dandori_registry::RegistryScope::System;
    lifecycle_manager
        .enable(
            "subscription-connect",
            &scope,
            dandori_registry::RegistryRole::SystemOwner,
            "admin",
            true,
        )
        .await
        .unwrap();
    lifecycle_manager
        .enable(
            "lead-worker",
            &scope,
            dandori_registry::RegistryRole::SystemOwner,
            "admin",
            true,
        )
        .await
        .unwrap();

    TestContext {
        app,
        pool,
        owner_cookie,
        temp_dir,
    }
}

#[tokio::test]
async fn test_p1_03_lead_worker_ac1_defaults_and_override() {
    let ctx = setup_test_context().await;
    let repo_path = ctx.temp_dir.path().to_str().unwrap();

    // Tạo workspace và project
    let (status, ws_body) = ctx
        .send_request(
            "POST",
            "/api/v1/workspaces",
            &ctx.owner_cookie,
            Some(json!({ "name": "WS P1-03 Defaults" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let ws_id = ws_body["workspace"]["id"].as_str().unwrap();

    let (status, proj_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/projects"),
            &ctx.owner_cookie,
            Some(json!({
                "name": "Proj P1-03 Defaults",
                "repo_path": repo_path,
                "default_branch": "main"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let proj_id = proj_body["project"]["id"].as_str().unwrap();

    // 1. Kiểm tra defaults ban đầu của project
    let (status, def_body) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/m/lead-worker/projects/{proj_id}/defaults"),
            &ctx.owner_cookie,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let defaults = def_body["defaults"].as_array().unwrap();
    assert_eq!(
        defaults.len(),
        4,
        "Phải có đủ 4 vai trò mặc định (pm, worker, test, doc)"
    );

    // Thêm các tài khoản ready cho claude và gemini
    let (status, _) = ctx
        .send_request(
            "POST",
            "/api/v1/m/subscription-connect/accounts",
            &ctx.owner_cookie,
            Some(json!({
                "provider": "gemini",
                "label": "Gemini Pro Account",
                "priority": 10
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, _) = ctx
        .send_request(
            "POST",
            "/api/v1/m/subscription-connect/accounts",
            &ctx.owner_cookie,
            Some(json!({
                "provider": "claude",
                "label": "Claude Sonnet Account",
                "priority": 20
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    // Đánh dấu cả hai tài khoản ở trạng thái ready
    ctx.set_account_ready("gemini").await;
    ctx.set_account_ready("claude").await;

    // Tạo Task 1 không chọn agent (để trống)
    let (status, t1_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/projects/{proj_id}/tasks"),
            &ctx.owner_cookie,
            Some(json!({
                "title": "Task 1 Mặc định",
                "description": "Không ghi đè agent"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let t1_id = t1_body["task"]["id"].as_str().unwrap();

    // Lấy thông tin assignment của Task 1:
    // Mặc định vai Worker: Gemini > Claude > Codex -> chọn Gemini
    let (status, assign1_body) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/m/lead-worker/tasks/{t1_id}/assignment"),
            &ctx.owner_cookie,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(assign1_body["effective_worker_agent"], "gemini");
    assert_eq!(assign1_body["is_worker_overridden"], false);

    // Tạo Task 2 có ghi đè worker_agent = "claude"
    let (status, t2_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/projects/{proj_id}/tasks"),
            &ctx.owner_cookie,
            Some(json!({
                "title": "Task 2 Ghi đè",
                "description": "Ghi đè sang Claude"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let t2_id = t2_body["task"]["id"].as_str().unwrap();

    let (status, patch_body) = ctx
        .send_request(
            "PATCH",
            &format!("/api/v1/tasks/{t2_id}"),
            &ctx.owner_cookie,
            Some(json!({
                "title": "Task 2 Ghi đè",
                "description": "Ghi đè sang Claude",
                "worker_agent": "claude"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(patch_body["task"]["worker_agent"], "claude");

    // Lấy thông tin assignment của Task 2: ghi đè Claude được ưu tiên hơn mặc định Gemini
    let (status, assign2_body) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/m/lead-worker/tasks/{t2_id}/assignment"),
            &ctx.owner_cookie,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(assign2_body["effective_worker_agent"], "claude");
    assert_eq!(assign2_body["is_worker_overridden"], true);

    // Cập nhật cấu hình mặc định của project qua PUT
    let (status, _) = ctx
        .send_request(
            "PUT",
            &format!("/api/v1/m/lead-worker/projects/{proj_id}/defaults"),
            &ctx.owner_cookie,
            Some(json!({
                "defaults": [
                    { "role": "pm", "provider_order": ["gemini", "claude"] },
                    { "role": "worker", "provider_order": ["claude", "gemini"] },
                    { "role": "test", "provider_order": ["codex", "claude"] },
                    { "role": "doc", "provider_order": ["claude", "gemini"] }
                ]
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Bây giờ Task 1 (không ghi đè) sẽ nhận Claude làm effective_worker_agent theo cấu hình mới
    let (status, assign1_updated) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/m/lead-worker/tasks/{t1_id}/assignment"),
            &ctx.owner_cookie,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(assign1_updated["effective_worker_agent"], "claude");
}

#[tokio::test]
async fn test_p1_03_lead_worker_ac2_cannot_select_unready_provider() {
    let ctx = setup_test_context().await;
    let repo_path = ctx.temp_dir.path().to_str().unwrap();

    let (status, ws_body) = ctx
        .send_request(
            "POST",
            "/api/v1/workspaces",
            &ctx.owner_cookie,
            Some(json!({ "name": "WS P1-03 Ready Check" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let ws_id = ws_body["workspace"]["id"].as_str().unwrap();

    let (status, proj_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/projects"),
            &ctx.owner_cookie,
            Some(json!({
                "name": "Proj P1-03 Ready Check",
                "repo_path": repo_path,
                "default_branch": "main"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let proj_id = proj_body["project"]["id"].as_str().unwrap();

    let (status, task_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/projects/{proj_id}/tasks"),
            &ctx.owner_cookie,
            Some(json!({
                "title": "Task Test Unready",
                "description": "Kiểm tra nhà cung cấp chưa sẵn sàng"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let task_id = task_body["task"]["id"].as_str().unwrap();

    // Hiện tại chưa có tài khoản nào của codex
    let (status, bad_patch) = ctx
        .send_request(
            "PATCH",
            &format!("/api/v1/tasks/{task_id}"),
            &ctx.owner_cookie,
            Some(json!({
                "title": "Task Test Unready",
                "description": "Kiểm tra nhà cung cấp chưa sẵn sàng",
                "worker_agent": "codex"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let err_msg = bad_patch["error"]["message"].as_str().unwrap();
    assert!(
        err_msg.contains("không có tài khoản nào ở trạng thái sẵn sàng"),
        "Thông báo lỗi phải chỉ rõ nhà cung cấp chưa có tài khoản sẵn sàng: {err_msg}"
    );

    // Thêm một tài khoản cho claude
    let (status, _) = ctx
        .send_request(
            "POST",
            "/api/v1/m/subscription-connect/accounts",
            &ctx.owner_cookie,
            Some(json!({
                "provider": "claude",
                "label": "Claude Account",
                "priority": 10
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    ctx.set_account_ready("claude").await;

    // Gán claude thì thành công
    let (status, good_patch) = ctx
        .send_request(
            "PATCH",
            &format!("/api/v1/tasks/{task_id}"),
            &ctx.owner_cookie,
            Some(json!({
                "title": "Task Test Unready",
                "description": "Kiểm tra nhà cung cấp chưa sẵn sàng",
                "worker_agent": "claude"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(good_patch["task"]["worker_agent"], "claude");
}

#[tokio::test]
async fn test_p1_03_lead_worker_ac3_dispatch_missing_worktree_vietnamese_message() {
    let ctx = setup_test_context().await;
    let repo_path = ctx.temp_dir.path().to_str().unwrap();

    let (status, ws_body) = ctx
        .send_request(
            "POST",
            "/api/v1/workspaces",
            &ctx.owner_cookie,
            Some(json!({ "name": "WS P1-03 Dispatch" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let ws_id = ws_body["workspace"]["id"].as_str().unwrap();

    let (status, proj_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/projects"),
            &ctx.owner_cookie,
            Some(json!({
                "name": "Proj P1-03 Dispatch",
                "repo_path": repo_path,
                "default_branch": "main"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let proj_id = proj_body["project"]["id"].as_str().unwrap();

    let (status, task_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/projects/{proj_id}/tasks"),
            &ctx.owner_cookie,
            Some(json!({
                "title": "Task Cần Giao Việc",
                "description": "Giao việc khi chưa có worktree"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let task_id = task_body["task"]["id"].as_str().unwrap();

    // Thêm một tài khoản claude sẵn sàng
    ctx.send_request(
        "POST",
        "/api/v1/m/subscription-connect/accounts",
        &ctx.owner_cookie,
        Some(json!({
            "provider": "claude",
            "label": "Claude Ready",
            "priority": 1
        })),
    )
    .await;

    // Gọi endpoint dispatch
    let (status, dispatch_resp) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/tasks/{task_id}/dispatch"),
            &ctx.owner_cookie,
            Some(json!({})),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(dispatch_resp["error"]["code"], "workdir_provider_missing");
    let msg = dispatch_resp["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("worktree") && msg.contains("Cài đặt > Tính năng"),
        "Thông điệp phải hướng dẫn người dùng bật module 'worktree': {msg}"
    );
}

#[tokio::test]
async fn test_p1_03_lead_worker_ac4_workspace_permission_bola() {
    let ctx = setup_test_context().await;
    let repo_path = ctx.temp_dir.path().to_str().unwrap();

    // Tạo user 2
    let (status, _) = ctx
        .send_request(
            "POST",
            "/api/v1/auth/users",
            &ctx.owner_cookie,
            Some(json!({
                "username": "user_two",
                "password": "Password123!",
                "system_role": "user"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let user2_cookie = ctx.login_user("user_two", "Password123!").await;

    // Workspace A do Owner tạo
    let (status, ws_a) = ctx
        .send_request(
            "POST",
            "/api/v1/workspaces",
            &ctx.owner_cookie,
            Some(json!({ "name": "Workspace A" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let ws_a_id = ws_a["workspace"]["id"].as_str().unwrap();

    let (status, proj_a) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/workspaces/{ws_a_id}/projects"),
            &ctx.owner_cookie,
            Some(json!({
                "name": "Project A",
                "repo_path": repo_path,
                "default_branch": "main"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let proj_a_id = proj_a["project"]["id"].as_str().unwrap();

    let (status, task_a) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/projects/{proj_a_id}/tasks"),
            &ctx.owner_cookie,
            Some(json!({ "title": "Task A", "description": "Thuộc Workspace A" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let task_a_id = task_a["task"]["id"].as_str().unwrap();

    // User 2 (không thuộc Workspace A) cố đọc defaults của Project A
    let (status, _) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/m/lead-worker/projects/{proj_a_id}/defaults"),
            &user2_cookie,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // User 2 cố ghi defaults của Project A
    let (status, _) = ctx
        .send_request(
            "PUT",
            &format!("/api/v1/m/lead-worker/projects/{proj_a_id}/defaults"),
            &user2_cookie,
            Some(json!({ "defaults": [] })),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // User 2 cố xem assignment của Task A
    let (status, _) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/m/lead-worker/tasks/{task_a_id}/assignment"),
            &user2_cookie,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // User 2 cố dispatch Task A
    let (status, _) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/tasks/{task_a_id}/dispatch"),
            &user2_cookie,
            Some(json!({})),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

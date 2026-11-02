//! Bộ kiểm thử tích hợp cho Task P1-05: Module `kanban` và các thao tác chuyển trạng thái task.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P1-05 dùng unwrap"
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
    owner_cookie: String,
    _temp_dir: tempfile::TempDir,
    _state: AppState,
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
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();

        let json_val = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, json_val)
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
        pool,
        Arc::clone(&event_bus),
        Arc::clone(&lifecycle_manager),
        config,
    );
    let app = create_app(state.clone());

    // Tạo owner
    let setup_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "owner", "password": "password123" }).to_string(),
        ))
        .unwrap();

    let setup_resp = app.clone().oneshot(setup_req).await.unwrap();
    assert_eq!(setup_resp.status(), StatusCode::OK);
    let owner_cookie = setup_resp
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    TestContext {
        app,
        owner_cookie,
        _temp_dir: temp_dir,
        _state: state,
    }
}

#[tokio::test]
async fn test_p1_05_kanban_ac1_module_status_and_columns_definition() {
    let ctx = setup_test_context().await;

    // Bật module kanban
    let (status, _) = ctx
        .send_request(
            "POST",
            "/api/v1/features/apply",
            &ctx.owner_cookie,
            Some(json!({
                "module_id": "kanban",
                "scope": "system",
                "target_id": "system",
                "enable": true
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Truy vấn endpoint status của module kanban
    let (status, body) = ctx
        .send_request("GET", "/api/v1/m/kanban/status", &ctx.owner_cookie, None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["module"], "kanban");
    assert_eq!(body["status"], "active");

    let cols = body["columns"].as_array().unwrap();
    assert_eq!(cols.len(), 5);
    let ids: Vec<&str> = cols.iter().map(|c| c["id"].as_str().unwrap()).collect();
    assert_eq!(
        ids,
        vec![
            "backlog",
            "running",
            "needs_human",
            "in_review",
            "ready_to_merge"
        ]
    );
}

#[tokio::test]
async fn test_p1_05_kanban_ac2_valid_and_invalid_status_transitions() {
    let ctx = setup_test_context().await;
    let repo_dir = tempdir().unwrap();
    std::fs::create_dir_all(repo_dir.path().join(".git")).unwrap();
    let repo_path_str = repo_dir.path().to_string_lossy().to_string();

    // 1. Tạo workspace, project và task
    let (status, ws_body) = ctx
        .send_request(
            "POST",
            "/api/v1/workspaces",
            &ctx.owner_cookie,
            Some(json!({ "name": "WS Kanban" })),
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
                "name": "Proj Kanban",
                "repo_path": repo_path_str,
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
                "title": "Nhiệm vụ Kanban mẫu",
                "description": "Kiểm tra chuyển trạng thái"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let task_id = task_body["task"]["id"].as_str().unwrap();
    assert_eq!(task_body["task"]["status"], "backlog");

    // 2. Chuyển không hợp lệ: backlog -> done (bị từ chối với 400 Bad Request)
    let (status, err_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/tasks/{task_id}/status"),
            &ctx.owner_cookie,
            Some(json!({ "status": "done" })),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        err_body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("chuyển đổi không hợp lệ")
    );

    // Xác nhận task vẫn ở trạng thái backlog
    let (status, get_body) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/tasks/{task_id}"),
            &ctx.owner_cookie,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(get_body["task"]["status"], "backlog");

    // 3. Chuyển hợp lệ theo đúng luồng: backlog -> running -> in_review -> ready_to_merge -> done
    let transitions = [
        ("running", "running"),
        ("in_review", "in_review"),
        ("ready_to_merge", "ready_to_merge"),
        ("done", "done"),
    ];

    for (target, expected) in transitions {
        let (status, res_body) = ctx
            .send_request(
                "PATCH",
                &format!("/api/v1/tasks/{task_id}/status"),
                &ctx.owner_cookie,
                Some(json!({ "status": target })),
            )
            .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "Chuyển sang {target} phải thành công"
        );
        assert_eq!(res_body["task"]["status"], expected);
    }

    // 4. Khi đã done, mọi chuyển đổi tiếp theo đều bị từ chối
    let (status, err_body) = ctx
        .send_request(
            "PATCH",
            &format!("/api/v1/tasks/{task_id}/status"),
            &ctx.owner_cookie,
            Some(json!({ "status": "running" })),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        err_body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("chuyển đổi không hợp lệ")
    );
}

#[tokio::test]
async fn test_p1_05_kanban_ac4_bola_security_isolation() {
    let ctx = setup_test_context().await;
    let repo_a_dir = tempdir().unwrap();
    std::fs::create_dir_all(repo_a_dir.path().join(".git")).unwrap();
    let repo_a_str = repo_a_dir.path().to_string_lossy().to_string();

    // 1. Owner tạo Workspace A và Task A
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
            Some(json!({ "name": "Project A", "repo_path": repo_a_str, "default_branch": "main" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let proj_a_id = proj_a["project"]["id"].as_str().unwrap();

    let (status, task_a) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/projects/{proj_a_id}/tasks"),
            &ctx.owner_cookie,
            Some(json!({ "title": "Task A", "description": "Description A" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let task_a_id = task_a["task"]["id"].as_str().unwrap();

    // 2. Tạo User B và đăng nhập
    let (status, _) = ctx
        .send_request(
            "POST",
            "/api/v1/auth/users",
            &ctx.owner_cookie,
            Some(json!({ "username": "user_b", "password": "password123", "system_role": "user" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let login_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "user_b", "password": "password123" }).to_string(),
        ))
        .unwrap();
    let login_resp = ctx.app.clone().oneshot(login_req).await.unwrap();
    assert_eq!(login_resp.status(), StatusCode::OK);
    let cookie_b = login_resp
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // 3. User B cố tình cập nhật status Task A của Workspace A -> Phải bị từ chối 403 Forbidden
    let (status, err_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/tasks/{task_a_id}/status"),
            &cookie_b,
            Some(json!({ "status": "running" })),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(err_body["error"]["code"], "forbidden");
}

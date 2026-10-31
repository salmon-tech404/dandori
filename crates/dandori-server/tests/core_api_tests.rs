use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use dandori_events::EventBus;
use dandori_registry::FeatureRegistry;
use dandori_registry::lifecycle::FeatureLifecycleManager;
use dandori_server::{AppState, ServerConfig, create_app};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_test_server()
-> Result<(axum::Router, SqlitePool, Arc<EventBus>), Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    let core_schema = include_str!("../../../crates/dandori-db/migrations/0001_core_schema.sql");
    sqlx::query(core_schema).execute(&pool).await?;

    let auth_schema = include_str!("../../../crates/mod-auth-login/migrations/0001_auth_login.sql");
    sqlx::query(auth_schema).execute(&pool).await?;

    let event_bus = Arc::new(EventBus::new(100));
    let registry = FeatureRegistry::new(pool.clone());
    registry.seed_modules_and_packs().await?;

    let lifecycle_manager = Arc::new(FeatureLifecycleManager::new(
        registry,
        pool.clone(),
        Arc::clone(&event_bus),
    ));

    let config = ServerConfig::default();
    let state = AppState::new(
        pool.clone(),
        Arc::clone(&event_bus),
        lifecycle_manager,
        config,
    );

    let app = create_app(state);
    Ok((app, pool, event_bus))
}

#[expect(clippy::unwrap_used, reason = "test helper")]
async fn send_req(
    app: &axum::Router,
    method: &str,
    uri: &str,
    cookie: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);

    if let Some(c) = cookie {
        builder = builder.header(header::COOKIE, c);
    }

    let req_body = if let Some(b) = body {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
        Body::from(b.to_string())
    } else {
        Body::empty()
    };

    let req = builder.body(req_body).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
    (status, json)
}

#[expect(clippy::unwrap_used, reason = "test helper")]
async fn login_user(app: &axum::Router, username: &str, password: &str) -> String {
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": username, "password": password }).to_string(),
        ))
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    let cookie_val = resp
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    cookie_val.split(';').next().unwrap().to_string()
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "integration test scenario covering acceptance criteria"
)]
async fn test_ac1_bola_isolation_user_b_cannot_access_workspace_a_data() {
    let (app, _pool, _bus) = setup_test_server().await.unwrap();

    // 1. Tạo owner/admin_a
    let req_init = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "admin_a", "password": "password123" }).to_string(),
        ))
        .unwrap();
    let resp_init = app.clone().oneshot(req_init).await.unwrap();
    let cookie_a = resp_init
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();

    // Tạo user_b
    let (status_create_b, _) = send_req(
        &app,
        "POST",
        "/api/v1/auth/users",
        Some(&cookie_a),
        Some(json!({ "username": "user_b", "password": "password123", "system_role": "user" })),
    )
    .await;
    assert_eq!(status_create_b, StatusCode::OK);

    let cookie_b = login_user(&app, "user_b", "password123").await;

    // 2. User A tạo Workspace A
    let (status_ws_a, body_ws_a) = send_req(
        &app,
        "POST",
        "/api/v1/workspaces",
        Some(&cookie_a),
        Some(json!({ "name": "Workspace A" })),
    )
    .await;
    assert_eq!(status_ws_a, StatusCode::OK);
    let ws_a_id = body_ws_a["workspace"]["id"].as_str().unwrap();

    // Tạo thư mục git giả lập cho repo của Project A
    let temp_git_dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(temp_git_dir.path().join(".git")).unwrap();
    let repo_path_a = temp_git_dir.path().to_str().unwrap();

    // User A tạo Project A trong Workspace A
    let (status_proj_a, body_proj_a) = send_req(
        &app,
        "POST",
        &format!("/api/v1/workspaces/{ws_a_id}/projects"),
        Some(&cookie_a),
        Some(json!({
            "name": "Project A",
            "repo_path": repo_path_a,
            "default_branch": "main"
        })),
    )
    .await;
    assert_eq!(status_proj_a, StatusCode::OK);
    let proj_a_id = body_proj_a["project"]["id"].as_str().unwrap();

    // User A tạo Task A trong Project A
    let (status_task_a, body_task_a) = send_req(
        &app,
        "POST",
        &format!("/api/v1/projects/{proj_a_id}/tasks"),
        Some(&cookie_a),
        Some(json!({
            "title": "Task A",
            "description": "Nhiệm vụ thuộc workspace A"
        })),
    )
    .await;
    assert_eq!(status_task_a, StatusCode::OK);
    let task_a_id = body_task_a["task"]["id"].as_str().unwrap();

    // 3. User B (không thuộc Workspace A) cố gắng truy cập mọi route có :id
    // Danh sách 17 route cần kiểm tra BOLA
    let forbidden_routes = vec![
        ("GET", format!("/api/v1/workspaces/{ws_a_id}"), None),
        (
            "PATCH",
            format!("/api/v1/workspaces/{ws_a_id}"),
            Some(json!({ "name": "Hacked WS" })),
        ),
        ("DELETE", format!("/api/v1/workspaces/{ws_a_id}"), None),
        ("GET", format!("/api/v1/workspaces/{ws_a_id}/members"), None),
        (
            "POST",
            format!("/api/v1/workspaces/{ws_a_id}/members"),
            Some(json!({ "user_id": "dummy", "role": "member" })),
        ),
        (
            "DELETE",
            format!("/api/v1/workspaces/{ws_a_id}/members/dummy"),
            None,
        ),
        (
            "GET",
            format!("/api/v1/workspaces/{ws_a_id}/projects"),
            None,
        ),
        (
            "POST",
            format!("/api/v1/workspaces/{ws_a_id}/projects"),
            Some(json!({ "name": "Hacked P", "repo_path": repo_path_a, "default_branch": "main" })),
        ),
        ("GET", format!("/api/v1/projects/{proj_a_id}"), None),
        (
            "PATCH",
            format!("/api/v1/projects/{proj_a_id}"),
            Some(json!({ "name": "Renamed", "default_branch": "dev" })),
        ),
        ("DELETE", format!("/api/v1/projects/{proj_a_id}"), None),
        ("GET", format!("/api/v1/projects/{proj_a_id}/tasks"), None),
        (
            "POST",
            format!("/api/v1/projects/{proj_a_id}/tasks"),
            Some(json!({ "title": "Hacked Task", "description": "hack" })),
        ),
        ("GET", format!("/api/v1/tasks/{task_a_id}"), None),
        (
            "PATCH",
            format!("/api/v1/tasks/{task_a_id}"),
            Some(json!({ "title": "Renamed Task", "description": "new" })),
        ),
        (
            "POST",
            format!("/api/v1/tasks/{task_a_id}/status"),
            Some(json!({ "status": "running" })),
        ),
        ("DELETE", format!("/api/v1/tasks/{task_a_id}"), None),
    ];

    for (m, uri, body) in forbidden_routes {
        let (status, resp_json) = send_req(&app, m, &uri, Some(&cookie_b), body).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "User B không được phép truy cập {m} {uri}. Nhận: {status} {resp_json}"
        );
    }
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "integration test scenario covering acceptance criteria"
)]
async fn test_ac2_repo_path_validation() {
    let (app, _pool, _bus) = setup_test_server().await.unwrap();

    let req_init = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "admin", "password": "password123" }).to_string(),
        ))
        .unwrap();
    let resp_init = app.clone().oneshot(req_init).await.unwrap();
    let cookie = resp_init
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();

    let (_, body_ws) = send_req(
        &app,
        "POST",
        "/api/v1/workspaces",
        Some(&cookie),
        Some(json!({ "name": "Test WS" })),
    )
    .await;
    let ws_id = body_ws["workspace"]["id"].as_str().unwrap();

    // 1. Đường dẫn không tồn tại
    let (status_not_found, _) = send_req(
        &app,
        "POST",
        &format!("/api/v1/workspaces/{ws_id}/projects"),
        Some(&cookie),
        Some(json!({
            "name": "P1",
            "repo_path": "C:/path/that/does/not/exist/at/all",
            "default_branch": "main"
        })),
    )
    .await;
    assert_eq!(status_not_found, StatusCode::BAD_REQUEST);

    // 2. Đường dẫn chứa ".."
    let (status_traversal, _) = send_req(
        &app,
        "POST",
        &format!("/api/v1/workspaces/{ws_id}/projects"),
        Some(&cookie),
        Some(json!({
            "name": "P2",
            "repo_path": "../relative/traversal",
            "default_branch": "main"
        })),
    )
    .await;
    assert_eq!(status_traversal, StatusCode::BAD_REQUEST);

    // 3. Đường dẫn tồn tại nhưng không phải git repo (thiếu .git)
    let temp_empty_dir = tempfile::tempdir().unwrap();
    let (status_no_git, _) = send_req(
        &app,
        "POST",
        &format!("/api/v1/workspaces/{ws_id}/projects"),
        Some(&cookie),
        Some(json!({
            "name": "P3",
            "repo_path": temp_empty_dir.path().to_str().unwrap(),
            "default_branch": "main"
        })),
    )
    .await;
    assert_eq!(status_no_git, StatusCode::BAD_REQUEST);

    // 4. Đường dẫn git hợp lệ
    let temp_git_dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(temp_git_dir.path().join(".git")).unwrap();
    let (status_valid, body_valid) = send_req(
        &app,
        "POST",
        &format!("/api/v1/workspaces/{ws_id}/projects"),
        Some(&cookie),
        Some(json!({
            "name": "P4 Valid",
            "repo_path": temp_git_dir.path().to_str().unwrap(),
            "default_branch": "main"
        })),
    )
    .await;
    assert_eq!(status_valid, StatusCode::OK);
    assert_eq!(body_valid["project"]["name"], "P4 Valid");
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "integration test scenario covering acceptance criteria"
)]
async fn test_ac3_task_status_state_machine_transitions() {
    let (app, _pool, bus) = setup_test_server().await.unwrap();
    let mut rx = bus.subscribe(dandori_events::EventFilter::all());

    let req_init = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "admin", "password": "password123" }).to_string(),
        ))
        .unwrap();
    let resp_init = app.clone().oneshot(req_init).await.unwrap();
    let cookie = resp_init
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();

    let (_, body_ws) = send_req(
        &app,
        "POST",
        "/api/v1/workspaces",
        Some(&cookie),
        Some(json!({ "name": "Task WS" })),
    )
    .await;
    let ws_id = body_ws["workspace"]["id"].as_str().unwrap();

    let temp_git = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(temp_git.path().join(".git")).unwrap();

    let (_, body_proj) = send_req(
        &app,
        "POST",
        &format!("/api/v1/workspaces/{ws_id}/projects"),
        Some(&cookie),
        Some(json!({
            "name": "Task Proj",
            "repo_path": temp_git.path().to_str().unwrap(),
            "default_branch": "main"
        })),
    )
    .await;
    let proj_id = body_proj["project"]["id"].as_str().unwrap();

    let (_, body_task) = send_req(
        &app,
        "POST",
        &format!("/api/v1/projects/{proj_id}/tasks"),
        Some(&cookie),
        Some(json!({ "title": "My Task", "description": "desc" })),
    )
    .await;
    let task_id = body_task["task"]["id"].as_str().unwrap();
    assert_eq!(body_task["task"]["status"], "backlog");

    // 1. Chuyển hợp lệ 1: backlog -> running
    let (status_run, body_run) = send_req(
        &app,
        "POST",
        &format!("/api/v1/tasks/{task_id}/status"),
        Some(&cookie),
        Some(json!({ "status": "running" })),
    )
    .await;
    assert_eq!(status_run, StatusCode::OK);
    assert_eq!(body_run["task"]["status"], "running");

    // Kiểm tra EventBus nhận được sự kiện task.status_changed
    let mut received_event = false;
    tokio::select! {
        _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {}
        _ = async {
            while let Some(envelope) = rx.recv().await {
                if envelope.kind == dandori_types::EventKind::TaskStatusChanged {
                    received_event = true;
                    break;
                }
            }
        } => {}
    }
    assert!(received_event, "EventBus phải phát TaskStatusChanged");

    // 2. Chuyển hợp lệ 2: running -> in_review
    let (status_rev, body_rev) = send_req(
        &app,
        "POST",
        &format!("/api/v1/tasks/{task_id}/status"),
        Some(&cookie),
        Some(json!({ "status": "in_review" })),
    )
    .await;
    assert_eq!(status_rev, StatusCode::OK);
    assert_eq!(body_rev["task"]["status"], "in_review");

    // 3. Chuyển hợp lệ 3: in_review -> ready_to_merge
    let (status_ready, body_ready) = send_req(
        &app,
        "POST",
        &format!("/api/v1/tasks/{task_id}/status"),
        Some(&cookie),
        Some(json!({ "status": "ready_to_merge" })),
    )
    .await;
    assert_eq!(status_ready, StatusCode::OK);
    assert_eq!(body_ready["task"]["status"], "ready_to_merge");

    // 4. Chuyển hợp lệ 4: ready_to_merge -> done
    let (status_done, body_done) = send_req(
        &app,
        "POST",
        &format!("/api/v1/tasks/{task_id}/status"),
        Some(&cookie),
        Some(json!({ "status": "done" })),
    )
    .await;
    assert_eq!(status_done, StatusCode::OK);
    assert_eq!(body_done["task"]["status"], "done");

    // 5. Chuyển KHÔNG hợp lệ 1: done -> running (từ chối)
    let (status_inv1, _) = send_req(
        &app,
        "POST",
        &format!("/api/v1/tasks/{task_id}/status"),
        Some(&cookie),
        Some(json!({ "status": "running" })),
    )
    .await;
    assert_eq!(status_inv1, StatusCode::BAD_REQUEST);

    // Tạo task thứ hai để test thêm các chuyển không hợp lệ
    let (_, body_task2) = send_req(
        &app,
        "POST",
        &format!("/api/v1/projects/{proj_id}/tasks"),
        Some(&cookie),
        Some(json!({ "title": "Task 2", "description": "desc 2" })),
    )
    .await;
    let task2_id = body_task2["task"]["id"].as_str().unwrap();

    // 6. Chuyển KHÔNG hợp lệ 2: backlog -> done (bỏ qua các bước, từ chối)
    let (status_inv2, _) = send_req(
        &app,
        "POST",
        &format!("/api/v1/tasks/{task2_id}/status"),
        Some(&cookie),
        Some(json!({ "status": "done" })),
    )
    .await;
    assert_eq!(status_inv2, StatusCode::BAD_REQUEST);

    // 7. Chuyển KHÔNG hợp lệ 3: backlog -> ready_to_merge (từ chối)
    let (status_inv3, _) = send_req(
        &app,
        "POST",
        &format!("/api/v1/tasks/{task2_id}/status"),
        Some(&cookie),
        Some(json!({ "status": "ready_to_merge" })),
    )
    .await;
    assert_eq!(status_inv3, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_ac4_unknown_fields_rejected_by_deny_unknown_fields() {
    let (app, _pool, _bus) = setup_test_server().await.unwrap();

    let req_init = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "admin", "password": "password123" }).to_string(),
        ))
        .unwrap();
    let resp_init = app.clone().oneshot(req_init).await.unwrap();
    let cookie = resp_init
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();

    // 1. Workspace payload chứa trường lạ
    let (status_ws, _) = send_req(
        &app,
        "POST",
        "/api/v1/workspaces",
        Some(&cookie),
        Some(json!({
            "name": "WS",
            "unexpected_field": "injected"
        })),
    )
    .await;
    assert_eq!(status_ws, StatusCode::UNPROCESSABLE_ENTITY);

    // 2. Setup owner với trường lạ
    let req_extra = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "admin2", "password": "password123", "extra": 42 }).to_string(),
        ))
        .unwrap();
    let resp_extra = app.oneshot(req_extra).await.unwrap();
    assert_eq!(resp_extra.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "integration test scenario covering acceptance criteria"
)]
async fn test_ac5_registry_three_tier_permissions() {
    let (app, _pool, _bus) = setup_test_server().await.unwrap();

    // Khởi tạo chủ (system_role = admin)
    let req_init = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "sys_admin", "password": "password123" }).to_string(),
        ))
        .unwrap();
    let resp_init = app.clone().oneshot(req_init).await.unwrap();
    let cookie_sys = resp_init
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();

    // Tạo user thường
    send_req(
        &app,
        "POST",
        "/api/v1/auth/users",
        Some(&cookie_sys),
        Some(
            json!({ "username": "regular_user", "password": "password123", "system_role": "user" }),
        ),
    )
    .await;
    let cookie_reg = login_user(&app, "regular_user", "password123").await;

    // 1. SystemAdmin có thể cấu hình cấp system
    let (status_sys_plan, _) = send_req(
        &app,
        "POST",
        "/api/v1/features/plan-enable",
        Some(&cookie_sys),
        Some(json!({
            "module_id": "sample-hello",
            "scope": "system",
            "target_id": "system"
        })),
    )
    .await;
    assert_eq!(status_sys_plan, StatusCode::OK);

    // 2. Regular user KHÔNG thể cấu hình cấp system -> 403 Forbidden
    let (status_reg_plan, _) = send_req(
        &app,
        "POST",
        "/api/v1/features/plan-enable",
        Some(&cookie_reg),
        Some(json!({
            "module_id": "sample-hello",
            "scope": "system",
            "target_id": "system"
        })),
    )
    .await;
    assert_eq!(status_reg_plan, StatusCode::FORBIDDEN);

    // 3. Tra cứu effective state
    let (status_eff, body_eff) = send_req(
        &app,
        "GET",
        "/api/v1/features/effective?module_id=sample-hello&workspace=ws_1&project=proj_1",
        Some(&cookie_reg),
        None,
    )
    .await;
    assert_eq!(status_eff, StatusCode::OK);
    assert!(body_eff.get("effective").is_some());
}

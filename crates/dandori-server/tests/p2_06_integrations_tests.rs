//! Bộ kiểm thử tích hợp cho Task P2-06: Module `integrations` và `GitHostProvider`.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    clippy::indexing_slicing,
    reason = "Kiểm thử tích hợp P2-06 dùng unwrap và indexing json"
)]

use std::sync::Arc;

use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use dandori_events::EventBus;
use dandori_module_api::{CreatePullRequestPayload, MergeMethod};
use dandori_registry::{FeatureLifecycleManager, FeatureRegistry, RegistryRole, RegistryScope};
use dandori_server::app::create_app;
use dandori_server::config::ServerConfig;
use dandori_server::state::AppState;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tempfile::tempdir;
use tokio::net::TcpListener;
use tower::ServiceExt;

struct MockServer {
    base_url: String,
    _handle: tokio::task::JoinHandle<()>,
}

async fn start_mock_git_server() -> MockServer {
    let app = Router::new()
        // GitHub Endpoints
        .route(
            "/user",
            get(|headers: HeaderMap| async move {
                let auth = headers.get("authorization").and_then(|h| h.to_str().ok());
                if auth == Some("Bearer valid_github_token") {
                    (StatusCode::OK, Json(json!({ "login": "octocat" }))).into_response()
                } else {
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({ "message": "Bad credentials" })),
                    )
                        .into_response()
                }
            }),
        )
        .route(
            "/repos/{owner}/{repo}/pulls",
            post(|Json(body): Json<Value>| async move {
                (
                    StatusCode::CREATED,
                    Json(json!({
                        "number": 42,
                        "html_url": "https://github.com/test-org/test-repo/pull/42",
                        "title": body["title"],
                        "state": "open",
                        "head": { "ref": body["head"] },
                        "base": { "ref": body["base"] },
                        "mergeable": true,
                        "merged": false
                    })),
                )
            }),
        )
        .route(
            "/repos/{owner}/{repo}/pulls/{number}",
            get(|| async move {
                Json(json!({
                    "number": 42,
                    "html_url": "https://github.com/test-org/test-repo/pull/42",
                    "title": "PR Title",
                    "state": "open",
                    "head": { "ref": "task/p2-06" },
                    "base": { "ref": "main" },
                    "mergeable": true,
                    "merged": false
                }))
            }),
        )
        .route(
            "/repos/{owner}/{repo}/commits/{git_ref}/check-runs",
            get(|| async move {
                Json(json!({
                    "total_count": 2,
                    "check_runs": [
                        { "conclusion": "success", "status": "completed" },
                        { "conclusion": "success", "status": "completed" }
                    ]
                }))
            }),
        )
        .route(
            "/repos/{owner}/{repo}/pulls/{number}/merge",
            put(|Json(body): Json<Value>| async move {
                let method = body["merge_method"].as_str().unwrap_or("merge");
                Json(json!({
                    "sha": "gh_merge_sha_123",
                    "merged": true,
                    "message": format!("Merged with {method}")
                }))
            }),
        )
        // GitLab Endpoints
        .route(
            "/api/v4/user",
            get(|headers: HeaderMap| async move {
                let token = headers.get("private-token").and_then(|h| h.to_str().ok());
                if token == Some("valid_gitlab_token") {
                    (StatusCode::OK, Json(json!({ "username": "gitlab-user" }))).into_response()
                } else {
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({ "message": "401 Unauthorized" })),
                    )
                        .into_response()
                }
            }),
        )
        .route(
            "/api/v4/projects/{repo}/merge_requests",
            post(|Json(body): Json<Value>| async move {
                (
                    StatusCode::CREATED,
                    Json(json!({
                        "iid": 10,
                        "web_url": "https://gitlab.com/test-org/test-repo/-/merge_requests/10",
                        "title": body["title"],
                        "state": "opened",
                        "source_branch": body["source_branch"],
                        "target_branch": body["target_branch"],
                        "has_conflicts": false
                    })),
                )
            }),
        )
        .route(
            "/api/v4/projects/{repo}/merge_requests/{number}",
            get(|| async move {
                Json(json!({
                    "iid": 10,
                    "web_url": "https://gitlab.com/test-org/test-repo/-/merge_requests/10",
                    "title": "MR Title",
                    "state": "opened",
                    "source_branch": "task/p2-06",
                    "target_branch": "main",
                    "has_conflicts": false
                }))
            }),
        )
        .route(
            "/api/v4/projects/{repo}/pipelines",
            get(|| async move {
                Json(json!([
                    { "id": 100, "status": "success" }
                ]))
            }),
        )
        .route(
            "/api/v4/projects/{repo}/merge_requests/{number}/merge",
            put(|| async move {
                Json(json!({
                    "title": "MR Merged",
                    "state": "merged",
                    "merge_commit_sha": "gl_merge_sha_456"
                }))
            }),
        )
        // Linear Endpoints
        .route(
            "/graphql",
            post(|Json(body): Json<Value>| async move {
                let query = body["query"].as_str().unwrap_or_default();
                if query.contains("viewer") {
                    Json(json!({ "data": { "viewer": { "id": "u-1", "name": "Linear Tester" } } }))
                } else if query.contains("issues") {
                    Json(json!({
                        "data": {
                            "issues": {
                                "nodes": [
                                    {
                                        "id": "lin-issue-1",
                                        "identifier": "DAN-101",
                                        "title": "Tác vụ mẫu từ Linear",
                                        "description": "Chi tiết tác vụ",
                                        "url": "https://linear.app/dandori/issue/DAN-101",
                                        "state": { "name": "Todo" }
                                    }
                                ]
                            }
                        }
                    }))
                } else {
                    Json(json!({ "data": null }))
                }
            }),
        );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let base_url = format!("http://127.0.0.1:{port}");

    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    MockServer {
        base_url,
        _handle: handle,
    }
}

struct TestContext {
    app: axum::Router,
    owner_cookie: String,
    non_member_cookie: String,
    pool: SqlitePool,
    lifecycle_manager: Arc<FeatureLifecycleManager>,
    workspace_id: String,
    project_id: String,
    mock_server: MockServer,
    _temp_dir: tempfile::TempDir,
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
    // Máy chủ giả nằm ở 127.0.0.1: bật công tắc dành riêng cho kiểm thử (mặc định tắt trong bản chạy thật).
    dandori_module_api::set_allow_local_endpoints_for_tests(true);
    let mock_server = start_mock_git_server().await;
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
        pool.clone(),
        Arc::clone(&event_bus),
        Arc::clone(&lifecycle_manager),
        config,
    );

    let app = create_app(state);

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

    let non_member_cookie = login_new_user(&app, &owner_cookie, "non_member").await;

    let ws_req = Request::builder()
        .method("POST")
        .uri("/api/v1/workspaces")
        .header(header::COOKIE, &owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "name": "Workspace Integrations" }).to_string(),
        ))
        .unwrap();
    let ws_resp = app.clone().oneshot(ws_req).await.unwrap();
    let ws_body: Value = serde_json::from_slice(
        &axum::body::to_bytes(ws_resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let workspace_id = ws_body["workspace"]["id"].as_str().unwrap().to_string();

    let project_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'Project Integrations', ?, 'main', datetime('now'))"
    )
    .bind(&project_id)
    .bind(&workspace_id)
    .bind(temp_dir.path().to_str().unwrap())
    .execute(&pool)
    .await
    .unwrap();

    lifecycle_manager
        .enable(
            "integrations",
            &RegistryScope::System,
            RegistryRole::SystemOwner,
            "owner",
            true,
        )
        .await
        .unwrap();

    TestContext {
        app,
        owner_cookie,
        non_member_cookie,
        pool,
        lifecycle_manager,
        workspace_id,
        project_id,
        mock_server,
        _temp_dir: temp_dir,
    }
}

#[tokio::test]
async fn test_ac1_connection_test_valid_and_invalid_tokens_without_token_leak() {
    let ctx = setup_test_context().await;

    // 1. GitHub với token đúng -> 200 OK
    let test_req1 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/integrations/workspaces/{}/connections/test",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "github",
                "base_url": ctx.mock_server.base_url,
                "token": "valid_github_token",
                "allow_private_network": true
            })
            .to_string(),
        ))
        .unwrap();

    let resp1 = ctx.app.clone().oneshot(test_req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);

    // 2. GitHub với token sai -> 400 Bad Request và response body KHÔNG chứa token
    let bad_secret = "ghp_super_secret_bad_token_999";
    let test_req2 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/integrations/workspaces/{}/connections/test",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "github",
                "base_url": ctx.mock_server.base_url,
                "token": bad_secret,
                "allow_private_network": true
            })
            .to_string(),
        ))
        .unwrap();

    let resp2 = ctx.app.clone().oneshot(test_req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(resp2.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(
        !body_str.contains(bad_secret),
        "Thông điệp lỗi API không được làm lộ token: {body_str}"
    );

    // 3. GitLab với token đúng -> 200 OK
    let test_req3 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/integrations/workspaces/{}/connections/test",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "gitlab",
                "base_url": ctx.mock_server.base_url,
                "token": "valid_gitlab_token",
                "allow_private_network": true
            })
            .to_string(),
        ))
        .unwrap();

    let resp3 = ctx.app.clone().oneshot(test_req3).await.unwrap();
    assert_eq!(resp3.status(), StatusCode::OK);

    // 4. GitLab với token sai -> 400 Bad Request
    let test_req4 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/integrations/workspaces/{}/connections/test",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "gitlab",
                "base_url": ctx.mock_server.base_url,
                "token": "bad_token",
                "allow_private_network": true
            })
            .to_string(),
        ))
        .unwrap();

    let resp4 = ctx.app.clone().oneshot(test_req4).await.unwrap();
    assert_eq!(resp4.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_ac2_ssrf_protection_rules() {
    let ctx = setup_test_context().await;

    // 1. Tên miền ngoài allowlist bị chặn (GitHub) -> 400
    let req1 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/integrations/workspaces/{}/connections",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "github",
                "name": "Attacker GitHub",
                "base_url": "https://attacker-domain.com",
                "token": "any_token",
                "allow_private_network": false
            })
            .to_string(),
        ))
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(req1).await.unwrap().status(),
        StatusCode::BAD_REQUEST
    );

    // 2. IP Loopback bị chặn khi allow_private_network = false -> 400
    let req2 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/integrations/workspaces/{}/connections",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "gitlab",
                "name": "Loopback GitLab",
                "base_url": "https://127.0.0.1",
                "token": "any_token",
                "allow_private_network": false
            })
            .to_string(),
        ))
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(req2).await.unwrap().status(),
        StatusCode::BAD_REQUEST
    );

    // 3. IP Metadata 169.254.169.254 bị chặn tuyệt đối kể cả khi allow_private_network = true -> 400
    let req3 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/integrations/workspaces/{}/connections",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "gitlab",
                "name": "Metadata Steal",
                "base_url": "https://169.254.169.254",
                "token": "any_token",
                "allow_private_network": true
            })
            .to_string(),
        ))
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(req3).await.unwrap().status(),
        StatusCode::BAD_REQUEST
    );

    // 4. Mạng nội bộ RFC 1918 bị chặn khi allow_private_network = false -> 400
    let req4 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/integrations/workspaces/{}/connections",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "gitlab",
                "name": "Private IP GitLab",
                "base_url": "https://192.168.1.50",
                "token": "any_token",
                "allow_private_network": false
            })
            .to_string(),
        ))
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(req4).await.unwrap().status(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn test_ac3_token_is_never_returned_by_api() {
    let ctx = setup_test_context().await;

    // Tạo kết nối qua API với token bí mật
    let create_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/integrations/workspaces/{}/connections",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "github",
                "name": "GitHub CI Project",
                "base_url": ctx.mock_server.base_url,
                "token": "valid_github_token",
                "allow_private_network": true
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Khẳng định token KHÔNG xuất hiện trong phản hồi API
    assert!(!body_str.contains("valid_github_token"));
    assert!(!body_str.contains("token_encrypted"));
    assert!(body_str.contains("GitHub CI Project"));

    // Lấy danh sách kết nối qua GET
    let list_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/integrations/workspaces/{}/connections",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();

    let list_resp = ctx.app.clone().oneshot(list_req).await.unwrap();
    let list_bytes = axum::body::to_bytes(list_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let list_str = String::from_utf8(list_bytes.to_vec()).unwrap();
    assert!(!list_str.contains("valid_github_token"));
}

#[tokio::test]
async fn test_ac4_git_host_provider_contract_github_and_gitlab() {
    let ctx = setup_test_context().await;

    // 1. Tạo kết nối GitHub qua API
    let create_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/integrations/workspaces/{}/connections",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "github",
                "name": "GitHub Provider",
                "base_url": ctx.mock_server.base_url,
                "token": "valid_github_token",
                "allow_private_network": true
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Lấy provider được đăng ký thông qua lifecycle_manager
    let provider = ctx
        .lifecycle_manager
        .get_git_host_provider("integrations")
        .await
        .expect("integrations provider must be registered");

    assert_eq!(provider.name(), "integrations");
    assert!(provider.test_connection().await.is_ok());

    let pr_payload = CreatePullRequestPayload {
        title: "P2-06 Feature PR".to_string(),
        body: "Automated PR from Dandori".to_string(),
        source_branch: "task/p2-06".to_string(),
        target_branch: "main".to_string(),
    };

    let pr = provider
        .create_pull_request("test-org/test-repo", &pr_payload)
        .await
        .unwrap();
    assert_eq!(pr.number, 42);
    assert_eq!(pr.mergeable, Some(true));

    let pr_fetched = provider
        .get_pull_request("test-org/test-repo", 42)
        .await
        .unwrap();
    assert_eq!(pr_fetched.number, 42);

    let ci = provider
        .get_ci_status("test-org/test-repo", "main")
        .await
        .unwrap();
    assert_eq!(ci.success_count, 2);

    let merge_res = provider
        .merge_pull_request("test-org/test-repo", 42, MergeMethod::Squash)
        .await
        .unwrap();
    assert!(merge_res.merged);
    assert_eq!(merge_res.sha, Some("gh_merge_sha_123".to_string()));

    // 2. Chuyển kết nối thành GitLab
    // Token mới cũng phải đi qua vault (cột chỉ giữ tham chiếu `vault:<id>`).
    let vault = dandori_module_api::Vault::new(
        dandori_module_api::SqliteSecretStorage::new(ctx.pool.clone()),
        dandori_module_api::default_key_store(),
    );
    let gl_secret = vault
        .put(
            "test-gitlab-token",
            dandori_module_api::SecretBox::from("valid_gitlab_token"),
        )
        .await
        .unwrap();
    sqlx::query("UPDATE m_integration_connection SET provider = 'gitlab', token_encrypted = ?")
        .bind(format!("vault:{gl_secret}"))
        .execute(&ctx.pool)
        .await
        .unwrap();

    let mr = provider
        .create_pull_request("test-org/test-repo", &pr_payload)
        .await
        .unwrap();
    assert_eq!(mr.number, 10);
    assert_eq!(mr.mergeable, Some(true));

    let mr_fetched = provider
        .get_pull_request("test-org/test-repo", 10)
        .await
        .unwrap();
    assert_eq!(mr_fetched.number, 10);

    let gl_ci = provider
        .get_ci_status("test-org/test-repo", "main")
        .await
        .unwrap();
    assert_eq!(gl_ci.success_count, 1);

    let gl_merge = provider
        .merge_pull_request("test-org/test-repo", 10, MergeMethod::Merge)
        .await
        .unwrap();
    assert!(gl_merge.merged);
    assert_eq!(gl_merge.sha, Some("gl_merge_sha_456".to_string()));
}

#[tokio::test]
async fn test_ac5_linear_issue_import_and_deduplication() {
    let ctx = setup_test_context().await;

    // 1. Tạo kết nối Linear qua API
    let create_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/integrations/workspaces/{}/connections",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "linear",
                "name": "Linear Workspace",
                "base_url": format!("{}/graphql", ctx.mock_server.base_url),
                "token": "valid_linear_token",
                "allow_private_network": true
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let conn_val: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let conn_id = conn_val["connection"]["id"].as_str().unwrap();

    // 2. Nhập issue vào backlog lần 1 -> Nhập thành công 1 task
    let import_req1 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/integrations/connections/{conn_id}/linear/import"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "project_id": ctx.project_id,
                "issue_ids": []
            })
            .to_string(),
        ))
        .unwrap();

    let import_resp1 = ctx.app.clone().oneshot(import_req1).await.unwrap();
    assert_eq!(import_resp1.status(), StatusCode::OK);
    let import_val1: Value = serde_json::from_slice(
        &axum::body::to_bytes(import_resp1.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(import_val1["imported_count"], 1);

    // Xác nhận task và bản ghi liên kết đã được tạo trong DB
    let (task_title, status): (String, String) = sqlx::query_as(
        "SELECT title, status FROM task WHERE project_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&ctx.project_id)
    .fetch_one(&ctx.pool)
    .await
    .unwrap();
    assert!(task_title.contains("[DAN-101]"));
    assert_eq!(status, "backlog");

    // 3. Nhập lại lần 2 -> Số lượng nhập là 0 (chống trùng lặp)
    let import_req2 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/integrations/connections/{conn_id}/linear/import"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "project_id": ctx.project_id,
                "issue_ids": []
            })
            .to_string(),
        ))
        .unwrap();

    let import_resp2 = ctx.app.clone().oneshot(import_req2).await.unwrap();
    assert_eq!(import_resp2.status(), StatusCode::OK);
    let import_val2: Value = serde_json::from_slice(
        &axum::body::to_bytes(import_resp2.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        import_val2["imported_count"], 0,
        "Nhập lại issue đã tồn tại phải bỏ qua để chống trùng lặp"
    );
}

#[tokio::test]
async fn test_ac6_workspace_isolation_bola() {
    let ctx = setup_test_context().await;

    // Người dùng không thuộc workspace cố gắng xem danh sách kết nối -> 403 Forbidden
    let list_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/integrations/workspaces/{}/connections",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.non_member_cookie)
        .body(Body::empty())
        .unwrap();

    let resp = ctx.app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Người dùng không thuộc workspace cố gắng tạo kết nối -> 403 Forbidden
    let create_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/integrations/workspaces/{}/connections",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.non_member_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "github",
                "name": "BOLA Hack Attempt",
                "token": "tok"
            })
            .to_string(),
        ))
        .unwrap();

    let create_resp = ctx.app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::FORBIDDEN);
}

async fn post_connection(
    ctx: &TestContext,
    cookie: &str,
    provider: &str,
    token: &str,
    allow_private: bool,
) -> (StatusCode, Value) {
    let base = if provider == "linear" {
        format!("{}/graphql", ctx.mock_server.base_url)
    } else {
        ctx.mock_server.base_url.clone()
    };
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/integrations/workspaces/{}/connections",
            ctx.workspace_id
        ))
        .header(header::COOKIE, cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": provider,
                "name": "conn",
                "base_url": base,
                "token": token,
                "allow_private_network": allow_private
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap_or(Value::Null);
    (status, body)
}

#[tokio::test]
async fn test_token_is_stored_in_vault_and_never_as_plaintext() {
    let ctx = setup_test_context().await;
    let token = "valid_github_token";
    let (status, body) = post_connection(&ctx, &ctx.owner_cookie, "github", token, true).await;
    assert_eq!(status, StatusCode::CREATED);
    let conn_id = body["connection"]["id"].as_str().unwrap().to_string();

    let stored: String =
        sqlx::query_scalar("SELECT token_encrypted FROM m_integration_connection WHERE id = ?")
            .bind(&conn_id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert!(
        stored.starts_with("vault:"),
        "cột token chỉ được giữ tham chiếu vault, thấy: {stored}"
    );
    assert!(!stored.contains(token));

    // Bản mã trong bảng secret cũng không chứa token thô.
    let blobs: Vec<Vec<u8>> = sqlx::query_scalar("SELECT ciphertext FROM secret")
        .fetch_all(&ctx.pool)
        .await
        .unwrap();
    assert!(!blobs.is_empty(), "token phải nằm trong bảng secret");
    assert!(
        blobs
            .iter()
            .all(|b| !b.windows(token.len()).any(|w| w == token.as_bytes())),
        "bản mã không được chứa token thô"
    );

    // Kết nối đã lưu vẫn dùng được (giải mã đúng khi gọi).
    let test_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/m/integrations/connections/{conn_id}/test"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(test_req).await.unwrap().status(),
        StatusCode::OK
    );

    // Xóa kết nối cũng xóa bí mật.
    let del = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/m/integrations/connections/{conn_id}"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(del).await.unwrap().status(),
        StatusCode::NO_CONTENT
    );
    let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM secret")
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    assert_eq!(left, 0, "xóa kết nối phải xóa cả bí mật trong vault");
}

#[tokio::test]
async fn test_only_workspace_admins_manage_connections_and_only_system_admin_allows_private() {
    let ctx = setup_test_context().await;
    let member_cookie = login_new_user(&ctx.app, &ctx.owner_cookie, "plain_member").await;
    let ws_admin_cookie = login_new_user(&ctx.app, &ctx.owner_cookie, "ws_admin").await;
    for (name, role) in [("plain_member", "member"), ("ws_admin", "admin")] {
        let uid: String = sqlx::query_scalar("SELECT id FROM user WHERE username = ?")
            .bind(name)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, ?)")
            .bind(&ctx.workspace_id)
            .bind(uid)
            .bind(role)
            .execute(&ctx.pool)
            .await
            .unwrap();
    }

    // Thành viên thường: không được tạo kết nối (token + tới dịch vụ ngoài), kể cả khi là thành viên.
    let (status, _) =
        post_connection(&ctx, &member_cookie, "github", "valid_github_token", true).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "thành viên thường");

    // Quản trị workspace nhưng không phải quản trị hệ thống: không được bật mạng nội bộ.
    let (status, _) =
        post_connection(&ctx, &ws_admin_cookie, "github", "valid_github_token", true).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "allow_private_network");

    // Quản trị hệ thống thì được.
    let (status, body) = post_connection(
        &ctx,
        &ctx.owner_cookie,
        "github",
        "valid_github_token",
        true,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let conn_id = body["connection"]["id"].as_str().unwrap().to_string();

    // Xóa kết nối cũng chỉ dành cho quản trị workspace (đường này không dính cờ mạng nội bộ nên
    // chứng minh riêng quyền quản trị): thành viên thường 403, quản trị workspace 204.
    let delete_as = |cookie: &str| {
        Request::builder()
            .method("DELETE")
            .uri(format!("/api/v1/m/integrations/connections/{conn_id}"))
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap()
    };
    assert_eq!(
        ctx.app
            .clone()
            .oneshot(delete_as(&member_cookie))
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN,
        "thành viên thường không được xóa kết nối"
    );
    assert_eq!(
        ctx.app
            .clone()
            .oneshot(delete_as(&ws_admin_cookie))
            .await
            .unwrap()
            .status(),
        StatusCode::NO_CONTENT,
        "quản trị workspace được xóa kết nối"
    );
}

#[tokio::test]
async fn test_linear_import_into_project_of_another_workspace_is_rejected() {
    let ctx = setup_test_context().await;
    let (status, body) = post_connection(
        &ctx,
        &ctx.owner_cookie,
        "linear",
        "valid_linear_token",
        true,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let conn_id = body["connection"]["id"].as_str().unwrap().to_string();

    // Dự án thuộc workspace khác.
    let other_ws = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO workspace (id, name, created_at) VALUES (?, 'Other', datetime('now'))",
    )
    .bind(&other_ws)
    .execute(&ctx.pool)
    .await
    .unwrap();
    let foreign_project = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'Foreign', '/x', 'main', datetime('now'))",
    )
    .bind(&foreign_project)
    .bind(&other_ws)
    .execute(&ctx.pool)
    .await
    .unwrap();

    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/integrations/connections/{conn_id}/linear/import"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "project_id": foreign_project, "issue_ids": [] }).to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_ne!(
        resp.status(),
        StatusCode::OK,
        "không được nhập vào dự án của workspace khác"
    );

    let created: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM task WHERE project_id = ?")
        .bind(&foreign_project)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    assert_eq!(
        created, 0,
        "không được tạo task trong dự án của workspace khác"
    );
}

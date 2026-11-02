//! Các bài kiểm thử tích hợp cho Task P1-04: Module `worktree` và crate `dandori-git`.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P1-04 dùng unwrap/expect và có fixture git dài"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use dandori_events::EventBus;
use dandori_git::GitClient;
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

    async fn set_account_ready(&self, provider: &str) {
        sqlx::query("UPDATE agent_account SET status = 'ready' WHERE provider = ?")
            .bind(provider)
            .execute(&self.pool)
            .await
            .unwrap();
    }
}

async fn init_git_repo(path: &Path) {
    let client = GitClient::new(path);
    client.init_repo("main").await.expect("git init");
    client
        .config("user.name", "Test Committer")
        .await
        .expect("git config user.name");
    client
        .config("user.email", "committer@example.com")
        .await
        .expect("git config user.email");
}

async fn commit_sample_file(path: &Path) {
    fs::write(path.join("file1.txt"), "Dandori Project Base\n").expect("write file1");
    let client = GitClient::new(path);
    client
        .commit_all("Initial baseline commit")
        .await
        .expect("git commit");
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

    let fake_bin = std::env::current_exe()
        .ok()
        .and_then(|p| {
            let parent = p.parent()?;
            let cand1 = parent.join(if cfg!(windows) {
                "fake-agent.exe"
            } else {
                "fake-agent"
            });
            if cand1.exists() {
                return Some(cand1);
            }
            let cand2 = parent.parent()?.join(if cfg!(windows) {
                "fake-agent.exe"
            } else {
                "fake-agent"
            });
            if cand2.exists() {
                return Some(cand2);
            }
            None
        })
        .unwrap_or_else(|| {
            PathBuf::from(if cfg!(windows) {
                "target/debug/fake-agent.exe"
            } else {
                "target/debug/fake-agent"
            })
        });
    let fake_adapter = Arc::new(dandori_agent_adapters::FakeAdapter::new(fake_bin));
    state
        .run_service
        .register_adapter("fake", fake_adapter.clone())
        .await;
    state
        .run_service
        .register_adapter("fake-agent", fake_adapter)
        .await;

    let app = create_app(state.clone());

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

    // Kích hoạt các module cần thiết: subscription-connect, lead-worker, worktree
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
    lifecycle_manager
        .enable(
            "worktree",
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
        _temp_dir: temp_dir,
        _state: state,
    }
}

#[tokio::test]
async fn test_p1_04_worktree_ac1_prepare_and_main_repo_hash_untouched() {
    let ctx = setup_test_context().await;
    let repo_dir = tempdir().unwrap();
    init_git_repo(repo_dir.path()).await;
    commit_sample_file(repo_dir.path()).await;

    let repo_path_str = repo_dir.path().to_string_lossy().to_string();
    let git_client = GitClient::new(repo_dir.path());
    let head_before = git_client.rev_parse("HEAD").await.unwrap();

    // 1. Tạo workspace và project
    let (status, ws_body) = ctx
        .send_request(
            "POST",
            "/api/v1/workspaces",
            &ctx.owner_cookie,
            Some(json!({ "name": "WS P1-04 AC1" })),
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
                "name": "Proj P1-04 Worktree",
                "repo_path": repo_path_str,
                "default_branch": "main"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let proj_id = proj_body["project"]["id"].as_str().unwrap();

    // 2. Tạo task
    let (status, task_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/projects/{proj_id}/tasks"),
            &ctx.owner_cookie,
            Some(json!({
                "title": "Tạo tính năng mới cho AC1",
                "description": "Kiểm tra worktree và nhánh"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let task_id = task_body["task"]["id"].as_str().unwrap();

    // 3. Thêm tài khoản agent ready
    let (status, _) = ctx
        .send_request(
            "POST",
            "/api/v1/m/subscription-connect/accounts",
            &ctx.owner_cookie,
            Some(json!({
                "provider": "fake",
                "label": "Fake Agent Ready"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    ctx.set_account_ready("fake").await;

    // 4. Kích hoạt giao việc (dispatch)
    let (status, dispatch_resp) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/tasks/{task_id}/dispatch"),
            &ctx.owner_cookie,
            None,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Dispatch phải thành công: {dispatch_resp:?}"
    );

    // 5. Kiểm tra thông tin worktree qua module route
    let (status, wt_resp) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/m/worktree/tasks/{task_id}/worktree"),
            &ctx.owner_cookie,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(wt_resp["exists"], true);
    let wt_path = wt_resp["workdir_path"].as_str().unwrap();
    assert!(Path::new(wt_path).exists());
    assert!(Path::new(wt_path).join("file1.txt").exists());

    let branch = wt_resp["branch"].as_str().unwrap();
    assert!(branch.starts_with("task/tao-tinh-nang-moi-cho-ac1"));

    // Kiểm tra git worktree list trên repo gốc
    let wt_list = git_client.worktree_list().await.unwrap();
    assert!(wt_list.iter().any(|w| w.branch.as_deref() == Some(branch)));

    // Xác nhận cây làm việc chính của repo gốc KHÔNG HỀ BỊ THAY ĐỔI
    let head_after = git_client.rev_parse("HEAD").await.unwrap();
    assert_eq!(
        head_before, head_after,
        "Commit HEAD của repo gốc không được thay đổi"
    );
    let main_dirty = git_client.status_porcelain(repo_dir.path()).await.unwrap();
    assert!(
        main_dirty.is_empty(),
        "Cây làm việc repo gốc phải sạch hoàn toàn"
    );
}

#[tokio::test]
async fn test_p1_04_worktree_ac2_repo_without_commits_clear_error() {
    let ctx = setup_test_context().await;
    let repo_dir = tempdir().unwrap();
    init_git_repo(repo_dir.path()).await;
    // Chưa tạo commit nào
    let repo_path_str = repo_dir.path().to_string_lossy().to_string();

    let (status, ws_body) = ctx
        .send_request(
            "POST",
            "/api/v1/workspaces",
            &ctx.owner_cookie,
            Some(json!({ "name": "WS P1-04 AC2 Empty Repo" })),
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
                "name": "Proj Empty Git",
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
                "title": "Task trên repo rỗng",
                "description": "Kiểm tra báo lỗi"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let task_id = task_body["task"]["id"].as_str().unwrap();

    let (status, _) = ctx
        .send_request(
            "POST",
            "/api/v1/m/subscription-connect/accounts",
            &ctx.owner_cookie,
            Some(json!({ "provider": "fake", "label": "Fake Agent" })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    ctx.set_account_ready("fake").await;

    // Giao việc -> phải thất bại với thông điệp rõ ràng
    let (status, dispatch_resp) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/tasks/{task_id}/dispatch"),
            &ctx.owner_cookie,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let msg = dispatch_resp["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("Kho git chưa có commit nào"),
        "Phải thông báo lỗi rõ ràng khi repo chưa có commit: {msg}"
    );
}

#[tokio::test]
async fn test_p1_04_worktree_ac3_identical_task_titles_distinct_branches() {
    let ctx = setup_test_context().await;
    let repo_dir = tempdir().unwrap();
    init_git_repo(repo_dir.path()).await;
    commit_sample_file(repo_dir.path()).await;
    let repo_path_str = repo_dir.path().to_string_lossy().to_string();

    let (status, ws_body) = ctx
        .send_request(
            "POST",
            "/api/v1/workspaces",
            &ctx.owner_cookie,
            Some(json!({ "name": "WS P1-04 AC3 Collision" })),
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
                "name": "Proj Collision Test",
                "repo_path": repo_path_str,
                "default_branch": "main"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let proj_id = proj_body["project"]["id"].as_str().unwrap();

    let (status, _) = ctx
        .send_request(
            "POST",
            "/api/v1/m/subscription-connect/accounts",
            &ctx.owner_cookie,
            Some(json!({ "provider": "fake", "label": "Fake Agent" })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    ctx.set_account_ready("fake").await;

    // Tạo Task 1
    let (status, t1_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/projects/{proj_id}/tasks"),
            &ctx.owner_cookie,
            Some(json!({
                "title": "Cùng Tiêu Đề Trùng Nhánh",
                "description": "Mô tả task 1"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let t1_id = t1_body["task"]["id"].as_str().unwrap();

    // Tạo Task 2 có tiêu đề y hệt
    let (status, t2_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/projects/{proj_id}/tasks"),
            &ctx.owner_cookie,
            Some(json!({
                "title": "Cùng Tiêu Đề Trùng Nhánh",
                "description": "Mô tả task 2"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let t2_id = t2_body["task"]["id"].as_str().unwrap();

    let (status, _) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/tasks/{t1_id}/dispatch"),
            &ctx.owner_cookie,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, _) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/tasks/{t2_id}/dispatch"),
            &ctx.owner_cookie,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let (_, wt1) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/m/worktree/tasks/{t1_id}/worktree"),
            &ctx.owner_cookie,
            None,
        )
        .await;
    let (_, wt2) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/m/worktree/tasks/{t2_id}/worktree"),
            &ctx.owner_cookie,
            None,
        )
        .await;

    let b1 = wt1["branch"].as_str().unwrap();
    let b2 = wt2["branch"].as_str().unwrap();
    assert_ne!(
        b1, b2,
        "Hai task cùng tiêu đề phải sinh ra 2 nhánh git khác biệt"
    );
}

#[tokio::test]
async fn test_p1_04_worktree_ac4_release_dirty_rejected_and_force_clean() {
    let ctx = setup_test_context().await;
    let repo_dir = tempdir().unwrap();
    init_git_repo(repo_dir.path()).await;
    commit_sample_file(repo_dir.path()).await;
    let repo_path_str = repo_dir.path().to_string_lossy().to_string();

    let (status, ws_body) = ctx
        .send_request(
            "POST",
            "/api/v1/workspaces",
            &ctx.owner_cookie,
            Some(json!({ "name": "WS P1-04 AC4 Dirty" })),
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
                "name": "Proj Dirty Release",
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
                "title": "Task Kiểm Tra Dirty Release",
                "description": "Mô tả dirty"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let task_id = task_body["task"]["id"].as_str().unwrap();

    let (status, _) = ctx
        .send_request(
            "POST",
            "/api/v1/m/subscription-connect/accounts",
            &ctx.owner_cookie,
            Some(json!({ "provider": "fake", "label": "Fake Agent" })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    ctx.set_account_ready("fake").await;

    // Dispatch để tạo worktree
    let (status, _) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/tasks/{task_id}/dispatch"),
            &ctx.owner_cookie,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let (_, wt_resp) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/m/worktree/tasks/{task_id}/worktree"),
            &ctx.owner_cookie,
            None,
        )
        .await;
    let wt_path = PathBuf::from(wt_resp["workdir_path"].as_str().unwrap());

    // Tạo file sửa đổi chưa commit trong worktree
    fs::write(wt_path.join("uncommitted.rs"), "// dirty file").unwrap();

    // 1. Dọn dẹp với force = false -> Bị từ chối
    let (status, clean_err) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/m/worktree/tasks/{task_id}/cleanup"),
            &ctx.owner_cookie,
            Some(json!({ "force": false })),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(clean_err["error"]["code"], "worktree_dirty");

    // Worktree vẫn tồn tại
    assert!(wt_path.exists());

    // 2. Dọn dẹp với force = true -> Thành công
    let (status, clean_ok) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/m/worktree/tasks/{task_id}/cleanup"),
            &ctx.owner_cookie,
            Some(json!({ "force": true })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(clean_ok["success"], true);

    // Thư mục worktree đã bị dọn dẹp
    assert!(!wt_path.exists());
}

#[tokio::test]
async fn test_p1_04_worktree_ac5_and_ac6_malicious_branch_and_path_traversal() {
    let ctx = setup_test_context().await;
    let repo_dir = tempdir().unwrap();
    init_git_repo(repo_dir.path()).await;
    commit_sample_file(repo_dir.path()).await;
    let repo_path_str = repo_dir.path().to_string_lossy().to_string();

    let (status, ws_body) = ctx
        .send_request(
            "POST",
            "/api/v1/workspaces",
            &ctx.owner_cookie,
            Some(json!({ "name": "WS P1-04 AC5 Security" })),
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
                "name": "Proj Security Test",
                "repo_path": repo_path_str,
                "default_branch": "main"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let proj_id = proj_body["project"]["id"].as_str().unwrap();

    let (status, _) = ctx
        .send_request(
            "POST",
            "/api/v1/m/subscription-connect/accounts",
            &ctx.owner_cookie,
            Some(json!({ "provider": "fake", "label": "Fake Agent" })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    ctx.set_account_ready("fake").await;

    // Task có tiêu đề chứa shell metacharacters và options
    let (status, task_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/projects/{proj_id}/tasks"),
            &ctx.owner_cookie,
            Some(json!({
                "title": "--upload-pack=evil.sh $(whoami) `calc`",
                "description": "Payload test"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let task_id = task_body["task"]["id"].as_str().unwrap();

    let (status, _) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/tasks/{task_id}/dispatch"),
            &ctx.owner_cookie,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let (_, wt_resp) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/m/worktree/tasks/{task_id}/worktree"),
            &ctx.owner_cookie,
            None,
        )
        .await;
    let branch = wt_resp["branch"].as_str().unwrap();
    assert!(branch.starts_with("task/"));
    assert!(!branch.contains('$'));
    assert!(!branch.contains('`'));
    assert!(!branch.contains('('));
    assert!(!branch.contains(')'));
}

#[tokio::test]
async fn test_p1_04_worktree_ac7_agent_writes_file_inside_worktree_not_main_repo() {
    let ctx = setup_test_context().await;
    let repo_dir = tempdir().unwrap();
    init_git_repo(repo_dir.path()).await;
    commit_sample_file(repo_dir.path()).await;
    let repo_path_str = repo_dir.path().to_string_lossy().to_string();

    let (status, ws_body) = ctx
        .send_request(
            "POST",
            "/api/v1/workspaces",
            &ctx.owner_cookie,
            Some(json!({ "name": "WS P1-04 AC7 Integration" })),
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
                "name": "Proj AC7 Workdir",
                "repo_path": repo_path_str,
                "default_branch": "main"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let proj_id = proj_body["project"]["id"].as_str().unwrap();

    let (status, _) = ctx
        .send_request(
            "POST",
            "/api/v1/m/subscription-connect/accounts",
            &ctx.owner_cookie,
            Some(json!({ "provider": "fake", "label": "Fake Agent" })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    ctx.set_account_ready("fake").await;

    // Kịch bản yêu cầu fake-agent ghi file agent_generated.txt
    let scenario_desc = json!({
        "write_file": {
            "path": "agent_generated.txt",
            "content": "Worktree agent content"
        },
        "exit_code": 0
    })
    .to_string();

    let (status, task_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/projects/{proj_id}/tasks"),
            &ctx.owner_cookie,
            Some(json!({
                "title": "Task AC7 Agent File Writing",
                "description": scenario_desc
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let task_id = task_body["task"]["id"].as_str().unwrap();

    let (status, _) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/tasks/{task_id}/dispatch"),
            &ctx.owner_cookie,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    // Chờ một chút để tiến trình con fake-agent hoàn thành ghi file
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;

    let (_, wt_resp) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/m/worktree/tasks/{task_id}/worktree"),
            &ctx.owner_cookie,
            None,
        )
        .await;
    let wt_path_str = wt_resp["workdir_path"].as_str().unwrap();
    let wt_path = PathBuf::from(wt_path_str);

    // 1. Tệp sinh ra NẰM TRONG thư mục worktree
    let agent_file_in_wt = wt_path.join("agent_generated.txt");
    assert!(
        agent_file_in_wt.exists(),
        "Tệp agent_generated.txt phải nằm trong thư mục worktree"
    );
    let content = fs::read_to_string(&agent_file_in_wt).unwrap();
    assert_eq!(content, "Worktree agent content");

    // 2. Tệp KHÔNG ĐƯỢC PHÉP nằm trong cây làm việc repo gốc
    let agent_file_in_main = repo_dir.path().join("agent_generated.txt");
    assert!(
        !agent_file_in_main.exists(),
        "Cây làm việc chính của repo gốc không được phép bị ghi tệp"
    );

    let git_client = GitClient::new(repo_dir.path());
    let main_dirty = git_client.status_porcelain(repo_dir.path()).await.unwrap();
    assert!(
        main_dirty.is_empty(),
        "Cây làm việc repo gốc phải hoàn toàn sạch sau khi agent chạy"
    );
}

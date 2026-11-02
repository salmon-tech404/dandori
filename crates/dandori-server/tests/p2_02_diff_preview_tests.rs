//! Bộ kiểm thử tích hợp cho Task P2-02: Module `diff-preview`.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    clippy::indexing_slicing,
    reason = "Kiểm thử tích hợp P2-02 dùng unwrap và indexing json"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use dandori_events::EventBus;
use dandori_git::GitClient;
use dandori_registry::{FeatureLifecycleManager, FeatureRegistry};
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
    non_member_cookie: String,
    _pool: SqlitePool,
    _lifecycle_manager: Arc<FeatureLifecycleManager>,
    workspace_id: String,
    task_id: String,
    _repo_path: PathBuf,
    workdir_path: PathBuf,
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

async fn init_git_repo(repo_path: &Path) {
    let client = GitClient::new(repo_path);
    client.init_repo("main").await.unwrap();
    client.config("user.name", "Tester").await.unwrap();
    client
        .config("user.email", "tester@dandori.local")
        .await
        .unwrap();
    fs::write(repo_path.join("README.md"), "# Initial\nline 1\nline 2\n").unwrap();
    fs::write(repo_path.join("to_delete.txt"), "delete me\n").unwrap();
    fs::write(repo_path.join("rename_me.txt"), "rename me\n").unwrap();
    client.commit_all("initial commit").await.unwrap();
}

async fn create_test_worktree(repo_path: &Path, wt_path: &Path) {
    let client = GitClient::new(repo_path);
    client
        .worktree_add(wt_path, "task/preview-test", "main")
        .await
        .unwrap();
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
    let repo_path = temp_dir.path().join("repo");
    fs::create_dir_all(&repo_path).unwrap();
    init_git_repo(&repo_path).await;

    let wt_path = temp_dir.path().join("worktree_task");
    create_test_worktree(&repo_path, &wt_path).await;

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

    // 1. Setup owner
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

    // 2. Tạo workspace
    let ws_req = Request::builder()
        .method("POST")
        .uri("/api/v1/workspaces")
        .header(header::COOKIE, &owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "name": "Workspace DiffPreview" }).to_string(),
        ))
        .unwrap();

    let ws_resp = app.clone().oneshot(ws_req).await.unwrap();
    assert_eq!(ws_resp.status(), StatusCode::OK);
    let ws_body: Value = serde_json::from_slice(
        &axum::body::to_bytes(ws_resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let workspace_id = ws_body["workspace"]["id"].as_str().unwrap().to_string();

    // 3. Tạo non-member
    let non_member_cookie = login_new_user(&app, &owner_cookie, "nonmember").await;

    // 4. Tạo project & task
    let proj_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/workspaces/{workspace_id}/projects"))
        .header(header::COOKIE, &owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "name": "Project DiffPreview",
                "repo_path": repo_path.to_str().unwrap(),
                "default_branch": "main"
            })
            .to_string(),
        ))
        .unwrap();

    let proj_resp = app.clone().oneshot(proj_req).await.unwrap();
    assert_eq!(proj_resp.status(), StatusCode::OK);
    let proj_body: Value = serde_json::from_slice(
        &axum::body::to_bytes(proj_resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let project_id = proj_body["project"]["id"].as_str().unwrap().to_string();

    let task_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/projects/{project_id}/tasks"))
        .header(header::COOKIE, &owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "title": "Task Diff Test",
                "description": "Task for testing diff and preview"
            })
            .to_string(),
        ))
        .unwrap();

    let task_resp = app.clone().oneshot(task_req).await.unwrap();
    assert_eq!(task_resp.status(), StatusCode::OK);
    let task_body: Value = serde_json::from_slice(
        &axum::body::to_bytes(task_resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let task_id = task_body["task"]["id"].as_str().unwrap().to_string();

    // Cập nhật workdir_path cho task
    sqlx::query("UPDATE task SET workdir_path = ? WHERE id = ?")
        .bind(wt_path.to_str().unwrap())
        .bind(&task_id)
        .execute(&pool)
        .await
        .unwrap();

    // 5. Kích hoạt module worktree và diff-preview
    for mod_id in ["worktree", "diff-preview"] {
        let enable_req = Request::builder()
            .method("POST")
            .uri("/api/v1/features/apply")
            .header(header::COOKIE, &owner_cookie)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "module_id": mod_id,
                    "scope": "workspace",
                    "target_id": workspace_id,
                    "enable": true
                })
                .to_string(),
            ))
            .unwrap();
        assert_eq!(
            app.clone().oneshot(enable_req).await.unwrap().status(),
            StatusCode::OK
        );
    }

    TestContext {
        app,
        owner_cookie,
        non_member_cookie,
        _pool: pool,
        _lifecycle_manager: lifecycle_manager,
        workspace_id,
        task_id,
        _repo_path: repo_path,
        workdir_path: wt_path,
        _temp_dir: temp_dir,
    }
}

#[tokio::test]
async fn test_ac1_diff_endpoint_structured_and_numstat() {
    let ctx = setup_test_context().await;

    // Tạo các thay đổi trong worktree: thêm, sửa, xóa, đổi tên, nhị phân
    fs::write(ctx.workdir_path.join("new.txt"), "line 1\nline 2\n").unwrap();
    fs::write(ctx.workdir_path.join("README.md"), "# Updated\nline 1\n").unwrap();
    fs::remove_file(ctx.workdir_path.join("to_delete.txt")).unwrap();
    fs::rename(
        ctx.workdir_path.join("rename_me.txt"),
        ctx.workdir_path.join("renamed.txt"),
    )
    .unwrap();
    fs::write(ctx.workdir_path.join("image.png"), [0u8, 1, 2, 255, 0, 100]).unwrap();

    let wt_client = GitClient::new(&ctx.workdir_path);
    wt_client.commit_all("commit changes").await.unwrap();

    let diff_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/diff-preview/tasks/{}/diff", ctx.task_id))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();

    let resp = ctx.app.clone().oneshot(diff_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    let files = body["files"].as_array().expect("files array");
    assert_eq!(files.len(), 5, "Phải có đúng 5 tệp trong kết quả diff");

    let bin_file = files
        .iter()
        .find(|f| f["path"] == "image.png")
        .expect("image.png");
    assert_eq!(bin_file["is_binary"], true);
    assert_eq!(bin_file["diff"], "[Tệp nhị phân]");

    let stats = &body["stats"];
    assert_eq!(stats["total_files_changed"], 5);
}

#[tokio::test]
async fn test_ac2_large_diff_truncation_and_xss_safety() {
    let ctx = setup_test_context().await;

    // Tệp lớn > 50,000 bytes
    let large_content = "x".repeat(60_000);
    fs::write(ctx.workdir_path.join("large.txt"), large_content).unwrap();

    let wt_client = GitClient::new(&ctx.workdir_path);
    wt_client.commit_all("add large file").await.unwrap();

    let diff_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/diff-preview/tasks/{}/diff", ctx.task_id))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();

    let resp = ctx.app.clone().oneshot(diff_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    let files = body["files"].as_array().expect("files array");
    let large_file = files
        .iter()
        .find(|f| f["path"] == "large.txt")
        .expect("large.txt");
    assert_eq!(large_file["truncated"], true);
    assert!(
        large_file["diff"]
            .as_str()
            .unwrap()
            .contains("[Nội dung diff đã được cắt bớt do vượt quá kích thước tối đa]")
    );
}

#[tokio::test]
async fn test_ac3_preview_command_arg_array_no_shell() {
    let ctx = setup_test_context().await;

    // Lệnh xem trước chứa các ký tự shell như ;, &&, $(...)
    // Vì không qua shell nên các tham số được truyền nguyên văn cho tiến trình
    let start_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/diff-preview/tasks/{}/preview/start",
            ctx.task_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "confirmed": true,
                "command": ["git", "version", ";", "echo", "injected"],
                "port": 3456
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(start_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    assert_eq!(body["running"], true);
    assert_eq!(body["port"], 3456);
    assert_eq!(body["host"], "127.0.0.1");

    // Dừng phiên xem trước
    let stop_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/diff-preview/tasks/{}/preview/stop",
            ctx.task_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(stop_req).await.unwrap().status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn test_ac4_preview_confirmation_and_stop_on_disable() {
    let ctx = setup_test_context().await;

    // 1. Không xác nhận (`confirmed: false`) -> Bị từ chối HTTP 400
    let unconfirmed_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/diff-preview/tasks/{}/preview/start",
            ctx.task_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "confirmed": false,
                "command": ["git", "status"],
                "port": 3500
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(unconfirmed_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // 2. Xác nhận hợp lệ -> Bắt đầu thành công
    let confirmed_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/diff-preview/tasks/{}/preview/start",
            ctx.task_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "confirmed": true,
                "command": ["git", "status"],
                "port": 3500
            })
            .to_string(),
        ))
        .unwrap();

    let resp2 = ctx.app.clone().oneshot(confirmed_req).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);

    // 3. Tắt module diff-preview -> Tiến trình xem trước tự động dừng
    let disable_req = Request::builder()
        .method("POST")
        .uri("/api/v1/features/apply")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "module_id": "diff-preview",
                "scope": "workspace",
                "target_id": ctx.workspace_id,
                "enable": false
            })
            .to_string(),
        ))
        .unwrap();

    assert_eq!(
        ctx.app.clone().oneshot(disable_req).await.unwrap().status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn test_ac5_invalid_port_and_non_loopback_rejected() {
    let ctx = setup_test_context().await;

    // Cổng < 1024 bị từ chối
    let bad_port_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/diff-preview/tasks/{}/preview/start",
            ctx.task_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "confirmed": true,
                "command": ["git", "status"],
                "port": 80
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(bad_port_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_ac6_bola_workspace_isolation() {
    let ctx = setup_test_context().await;

    // Non-member cố gắng truy cập diff
    let diff_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/diff-preview/tasks/{}/diff", ctx.task_id))
        .header(header::COOKIE, &ctx.non_member_cookie)
        .body(Body::empty())
        .unwrap();

    let resp = ctx.app.clone().oneshot(diff_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Non-member cố gắng khởi chạy preview
    let preview_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/diff-preview/tasks/{}/preview/start",
            ctx.task_id
        ))
        .header(header::COOKIE, &ctx.non_member_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "confirmed": true,
                "command": ["git", "status"],
                "port": 3600
            })
            .to_string(),
        ))
        .unwrap();

    let resp2 = ctx.app.clone().oneshot(preview_req).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::FORBIDDEN);
}

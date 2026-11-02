//! Các bài kiểm thử tích hợp cho Task P3-09: Module `instruction-sync` và điểm mở rộng `WorkdirDecorator`.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P3-09 dùng unwrap và kiểm tra nhiều tiêu chí nghiệm thu"
)]

use std::fs;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use dandori_events::EventBus;
use dandori_git::GitClient;
use dandori_module_api::{AgentRole, DecoratorContext, TaskRunner};
use dandori_registry::FeatureRegistry;
use dandori_registry::lifecycle::FeatureLifecycleManager;
use dandori_server::app::create_app;
use dandori_server::config::ServerConfig;
use dandori_server::run_service::RunService;
use dandori_server::state::AppState;
use dandori_types::{ProjectId, WorkspaceId};
use serde_json::json;
use sqlx::SqlitePool;
use tempfile::tempdir;
use tower::ServiceExt;

struct TestContext {
    app: axum::Router,
    lifecycle_manager: Arc<FeatureLifecycleManager>,
    owner_cookie: String,
    member_cookie: String,
    outsider_cookie: String,
    workspace_id: String,
    project_id: String,
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
    let run_service = Arc::new(RunService::new(
        pool.clone(),
        Arc::clone(&event_bus),
        Arc::clone(&lifecycle_manager),
        10,
    ));
    let fake_adapter = Arc::new(dandori_agent_adapters::FakeAdapter::new(
        std::path::PathBuf::from("fake-agent"),
    ));
    run_service
        .register_adapter("fake", fake_adapter.clone())
        .await;

    let runner: Arc<dyn TaskRunner> = run_service.clone();
    lifecycle_manager.set_task_runner(runner);

    let state = AppState::new_with_run_service(
        pool.clone(),
        Arc::clone(&event_bus),
        Arc::clone(&lifecycle_manager),
        config,
        run_service.clone(),
    );
    let app = create_app(state);

    // 1. Setup Owner (Admin)
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
        .split(';')
        .next()
        .unwrap()
        .to_string();

    let now = chrono::Utc::now().to_rfc3339();

    // 2. Setup Member & Outsider
    let member_user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, 'member', 'hash', 'user', ?)")
        .bind(&member_user_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let member_token = "member_token_instr";
    let member_token_hash = dandori_server::auth::hash_token(member_token);
    let expires = (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339();
    sqlx::query(
        "INSERT INTO session (id, user_id, expires_at, token_hash) VALUES ('sess_mem_instr', ?, ?, ?)",
    )
    .bind(&member_user_id)
    .bind(&expires)
    .bind(&member_token_hash)
    .execute(&pool)
    .await
    .unwrap();
    let member_cookie = format!("dandori_session={member_token}");

    let outsider_user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, 'outsider', 'hash', 'user', ?)")
        .bind(&outsider_user_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();
    let outsider_token = "outsider_token_instr";
    let outsider_token_hash = dandori_server::auth::hash_token(outsider_token);
    sqlx::query(
        "INSERT INTO session (id, user_id, expires_at, token_hash) VALUES ('sess_out_instr', ?, ?, ?)",
    )
    .bind(&outsider_user_id)
    .bind(&expires)
    .bind(&outsider_token_hash)
    .execute(&pool)
    .await
    .unwrap();
    let outsider_cookie = format!("dandori_session={outsider_token}");

    // Tạo workspace và project
    let workspace_id = WorkspaceId::new().to_string();
    let project_id = ProjectId::new().to_string();
    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'Workspace 1', ?)")
        .bind(&workspace_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'Project 1', '/tmp/repo', 'main', ?)")
        .bind(&project_id)
        .bind(&workspace_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    // Thêm member vào workspace
    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'member')",
    )
    .bind(&workspace_id)
    .bind(&member_user_id)
    .execute(&pool)
    .await
    .unwrap();

    // Kích hoạt các module phụ thuộc và module instruction-sync
    for mod_id in &[
        "lead-worker",
        "tech-profile",
        "prompt-library",
        "skills-md",
        "instruction-sync",
    ] {
        let apply_req = Request::builder()
            .method("POST")
            .uri("/api/v1/features/apply")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::COOKIE, &owner_cookie)
            .body(Body::from(
                json!({
                    "module_id": mod_id,
                    "scope": "workspace",
                    "target_id": workspace_id.clone(),
                    "enable": true,
                    "risk_acknowledged": true
                })
                .to_string(),
            ))
            .unwrap();
        let res = app.clone().oneshot(apply_req).await.unwrap();
        let status = res.status();
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert_eq!(status, StatusCode::OK, "apply {mod_id} failed: {body_str}");
    }

    TestContext {
        app,
        lifecycle_manager,
        owner_cookie,
        member_cookie,
        outsider_cookie,
        workspace_id,
        project_id,
        temp_dir,
    }
}

#[tokio::test]
async fn test_instruction_sync_authorization_and_bola() {
    let ctx = setup_test_context().await;

    // 1. Người ngoài (Outsider) xem cấu hình -> 403 Forbidden
    let req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/instruction-sync/workspaces/{}/projects/{}/config",
            ctx.workspace_id, ctx.project_id
        ))
        .header(header::COOKIE, &ctx.outsider_cookie)
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 2. Thành viên (Member) xem cấu hình -> 200 OK
    let req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/instruction-sync/workspaces/{}/projects/{}/config",
            ctx.workspace_id, ctx.project_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 3. Thành viên không phải admin sửa cấu hình -> 403 Forbidden
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/instruction-sync/workspaces/{}/projects/{}/config",
            ctx.workspace_id, ctx.project_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "claude",
                "target_filename": "CLAUDE.md",
                "strategy": "skip",
                "is_enabled": true
            })
            .to_string(),
        ))
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 4. Quản trị viên (Admin) sửa cấu hình -> 200 OK
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/instruction-sync/workspaces/{}/projects/{}/config",
            ctx.workspace_id, ctx.project_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "claude",
                "target_filename": "CLAUDE.md",
                "strategy": "skip",
                "is_enabled": true
            })
            .to_string(),
        ))
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 5. Outsider gọi xem trước -> 403 Forbidden
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/instruction-sync/workspaces/{}/preview",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.outsider_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "project_id": ctx.project_id,
                "provider": "claude"
            })
            .to_string(),
        ))
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 6. Member gọi xem trước -> 200 OK
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/instruction-sync/workspaces/{}/preview",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "project_id": ctx.project_id,
                "provider": "claude"
            })
            .to_string(),
        ))
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_ac1_whitelist_and_path_traversal_rejection() {
    let ctx = setup_test_context().await;

    // Cố tình cấu hình tệp không nằm trong whitelist -> 400 Bad Request
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/instruction-sync/workspaces/{}/projects/{}/config",
            ctx.workspace_id, ctx.project_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "claude",
                "target_filename": "evil_script.sh",
                "strategy": "skip",
                "is_enabled": true
            })
            .to_string(),
        ))
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // Cố tình cấu hình path traversal có .. -> 400 Bad Request
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/instruction-sync/workspaces/{}/projects/{}/config",
            ctx.workspace_id, ctx.project_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "claude",
                "target_filename": "../CLAUDE.md",
                "strategy": "skip",
                "is_enabled": true
            })
            .to_string(),
        ))
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_ac2_and_ac3_git_exclude_and_skip_append_behavior() {
    let ctx = setup_test_context().await;

    // Khởi tạo một git repository thật trong thư mục tạm
    let repo_dir = tempdir().unwrap();
    let repo_path = repo_dir.path();

    let git_client = GitClient::new(repo_path);
    git_client.init_repo("main").await.unwrap();
    git_client.config("user.name", "Test User").await.unwrap();
    git_client
        .config("user.email", "test@dandori.local")
        .await
        .unwrap();

    // Thêm commit khởi đầu để git status và rev-parse hoạt động chuẩn
    fs::write(repo_path.join("README.md"), "# Initial Repo\n").unwrap();
    git_client.commit_all("Initial commit").await.unwrap();

    // Lấy decorator từ lifecycle manager
    let decorators = ctx.lifecycle_manager.get_workdir_decorators().await;
    assert!(!decorators.is_empty(), "Decorator phải được đăng ký");
    let decorator = &decorators[0];

    let d_ctx = DecoratorContext {
        workspace_id: ctx.workspace_id.clone(),
        project_id: ctx.project_id.clone(),
        task_id: "task-test-1".to_string(),
        role: AgentRole::Worker,
        provider: Some("claude".to_string()),
    };

    // --- Kịch bản 1: Chưa có tệp CLAUDE.md trong repo ---
    let result = decorator.decorate(repo_path, &d_ctx).await.unwrap();
    assert_eq!(result.created_files, vec!["CLAUDE.md".to_string()]);
    assert!(repo_path.join("CLAUDE.md").exists());

    // AC2: Kiểm tra git status --porcelain: Tệp CLAUDE.md KHÔNG xuất hiện vì đã được exclude
    let status_entries = git_client.status_porcelain(repo_path).await.unwrap();
    assert!(
        !status_entries.iter().any(|s| s.contains("CLAUDE.md")),
        "CLAUDE.md phải bị loại trừ khỏi git status: {:?}",
        status_entries
    );

    // --- Kịch bản 2: Đã có tệp AGENTS.md thật của dự án với nội dung gốc ---
    let original_agents_content = "# Original AGENTS.md by human\nDo not overwrite me!\n";
    fs::write(repo_path.join("AGENTS.md"), original_agents_content).unwrap();

    let d_ctx_codex = DecoratorContext {
        workspace_id: ctx.workspace_id.clone(),
        project_id: ctx.project_id.clone(),
        task_id: "task-test-2".to_string(),
        role: AgentRole::Worker,
        provider: Some("codex".to_string()),
    };

    // Mặc định: Strategy là 'skip' -> Không ghi đè
    let result_skip = decorator.decorate(repo_path, &d_ctx_codex).await.unwrap();
    assert_eq!(result_skip.skipped_files, vec!["AGENTS.md".to_string()]);

    let after_skip_content = fs::read_to_string(repo_path.join("AGENTS.md")).unwrap();
    // AC3: Nội dung nguyên vẹn không đổi
    assert_eq!(original_agents_content, after_skip_content);

    // Cấu hình sang 'append'
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/instruction-sync/workspaces/{}/projects/{}/config",
            ctx.workspace_id, ctx.project_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "codex",
                "target_filename": "AGENTS.md",
                "strategy": "append",
                "is_enabled": true
            })
            .to_string(),
        ))
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Chạy lại decorator với 'append'
    let result_append = decorator.decorate(repo_path, &d_ctx_codex).await.unwrap();
    assert_eq!(result_append.appended_files, vec!["AGENTS.md".to_string()]);

    let appended_content = fs::read_to_string(repo_path.join("AGENTS.md")).unwrap();
    assert!(appended_content.starts_with("# Original AGENTS.md by human"));
    assert!(appended_content.contains("<!-- BEGIN DANDORI INSTRUCTION SYNC -->"));
    assert!(appended_content.contains("<!-- END DANDORI INSTRUCTION SYNC -->"));
}

#[tokio::test]
async fn test_ac5_module_disable_removes_decorator() {
    let ctx = setup_test_context().await;

    // Tắt module instruction-sync thông qua API apply
    let apply_req = Request::builder()
        .method("POST")
        .uri("/api/v1/features/apply")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::from(
            json!({
                "module_id": "instruction-sync",
                "scope": "workspace",
                "target_id": ctx.workspace_id,
                "enable": false
            })
            .to_string(),
        ))
        .unwrap();
    let res = ctx.app.clone().oneshot(apply_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let active_decorators = ctx.lifecycle_manager.get_workdir_decorators().await;
    let has_instruction_sync = active_decorators
        .iter()
        .any(|d| d.allowed_filenames().contains(&"CLAUDE.md"));
    assert!(
        !has_instruction_sync,
        "Decorator phải được gỡ bỏ khi module tắt"
    );
}

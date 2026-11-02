//! Bộ kiểm thử tích hợp cho Task P2-05: Module `doc-agent`.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    clippy::indexing_slicing,
    reason = "Kiểm thử tích hợp P2-05 dùng unwrap và indexing json"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use dandori_events::EventBus;
use dandori_git::GitClient;
use dandori_module_api::{BoxFuture, ReleaseReason, Workdir, WorkdirError, WorkdirProvider};
use dandori_registry::{FeatureLifecycleManager, FeatureRegistry, RegistryRole, RegistryScope};
use dandori_server::app::create_app;
use dandori_server::config::ServerConfig;
use dandori_server::state::AppState;
use dandori_types::{
    Envelope, EventPayload, EventScope, ProjectId, RunId, Task, TaskId, WorkspaceId,
};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tempfile::tempdir;
use tower::ServiceExt;

#[derive(Debug, Clone)]
struct MockWorkdirProvider {
    workdir_path: PathBuf,
}

impl WorkdirProvider for MockWorkdirProvider {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn prepare(&self, _task: &Task) -> BoxFuture<'static, Result<Workdir, WorkdirError>> {
        let p = self.workdir_path.clone();
        Box::pin(async move { Ok(Workdir::new(p)) })
    }

    fn release(
        &self,
        _task: &Task,
        _reason: ReleaseReason,
    ) -> BoxFuture<'static, Result<(), WorkdirError>> {
        Box::pin(async move { Ok(()) })
    }
}

fn fake_agent_bin_path() -> PathBuf {
    let target_debug = PathBuf::from("target/debug").join(if cfg!(windows) {
        "fake-agent.exe"
    } else {
        "fake-agent"
    });
    if target_debug.exists() {
        return target_debug;
    }
    PathBuf::from(if cfg!(windows) {
        "fake-agent.exe"
    } else {
        "fake-agent"
    })
}

struct TestContext {
    app: axum::Router,
    owner_cookie: String,
    non_member_cookie: String,
    pool: SqlitePool,
    lifecycle_manager: Arc<FeatureLifecycleManager>,
    event_bus: Arc<EventBus>,
    workspace_id: String,
    project_id: String,
    task_id: String,
    repo_path: PathBuf,
    _temp_dir: tempfile::TempDir,
}

async fn init_git_repo(repo_path: &Path) {
    let client = GitClient::new(repo_path);
    client.init_repo("main").await.unwrap();
    client.config("user.name", "Tester").await.unwrap();
    client
        .config("user.email", "tester@dandori.local")
        .await
        .unwrap();
    fs::write(repo_path.join("README.md"), "# Initial\n").unwrap();
    client.commit_all("initial commit").await.unwrap();
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

    lifecycle_manager
        .register_workdir_provider_for_test(Arc::new(MockWorkdirProvider {
            workdir_path: repo_path.clone(),
        }))
        .await;

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

    let fake_bin = fake_agent_bin_path();
    let fake_adapter = Arc::new(dandori_agent_adapters::FakeAdapter::new(fake_bin));
    state
        .run_service
        .register_adapter("fake", fake_adapter.clone())
        .await;
    state
        .run_service
        .register_adapter("fake-agent", fake_adapter)
        .await;

    let fake_acc_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO agent_account (id, provider, label, home_dir, auth_type, status, enabled, priority) \
         VALUES (?, 'fake', 'Fake Agent', '', 'cli', 'ready', 1, 10)",
    )
    .bind(&fake_acc_id)
    .execute(&pool)
    .await
    .unwrap();

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
            json!({ "name": "Workspace Doc Agent" }).to_string(),
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

    let prj_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/workspaces/{workspace_id}/projects"))
        .header(header::COOKIE, &owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "name": "Project Doc Agent",
                "repo_path": repo_path.to_str().unwrap(),
                "default_branch": "main"
            })
            .to_string(),
        ))
        .unwrap();
    let prj_resp = app.clone().oneshot(prj_req).await.unwrap();
    let prj_body: Value = serde_json::from_slice(
        &axum::body::to_bytes(prj_resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let project_id = prj_body["project"]["id"].as_str().unwrap().to_string();

    let task_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/projects/{project_id}/tasks"))
        .header(header::COOKIE, &owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "title": "Nhiệm vụ kiểm thử doc agent",
                "description": "Mô tả nhiệm vụ kiểm thử"
            })
            .to_string(),
        ))
        .unwrap();
    let task_resp = app.clone().oneshot(task_req).await.unwrap();
    let task_body: Value = serde_json::from_slice(
        &axum::body::to_bytes(task_resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let task_id = task_body["task"]["id"].as_str().unwrap().to_string();

    // Khởi tạo workdir_path cho task bằng chính repo_path
    let now_str = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE task SET workdir_path = ?, status = 'in_review', updated_at = ? WHERE id = ?",
    )
    .bind(repo_path.to_str().unwrap())
    .bind(&now_str)
    .bind(&task_id)
    .execute(&pool)
    .await
    .unwrap();

    lifecycle_manager
        .enable(
            "doc-agent",
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
        event_bus,
        workspace_id,
        project_id,
        task_id,
        repo_path,
        _temp_dir: temp_dir,
    }
}

#[tokio::test]
async fn test_ac1_ac2_ready_to_merge_triggers_doc_agent_once_anti_loop() {
    let ctx = setup_test_context().await;
    let ws_id = ctx.workspace_id.parse::<WorkspaceId>().unwrap();
    let prj_id = ctx.project_id.parse::<ProjectId>().unwrap();
    let t_id = ctx.task_id.parse::<TaskId>().unwrap();

    // 1. Chuyển sang in_review -> KHÔNG kích hoạt doc agent
    ctx.event_bus.publish(Envelope::new(
        chrono::Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::TaskStatusChanged {
            task_id: t_id,
            old_status: "running".to_string(),
            new_status: "in_review".to_string(),
        },
    ));
    tokio::time::sleep(Duration::from_millis(200)).await;

    let runs_in_review: (i64,) =
        sqlx::query_as("SELECT count(*) FROM m_doc_agent_run WHERE task_id = ?")
            .bind(&ctx.task_id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(
        runs_in_review.0, 0,
        "in_review không được kích hoạt doc agent"
    );

    // 2. Chuyển sang ready_to_merge -> KÍCH HOẠT doc agent đúng một lần
    ctx.event_bus.publish(Envelope::new(
        chrono::Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::TaskStatusChanged {
            task_id: t_id,
            old_status: "in_review".to_string(),
            new_status: "ready_to_merge".to_string(),
        },
    ));
    tokio::time::sleep(Duration::from_millis(300)).await;

    let doc_runs: Vec<(String, String)> =
        sqlx::query_as("SELECT id, status FROM m_doc_agent_run WHERE task_id = ?")
            .bind(&ctx.task_id)
            .fetch_all(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(
        doc_runs.len(),
        1,
        "ready_to_merge kích hoạt doc agent đúng 1 lần"
    );
    assert!(doc_runs[0].1 == "running" || doc_runs[0].1 == "completed");

    // 3. AC2: Doc agent kết thúc (RunFinished) -> KHÔNG kích hoạt doc agent lần nữa
    let (run_id_str,): (String,) =
        sqlx::query_as("SELECT run_id FROM m_doc_agent_run WHERE task_id = ?")
            .bind(&ctx.task_id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    let run_id = run_id_str.parse::<RunId>().unwrap();

    ctx.event_bus.publish(Envelope::new(
        chrono::Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::RunFinished {
            run_id,
            exit_code: Some(0),
        },
    ));
    tokio::time::sleep(Duration::from_millis(300)).await;

    let doc_runs_after: (i64,) =
        sqlx::query_as("SELECT count(*) FROM m_doc_agent_run WHERE task_id = ?")
            .bind(&ctx.task_id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(
        doc_runs_after.0, 1,
        "RunFinished không được tạo thêm doc run mới (chống loop)"
    );
}

#[tokio::test]
async fn test_ac3_doc_agent_flags_unallowed_files() {
    let ctx = setup_test_context().await;
    let ws_id = ctx.workspace_id.parse::<WorkspaceId>().unwrap();
    let prj_id = ctx.project_id.parse::<ProjectId>().unwrap();
    let t_id = ctx.task_id.parse::<TaskId>().unwrap();

    // Chuẩn bị trước các file sửa đổi trong workdir của task
    fs::create_dir_all(ctx.repo_path.join("docs")).unwrap();
    fs::write(ctx.repo_path.join("docs/api.md"), "# API Docs\n").unwrap();
    fs::create_dir_all(ctx.repo_path.join("src")).unwrap();
    fs::write(ctx.repo_path.join("src/main.rs"), "fn main() {}\n").unwrap();

    // Kích hoạt doc agent chạy
    ctx.event_bus.publish(Envelope::new(
        chrono::Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::TaskStatusChanged {
            task_id: t_id,
            old_status: "in_review".to_string(),
            new_status: "ready_to_merge".to_string(),
        },
    ));
    // Đợi worker chạy xong (fake-agent kết thúc và trigger RunFinished)
    tokio::time::sleep(Duration::from_millis(600)).await;

    let (status, changed_docs, flagged): (String, String, String) = sqlx::query_as(
        "SELECT status, changed_doc_files, flagged_files FROM m_doc_agent_run WHERE task_id = ?",
    )
    .bind(&ctx.task_id)
    .fetch_one(&ctx.pool)
    .await
    .unwrap();

    assert_eq!(status, "completed");
    assert!(
        changed_docs.contains("docs/api.md") || changed_docs.contains("docs\\api.md"),
        "docs/api.md phải được ghi nhận là tệp tài liệu hợp lệ, thực tế: {changed_docs}"
    );
    assert!(
        flagged.contains("src/main.rs") || flagged.contains("src\\main.rs"),
        "src/main.rs phải bị gắn cờ cảnh báo ngoài phạm vi, thực tế: {flagged}"
    );
}

#[tokio::test]
async fn test_ac4_disabled_module_does_not_trigger() {
    let ctx = setup_test_context().await;
    let ws_id = ctx.workspace_id.parse::<WorkspaceId>().unwrap();
    let prj_id = ctx.project_id.parse::<ProjectId>().unwrap();
    let t_id = ctx.task_id.parse::<TaskId>().unwrap();

    // Tắt module doc-agent
    ctx.lifecycle_manager
        .disable(
            "doc-agent",
            &RegistryScope::System,
            RegistryRole::SystemOwner,
            "owner",
            true,
        )
        .await
        .unwrap();

    ctx.event_bus.publish(Envelope::new(
        chrono::Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::TaskStatusChanged {
            task_id: t_id,
            old_status: "in_review".to_string(),
            new_status: "ready_to_merge".to_string(),
        },
    ));
    tokio::time::sleep(Duration::from_millis(300)).await;

    let count: (i64,) = sqlx::query_as("SELECT count(*) FROM m_doc_agent_run WHERE task_id = ?")
        .bind(&ctx.task_id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    assert_eq!(
        count.0, 0,
        "Khi tắt module doc-agent, không được kích hoạt chạy"
    );
}

#[tokio::test]
async fn test_ac5_workspace_isolation_bola() {
    let ctx = setup_test_context().await;

    // Thành viên sở hữu lấy cấu hình thành công (200 OK)
    let get_owner = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/doc-agent/projects/{}/config",
            ctx.project_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp_owner = ctx.app.clone().oneshot(get_owner).await.unwrap();
    assert_eq!(resp_owner.status(), StatusCode::OK);

    // Người dùng ngoài workspace lấy cấu hình -> Bị từ chối (403 Forbidden hoặc 404)
    let get_non_member = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/doc-agent/projects/{}/config",
            ctx.project_id
        ))
        .header(header::COOKIE, &ctx.non_member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp_non_member = ctx.app.clone().oneshot(get_non_member).await.unwrap();
    assert!(
        resp_non_member.status() == StatusCode::FORBIDDEN
            || resp_non_member.status() == StatusCode::NOT_FOUND,
        "Người dùng không thuộc workspace phải bị chặn truy cập cấu hình doc agent"
    );
}

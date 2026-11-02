//! Bộ kiểm thử tích hợp cho Task P2-04: Module `test-agent`.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    clippy::indexing_slicing,
    reason = "Kiểm thử tích hợp P2-04 dùng unwrap và indexing json"
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
    Envelope, EventPayload, EventScope, ProjectId, RunId, Task, TaskId, TaskStatus, WorkspaceId,
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
            json!({ "name": "Workspace Test Agent" }).to_string(),
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
                "name": "Project Test Agent",
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
                "title": "Nhiệm vụ kiểm thử test agent",
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
            "test-agent",
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
async fn test_ac1_ac2_worker_triggers_test_agent_once_anti_loop() {
    let ctx = setup_test_context().await;

    // Lấy config ban đầu
    let get_cfg = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/test-agent/projects/{}/config",
            ctx.project_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(get_cfg).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Kích hoạt worker hoàn thành
    let worker_run_id = RunId::new();
    let ws_id = ctx.workspace_id.parse::<WorkspaceId>().unwrap();
    let prj_id = ctx.project_id.parse::<ProjectId>().unwrap();
    let t_id = ctx.task_id.parse::<TaskId>().unwrap();

    ctx.event_bus.publish(Envelope::new(
        chrono::Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::RunFinished {
            run_id: worker_run_id,
            exit_code: Some(0),
        },
    ));

    // Đợi worker xử lý
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Kiểm tra đã tạo test run record
    let runs: Vec<(String, String)> =
        sqlx::query_as("SELECT status, trigger_run_id FROM m_test_agent_run WHERE task_id = ?")
            .bind(&ctx.task_id)
            .fetch_all(&ctx.pool)
            .await
            .unwrap();

    assert_eq!(runs.len(), 1);
    assert!(runs[0].0 == "running" || runs[0].0 == "passed");
    assert_eq!(runs[0].1, worker_run_id.to_string());

    // AC2: Test run hoàn thành KHÔNG được kích hoạt thêm test run mới
    let (test_run_id,): (String,) =
        sqlx::query_as("SELECT run_id FROM m_test_agent_run WHERE task_id = ?")
            .bind(&ctx.task_id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

    let t_run_id = test_run_id.parse::<RunId>().unwrap();
    ctx.event_bus.publish(Envelope::new(
        chrono::Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::RunFinished {
            run_id: t_run_id,
            exit_code: Some(0),
        },
    ));

    tokio::time::sleep(Duration::from_millis(300)).await;

    let runs_after: (i64,) =
        sqlx::query_as("SELECT count(*) FROM m_test_agent_run WHERE task_id = ?")
            .bind(&ctx.task_id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

    assert_eq!(
        runs_after.0, 1,
        "Test run hoàn thành không được tạo thêm run mới (chống loop)"
    );
}

#[tokio::test]
async fn test_ac3_test_failure_transitions_to_needs_human() {
    let ctx = setup_test_context().await;
    let ws_id = ctx.workspace_id.parse::<WorkspaceId>().unwrap();
    let prj_id = ctx.project_id.parse::<ProjectId>().unwrap();
    let t_id = ctx.task_id.parse::<TaskId>().unwrap();

    // Giả lập worker hoàn thành
    let worker_run_id = RunId::new();
    ctx.event_bus.publish(Envelope::new(
        chrono::Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::RunFinished {
            run_id: worker_run_id,
            exit_code: Some(0),
        },
    ));
    tokio::time::sleep(Duration::from_millis(300)).await;

    let (test_run_id,): (String,) =
        sqlx::query_as("SELECT run_id FROM m_test_agent_run WHERE task_id = ?")
            .bind(&ctx.task_id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

    // Test agent kết thúc thất bại (exit_code = 1)
    let t_run_id = test_run_id.parse::<RunId>().unwrap();
    ctx.event_bus.publish(Envelope::new(
        chrono::Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::RunFinished {
            run_id: t_run_id,
            exit_code: Some(1),
        },
    ));
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Kiểm tra task chuyển thành needs_human
    let (task_status,): (String,) = sqlx::query_as("SELECT status FROM task WHERE id = ?")
        .bind(&ctx.task_id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(task_status, TaskStatus::NeedsHuman.as_str());

    // Kiểm tra bản ghi test run chuyển thành failed
    let (test_status, exit_code): (String, Option<i64>) =
        sqlx::query_as("SELECT status, exit_code FROM m_test_agent_run WHERE run_id = ?")
            .bind(&test_run_id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

    assert_eq!(test_status, "failed");
    assert_eq!(exit_code, Some(1));
}

#[tokio::test]
async fn test_ac4_flagged_files_outside_pattern() {
    let ctx = setup_test_context().await;
    let ws_id = ctx.workspace_id.parse::<WorkspaceId>().unwrap();
    let prj_id = ctx.project_id.parse::<ProjectId>().unwrap();
    let t_id = ctx.task_id.parse::<TaskId>().unwrap();

    // Tạo một file sửa đổi nằm ngoài pattern (production.rs)
    let bad_file = ctx.repo_path.join("production.rs");
    fs::write(&bad_file, "// modified by test agent\n").unwrap();

    // Giả lập worker xong
    let worker_run_id = RunId::new();
    ctx.event_bus.publish(Envelope::new(
        chrono::Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::RunFinished {
            run_id: worker_run_id,
            exit_code: Some(0),
        },
    ));
    tokio::time::sleep(Duration::from_millis(300)).await;

    let (test_run_id,): (String,) =
        sqlx::query_as("SELECT run_id FROM m_test_agent_run WHERE task_id = ?")
            .bind(&ctx.task_id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

    // Kết thúc test agent
    let t_run_id = test_run_id.parse::<RunId>().unwrap();
    ctx.event_bus.publish(Envelope::new(
        chrono::Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::RunFinished {
            run_id: t_run_id,
            exit_code: Some(0),
        },
    ));
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Kiểm tra flagged_files
    let (flagged_files,): (String,) =
        sqlx::query_as("SELECT flagged_files FROM m_test_agent_run WHERE run_id = ?")
            .bind(&test_run_id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

    assert!(
        flagged_files.contains("production.rs"),
        "production.rs phải bị gắn cờ vì nằm ngoài allowed patterns"
    );
}

#[tokio::test]
async fn test_ac5_disabling_module_stops_auto_trigger() {
    let ctx = setup_test_context().await;
    let ws_id = ctx.workspace_id.parse::<WorkspaceId>().unwrap();
    let prj_id = ctx.project_id.parse::<ProjectId>().unwrap();
    let t_id = ctx.task_id.parse::<TaskId>().unwrap();

    // Dừng module test-agent
    ctx.lifecycle_manager
        .disable(
            "test-agent",
            &RegistryScope::System,
            RegistryRole::SystemOwner,
            "owner",
            false,
        )
        .await
        .unwrap();

    let worker_run_id = RunId::new();
    ctx.event_bus.publish(Envelope::new(
        chrono::Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::RunFinished {
            run_id: worker_run_id,
            exit_code: Some(0),
        },
    ));
    tokio::time::sleep(Duration::from_millis(300)).await;

    let count: (i64,) = sqlx::query_as("SELECT count(*) FROM m_test_agent_run")
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(
        count.0, 0,
        "Khi tắt module, worker kết thúc không tự chạy test agent"
    );
}

#[tokio::test]
async fn test_ac6_workspace_isolation_bola() {
    let ctx = setup_test_context().await;

    // Non-member cố truy cập cấu hình project -> 403 Forbidden
    let req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/test-agent/projects/{}/config",
            ctx.project_id
        ))
        .header(header::COOKIE, &ctx.non_member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Non-member cố cập nhật cấu hình project -> 403 Forbidden
    let req_update = Request::builder()
        .method("PUT")
        .uri(format!(
            "/api/v1/m/test-agent/projects/{}/config",
            ctx.project_id
        ))
        .header(header::COOKIE, &ctx.non_member_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "enabled": false }).to_string()))
        .unwrap();
    let resp_update = ctx.app.clone().oneshot(req_update).await.unwrap();
    assert_eq!(resp_update.status(), StatusCode::FORBIDDEN);

    // Non-member cố xem kết quả task -> 403 Forbidden
    let req_results = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/test-agent/tasks/{}/results",
            ctx.task_id
        ))
        .header(header::COOKIE, &ctx.non_member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp_results = ctx.app.clone().oneshot(req_results).await.unwrap();
    assert_eq!(resp_results.status(), StatusCode::FORBIDDEN);
}

//! Các kiểm thử tích hợp toàn diện cho Task P1-01 (Run Pipeline).

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    clippy::indexing_slicing,
    reason = "Kiểm thử tích hợp P1-01 dùng unwrap, indexing json cho kịch bản và có fixture dài"
)]

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use dandori_agent_adapters::FakeAdapter;
use dandori_agent_port::{AgentRole, BoxFuture};
use dandori_db::repo::{
    AgentAccountRepo, ProjectRepo, RunEventRepo, RunRepo, TaskRepo, WorkspaceRepo,
};
use dandori_events::EventBus;
use dandori_module_api::{
    PromptContributor, PromptPart, ReleaseReason, Workdir, WorkdirError, WorkdirProvider,
};
use dandori_registry::FeatureRegistry;
use dandori_registry::lifecycle::FeatureLifecycleManager;
use dandori_server::app::create_app;
use dandori_server::config::ServerConfig;
use dandori_server::run_service::{DEFAULT_MAX_CONCURRENT_RUNS, RunService};
use dandori_server::state::AppState;
use dandori_types::{
    AgentAccount, AgentAccountId, Project, ProjectId, RunId, Task, TaskId, UserId, Workspace,
    WorkspaceId, WorkspaceMember,
};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tower::ServiceExt;

#[derive(Debug, Clone)]
struct MockWorkdirProvider {
    workdir_path: PathBuf,
}

impl WorkdirProvider for MockWorkdirProvider {
    fn name(&self) -> &'static str {
        "mock-workdir-provider"
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

#[derive(Debug, Clone)]
struct MockPromptContributor {
    contributor_name: &'static str,
    source: String,
    content: String,
    order: i32,
}

impl PromptContributor for MockPromptContributor {
    fn name(&self) -> &'static str {
        self.contributor_name
    }

    fn contribute(&self, _task: &Task, _role: &AgentRole) -> BoxFuture<'static, Vec<PromptPart>> {
        let part = PromptPart::new(self.source.clone(), self.order, self.content.clone());
        Box::pin(async move { vec![part] })
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
    pool: SqlitePool,
    lifecycle_manager: Arc<FeatureLifecycleManager>,
    run_service: Arc<RunService>,
    cookie: String,
    user_id: UserId,
    project_id: ProjectId,
    task_id: TaskId,
    account_id: AgentAccountId,
    temp_dir: tempfile::TempDir,
}

async fn setup_p1_test_context(max_concurrent: usize) -> TestContext {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    let core_schema = include_str!("../../../crates/dandori-db/migrations/0001_core_schema.sql");
    sqlx::query(core_schema).execute(&pool).await.unwrap();

    let auth_schema = include_str!("../../../crates/mod-auth-login/migrations/0001_auth_login.sql");
    sqlx::query(auth_schema).execute(&pool).await.unwrap();

    let event_bus = Arc::new(EventBus::new(200));
    let registry = FeatureRegistry::new(pool.clone());
    let lifecycle_manager = Arc::new(FeatureLifecycleManager::new(
        registry,
        pool.clone(),
        Arc::clone(&event_bus),
    ));

    let run_service = Arc::new(RunService::new(
        pool.clone(),
        Arc::clone(&event_bus),
        Arc::clone(&lifecycle_manager),
        max_concurrent,
    ));

    let fake_bin = fake_agent_bin_path();
    let fake_adapter = Arc::new(FakeAdapter::new(fake_bin));
    run_service
        .register_adapter("fake", fake_adapter.clone())
        .await;
    run_service
        .register_adapter("fake-agent", fake_adapter)
        .await;

    let config = ServerConfig::default();
    let state = AppState::new_with_run_service(
        pool.clone(),
        Arc::clone(&event_bus),
        Arc::clone(&lifecycle_manager),
        config,
        Arc::clone(&run_service),
    );

    let app = create_app(state);

    // 1. Tạo owner và đăng nhập
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

    let cookie_val = resp_login
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let cookie = cookie_val.split(';').next().unwrap().to_string();

    // 2. Lấy User ID
    let req_me = Request::builder()
        .method("GET")
        .uri("/api/v1/me")
        .header(header::COOKIE, &cookie)
        .body(Body::empty())
        .unwrap();
    let resp_me = app.clone().oneshot(req_me).await.unwrap();
    let body_me = axum::body::to_bytes(resp_me.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let me_json: Value = serde_json::from_slice(&body_me).unwrap();
    let user_id_str = me_json["id"].as_str().unwrap();
    let user_id = UserId::from_str(user_id_str).unwrap();

    // 3. Tạo workspace, project, task, agent_account trực tiếp qua DB
    let ws_id = WorkspaceId::new();
    let ws_repo = WorkspaceRepo::new(&pool);
    ws_repo
        .create_workspace(&Workspace {
            id: ws_id,
            name: "Test Workspace".into(),
            created_at: Utc::now(),
        })
        .await
        .unwrap();

    ws_repo
        .add_member(&WorkspaceMember {
            workspace_id: ws_id,
            user_id,
            role: "admin".into(),
        })
        .await
        .unwrap();

    let proj_id = ProjectId::new();
    let proj_repo = ProjectRepo::new(&pool);
    proj_repo
        .create_project(&Project {
            id: proj_id,
            workspace_id: ws_id,
            name: "Test Project".into(),
            repo_path: "/tmp/repo".into(),
            default_branch: "main".into(),
            created_at: Utc::now(),
        })
        .await
        .unwrap();

    let task_id = TaskId::new();
    let task_repo = TaskRepo::new(&pool);
    task_repo
        .create_task(&Task {
            id: task_id,
            project_id: proj_id,
            title: "Task 1".into(),
            description: "Default description".into(),
            status: "todo".into(),
            lead_agent: "lead".into(),
            worker_agent: "worker".into(),
            workdir_path: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .await
        .unwrap();

    let acc_id = AgentAccountId::new();
    let acc_repo = AgentAccountRepo::new(&pool);
    acc_repo
        .create_account(&AgentAccount {
            id: acc_id,
            provider: "fake".into(),
            label: "Test Fake Agent".into(),
            home_dir: String::new(),
            auth_type: "cli".into(),
            secret_id: None,
            status: "ready".into(),
            enabled: true,
            priority: 10,
            exhausted_until: None,
            last_checked_at: None,
        })
        .await
        .unwrap();

    let temp_dir = tempfile::tempdir().unwrap();

    TestContext {
        app,
        pool,
        lifecycle_manager,
        run_service,
        cookie,
        user_id,
        project_id: proj_id,
        task_id,
        account_id: acc_id,
        temp_dir,
    }
}

#[tokio::test]
async fn test_ac1_missing_workdir_provider_returns_workdir_provider_missing() {
    let ctx = setup_p1_test_context(DEFAULT_MAX_CONCURRENT_RUNS).await;

    // Không đăng ký bất kỳ WorkdirProvider nào
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{}/runs", ctx.task_id))
        .header(header::COOKIE, &ctx.cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({}).to_string()))
        .unwrap();

    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let err_json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(err_json["error"]["code"], "workdir_provider_missing");

    // Task status vẫn là 'todo', không có run nào được tạo trong DB
    let task = TaskRepo::new(&ctx.pool)
        .get_task_by_id(&ctx.task_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(task.status, "todo");
}

#[tokio::test]
async fn test_ac2_normal_run_completes_full_lifecycle() {
    let ctx = setup_p1_test_context(DEFAULT_MAX_CONCURRENT_RUNS).await;

    // Đăng ký WorkdirProvider
    let provider = Arc::new(MockWorkdirProvider {
        workdir_path: ctx.temp_dir.path().to_path_buf(),
    });
    ctx.lifecycle_manager
        .register_workdir_provider_for_test(provider)
        .await;

    // Cập nhật kịch bản chạy bình thường (exit code 0) cho task description
    let scenario = json!({
        "print_lines": ["Line 1: starting task", "Line 2: processing work"],
        "exit_code": 0
    });
    TaskRepo::new(&ctx.pool)
        .update_task(
            &ctx.project_id,
            &ctx.task_id,
            "Task 1",
            &scenario.to_string(),
        )
        .await
        .unwrap();

    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{}/runs", ctx.task_id))
        .header(header::COOKIE, &ctx.cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({}).to_string()))
        .unwrap();

    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let resp_json: Value = serde_json::from_slice(&bytes).unwrap();
    let run_id_str = resp_json["run_id"].as_str().unwrap();
    let run_id = RunId::from_str(run_id_str).unwrap();

    // Chờ vòng lặp chạy nền kết thúc (tối đa 5s)
    let mut finished = false;
    for _ in 0..50 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let task = TaskRepo::new(&ctx.pool)
            .get_task_by_id(&ctx.task_id)
            .await
            .unwrap()
            .unwrap();
        if task.status == "in_review" {
            finished = true;
            break;
        }
    }
    assert!(finished, "Task phải chuyển sang trạng thái 'in_review'");

    // Kiểm tra bản ghi Run trong DB
    let run = RunRepo::new(&ctx.pool)
        .get_run_by_id(&run_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(run.status, "success");
    assert_eq!(run.exit_code, Some(0));

    // Kiểm tra danh sách sự kiện qua API
    let req_events = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/runs/{run_id}/events"))
        .header(header::COOKIE, &ctx.cookie)
        .body(Body::empty())
        .unwrap();
    let resp_events = ctx.app.clone().oneshot(req_events).await.unwrap();
    assert_eq!(resp_events.status(), StatusCode::OK);

    let body_events = axum::body::to_bytes(resp_events.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let events_json: Value = serde_json::from_slice(&body_events).unwrap();
    let events = events_json["events"].as_array().unwrap();
    assert!(!events.is_empty(), "Phải có sự kiện được ghi vào DB");
}

#[tokio::test]
async fn test_ac3_agent_exit_code_failure() {
    let ctx = setup_p1_test_context(DEFAULT_MAX_CONCURRENT_RUNS).await;

    let provider = Arc::new(MockWorkdirProvider {
        workdir_path: ctx.temp_dir.path().to_path_buf(),
    });
    ctx.lifecycle_manager
        .register_workdir_provider_for_test(provider)
        .await;

    // Kịch bản thoát với mã lỗi 42
    let scenario = json!({
        "print_lines": ["Lỗi nghiêm trọng xảy ra"],
        "exit_code": 42
    });
    TaskRepo::new(&ctx.pool)
        .update_task(
            &ctx.project_id,
            &ctx.task_id,
            "Task 1",
            &scenario.to_string(),
        )
        .await
        .unwrap();

    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{}/runs", ctx.task_id))
        .header(header::COOKIE, &ctx.cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({}).to_string()))
        .unwrap();

    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let resp_json: Value = serde_json::from_slice(&bytes).unwrap();
    let run_id = RunId::from_str(resp_json["run_id"].as_str().unwrap()).unwrap();

    let mut finished = false;
    for _ in 0..50 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let task = TaskRepo::new(&ctx.pool)
            .get_task_by_id(&ctx.task_id)
            .await
            .unwrap()
            .unwrap();
        if task.status == "failed" {
            finished = true;
            break;
        }
    }
    assert!(finished, "Task phải chuyển sang trạng thái 'failed'");

    let run = RunRepo::new(&ctx.pool)
        .get_run_by_id(&run_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(run.status, "failed");
    assert_eq!(run.exit_code, Some(42));
}

#[tokio::test]
async fn test_ac4_agent_quota_exhausted() {
    let ctx = setup_p1_test_context(DEFAULT_MAX_CONCURRENT_RUNS).await;

    let provider = Arc::new(MockWorkdirProvider {
        workdir_path: ctx.temp_dir.path().to_path_buf(),
    });
    ctx.lifecycle_manager
        .register_workdir_provider_for_test(provider)
        .await;

    // Kịch bản hết quota
    let scenario = json!({
        "quota_exhausted": true,
        "exit_code": 1
    });
    TaskRepo::new(&ctx.pool)
        .update_task(
            &ctx.project_id,
            &ctx.task_id,
            "Task 1",
            &scenario.to_string(),
        )
        .await
        .unwrap();

    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{}/runs", ctx.task_id))
        .header(header::COOKIE, &ctx.cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({}).to_string()))
        .unwrap();

    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let mut finished = false;
    for _ in 0..50 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let task = TaskRepo::new(&ctx.pool)
            .get_task_by_id(&ctx.task_id)
            .await
            .unwrap()
            .unwrap();
        if task.status == "needs_human" {
            finished = true;
            break;
        }
    }
    assert!(
        finished,
        "Task phải chuyển sang trạng thái 'needs_human' khi hết hạn mức"
    );

    // Kiểm tra agent_account đã được cập nhật exhausted_until
    let acc = AgentAccountRepo::new(&ctx.pool)
        .get_account(&ctx.account_id)
        .await
        .unwrap()
        .unwrap();
    assert!(
        acc.exhausted_until.is_some(),
        "Tài khoản phải có exhausted_until"
    );
    assert!(acc.exhausted_until.unwrap() > Utc::now());
}

#[tokio::test]
async fn test_ac5_cancel_run_mid_flight() {
    let ctx = setup_p1_test_context(DEFAULT_MAX_CONCURRENT_RUNS).await;

    let provider = Arc::new(MockWorkdirProvider {
        workdir_path: ctx.temp_dir.path().to_path_buf(),
    });
    ctx.lifecycle_manager
        .register_workdir_provider_for_test(provider)
        .await;

    // Kịch bản ngủ 5s để kịp gửi lệnh hủy
    let scenario = json!({
        "sleep_before_ms": 5000,
        "exit_code": 0
    });
    TaskRepo::new(&ctx.pool)
        .update_task(
            &ctx.project_id,
            &ctx.task_id,
            "Task 1",
            &scenario.to_string(),
        )
        .await
        .unwrap();

    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{}/runs", ctx.task_id))
        .header(header::COOKIE, &ctx.cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({}).to_string()))
        .unwrap();

    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let resp_json: Value = serde_json::from_slice(&bytes).unwrap();
    let run_id = RunId::from_str(resp_json["run_id"].as_str().unwrap()).unwrap();

    // Đợi 200ms để đảm bảo tiến trình đã khởi chạy
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Gửi lệnh POST /runs/{id}/cancel
    let req_cancel = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/runs/{run_id}/cancel"))
        .header(header::COOKIE, &ctx.cookie)
        .body(Body::empty())
        .unwrap();
    let resp_cancel = ctx.app.clone().oneshot(req_cancel).await.unwrap();
    assert_eq!(resp_cancel.status(), StatusCode::OK);

    // Chờ task chuyển sang 'cancelled'
    let mut finished = false;
    for _ in 0..50 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let task = TaskRepo::new(&ctx.pool)
            .get_task_by_id(&ctx.task_id)
            .await
            .unwrap()
            .unwrap();
        if task.status == "cancelled" {
            finished = true;
            break;
        }
    }
    assert!(finished, "Task phải chuyển sang trạng thái 'cancelled'");

    let run = RunRepo::new(&ctx.pool)
        .get_run_by_id(&run_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(run.status, "cancelled");
}

#[tokio::test]
async fn test_ac6_concurrency_limit_429() {
    // Khởi tạo hệ thống với giới hạn tối đa 3 run đồng thời
    let ctx = setup_p1_test_context(3).await;

    let provider = Arc::new(MockWorkdirProvider {
        workdir_path: ctx.temp_dir.path().to_path_buf(),
    });
    ctx.lifecycle_manager
        .register_workdir_provider_for_test(provider)
        .await;

    // Tạo 4 task ngủ 10s
    let scenario = json!({
        "sleep_before_ms": 10000,
        "exit_code": 0
    });

    let mut task_ids = Vec::new();
    for i in 1..=4 {
        let t_id = TaskId::new();
        TaskRepo::new(&ctx.pool)
            .create_task(&Task {
                id: t_id,
                project_id: ctx.project_id,
                title: format!("Concurrent Task {i}"),
                description: scenario.to_string(),
                status: "todo".into(),
                lead_agent: "lead".into(),
                worker_agent: "worker".into(),
                workdir_path: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
            .await
            .unwrap();
        task_ids.push(t_id);
    }

    let mut run_ids = Vec::new();
    // Bắt đầu 3 task đầu tiên -> đều thành công (201)
    for t_id in &task_ids[0..3] {
        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/tasks/{t_id}/runs"))
            .header(header::COOKIE, &ctx.cookie)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({}).to_string()))
            .unwrap();
        let resp = ctx.app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let b = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let r_json: Value = serde_json::from_slice(&b).unwrap();
        run_ids.push(RunId::from_str(r_json["run_id"].as_str().unwrap()).unwrap());
    }

    // Task thứ 4 phải bị trả về 429 Too Many Requests kèm Retry-After: 5
    let req4 = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{}/runs", task_ids[3]))
        .header(header::COOKIE, &ctx.cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({}).to_string()))
        .unwrap();
    let resp4 = ctx.app.clone().oneshot(req4).await.unwrap();
    assert_eq!(resp4.status(), StatusCode::TOO_MANY_REQUESTS);

    let retry_after = resp4
        .headers()
        .get(header::RETRY_AFTER)
        .expect("Phải có header Retry-After")
        .to_str()
        .unwrap();
    assert_eq!(retry_after, "5");

    let b4 = axum::body::to_bytes(resp4.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let j4: Value = serde_json::from_slice(&b4).unwrap();
    assert_eq!(j4["error"]["code"], "concurrency_limit_reached");

    // Dọn dẹp hủy các run đang chạy
    for r_id in run_ids {
        let _ = ctx.run_service.cancel_run(r_id, ctx.user_id).await;
    }
}

#[tokio::test]
async fn test_ac7_prompt_assembly_ordering_sources_and_truncation() {
    let ctx = setup_p1_test_context(DEFAULT_MAX_CONCURRENT_RUNS).await;

    // Sử dụng adapter ghi nhận để kiểm tra chuỗi prompt thực tế được lắp ráp
    let captured = Arc::new(tokio::sync::Mutex::new(None::<String>));
    let cap_clone = Arc::clone(&captured);

    #[derive(Debug)]
    struct CapturingAdapter {
        cap: Arc<tokio::sync::Mutex<Option<String>>>,
    }

    impl dandori_agent_port::AgentAdapter for CapturingAdapter {
        fn name(&self) -> &'static str {
            "capturing"
        }
        fn start_run(
            &self,
            spec: dandori_agent_port::AgentRunSpec,
        ) -> BoxFuture<
            '_,
            Result<
                tokio::sync::mpsc::Receiver<dandori_agent_port::AgentEvent>,
                dandori_agent_port::AgentPortError,
            >,
        > {
            let cap = Arc::clone(&self.cap);
            Box::pin(async move {
                *cap.lock().await = Some(spec.prompt);
                let (tx, rx) = tokio::sync::mpsc::channel(4);
                let _ = tx
                    .send(dandori_agent_port::AgentEvent::Finished { exit_code: 0 })
                    .await;
                Ok(rx)
            })
        }
    }

    ctx.run_service
        .register_adapter("fake", Arc::new(CapturingAdapter { cap: cap_clone }))
        .await;

    // Đăng ký 2 prompt contributor với order khác nhau: c1 order 20, c2 order 10
    let c1 = Arc::new(MockPromptContributor {
        contributor_name: "c1",
        source: "tech-profile".into(),
        content: "Nội dung profile kỹ thuật".into(),
        order: 20,
    });
    let c2 = Arc::new(MockPromptContributor {
        contributor_name: "c2",
        source: "spec-doc".into(),
        content: "Nội dung tài liệu thiết kế".into(),
        order: 10,
    });
    ctx.lifecycle_manager
        .register_prompt_contributor_for_test(c1)
        .await;
    ctx.lifecycle_manager
        .register_prompt_contributor_for_test(c2)
        .await;

    let provider = Arc::new(MockWorkdirProvider {
        workdir_path: ctx.temp_dir.path().to_path_buf(),
    });
    ctx.lifecycle_manager
        .register_workdir_provider_for_test(provider)
        .await;

    // Khởi chạy task
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{}/runs", ctx.task_id))
        .header(header::COOKIE, &ctx.cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({}).to_string()))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Chờ adapter thu thập prompt
    tokio::time::sleep(Duration::from_millis(200)).await;
    let prompt_opt = captured.lock().await.clone();
    assert!(prompt_opt.is_some(), "Adapter phải thu thập được prompt");
    let prompt = prompt_opt.unwrap();

    // 1. Kiểm tra nguồn có xuất hiện
    assert!(prompt.contains("[Nguồn: spec-doc]"));
    assert!(prompt.contains("[Nguồn: tech-profile]"));

    // 2. Kiểm tra thứ tự: order 10 (spec-doc) phải xuất hiện trước order 20 (tech-profile)
    let pos_spec = prompt.find("[Nguồn: spec-doc]").unwrap();
    let pos_tech = prompt.find("[Nguồn: tech-profile]").unwrap();
    assert!(
        pos_spec < pos_tech,
        "order 10 (spec-doc) phải đứng trước order 20 (tech-profile)"
    );

    // 3. Kiểm tra cắt ngắn khi vượt quá 32,000 ký tự
    let huge_content = "X".repeat(40_000);
    let c_huge = Arc::new(MockPromptContributor {
        contributor_name: "c_huge",
        source: "huge-source".into(),
        content: huge_content,
        order: 30,
    });
    ctx.lifecycle_manager
        .register_prompt_contributor_for_test(c_huge)
        .await;

    // Tạo task thứ 2 để test cắt bớt
    let task_id_2 = TaskId::new();
    TaskRepo::new(&ctx.pool)
        .create_task(&Task {
            id: task_id_2,
            project_id: ctx.project_id,
            title: "Task 2 Huge".into(),
            description: "Huge task".into(),
            status: "todo".into(),
            lead_agent: "lead".into(),
            worker_agent: "worker".into(),
            workdir_path: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .await
        .unwrap();

    let req2 = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{task_id_2}/runs"))
        .header(header::COOKIE, &ctx.cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({}).to_string()))
        .unwrap();
    let resp2 = ctx.app.clone().oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::CREATED);

    tokio::time::sleep(Duration::from_millis(200)).await;
    let huge_prompt = captured.lock().await.clone().unwrap();
    assert!(huge_prompt.contains("[CẮT BỚT DO VƯỢT QUÁ GIỚI HẠN 32000 KÝ TỰ]"));
    assert!(huge_prompt.chars().count() <= 32_000);
}

#[tokio::test]
async fn test_ac8_sensitive_data_redaction() {
    let ctx = setup_p1_test_context(DEFAULT_MAX_CONCURRENT_RUNS).await;

    let provider = Arc::new(MockWorkdirProvider {
        workdir_path: ctx.temp_dir.path().to_path_buf(),
    });
    ctx.lifecycle_manager
        .register_workdir_provider_for_test(provider)
        .await;

    // Giả lập output của CLI agent in ra API key và token nhạy cảm
    let sensitive_key = format!("{}-api03-abcdef123456789012345678901234567890", "sk-ant");
    let bearer_secret = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.secret1234567890";
    let scenario = json!({
        "print_lines": [
            format!("API Key tìm thấy: {sensitive_key}"),
            format!("Header ủy quyền: {bearer_secret}")
        ],
        "exit_code": 0
    });

    TaskRepo::new(&ctx.pool)
        .update_task(
            &ctx.project_id,
            &ctx.task_id,
            "Task 1",
            &scenario.to_string(),
        )
        .await
        .unwrap();

    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{}/runs", ctx.task_id))
        .header(header::COOKIE, &ctx.cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({}).to_string()))
        .unwrap();

    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let resp_json: Value = serde_json::from_slice(&bytes).unwrap();
    let run_id = RunId::from_str(resp_json["run_id"].as_str().unwrap()).unwrap();

    // Chờ task hoàn thành
    for _ in 0..50 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let task = TaskRepo::new(&ctx.pool)
            .get_task_by_id(&ctx.task_id)
            .await
            .unwrap()
            .unwrap();
        if task.status == "in_review" {
            break;
        }
    }

    // Đọc sự kiện từ DB
    let (events, _) = RunEventRepo::new(&ctx.pool)
        .list_run_events_paginated(&run_id, 100, 0)
        .await
        .unwrap();

    for ev in events {
        assert!(
            !ev.data.contains(&sensitive_key),
            "Dữ liệu sự kiện trong DB không được chứa secret key nguyên văn: {}",
            ev.data
        );
        assert!(
            !ev.data.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"),
            "Dữ liệu sự kiện trong DB không được chứa bearer token nguyên văn: {}",
            ev.data
        );
    }
}

#[tokio::test]
async fn test_ac9_workspace_isolation_forbidden() {
    let ctx = setup_p1_test_context(DEFAULT_MAX_CONCURRENT_RUNS).await;

    // Tạo workspace B và user B
    let user_b_id = UserId::new();
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(user_b_id.to_string())
    .bind("user_b")
    .bind("hash")
    .bind("user")
    .bind(&now)
    .execute(&ctx.pool)
    .await
    .unwrap();

    let ws_b_id = WorkspaceId::new();
    let ws_repo = WorkspaceRepo::new(&ctx.pool);
    ws_repo
        .create_workspace(&Workspace {
            id: ws_b_id,
            name: "Workspace B".into(),
            created_at: Utc::now(),
        })
        .await
        .unwrap();

    ws_repo
        .add_member(&WorkspaceMember {
            workspace_id: ws_b_id,
            user_id: user_b_id,
            role: "member".into(),
        })
        .await
        .unwrap();

    // Đăng ký WorkdirProvider
    let provider = Arc::new(MockWorkdirProvider {
        workdir_path: ctx.temp_dir.path().to_path_buf(),
    });
    ctx.lifecycle_manager
        .register_workdir_provider_for_test(provider)
        .await;

    // Gọi start_run trực tiếp từ run_service với user_b_id trên task thuộc workspace A
    let res = ctx
        .run_service
        .start_run(ctx.task_id, AgentRole::Worker, user_b_id, vec![])
        .await;

    assert!(
        res.is_err(),
        "User thuộc Workspace B không được phép chạy task thuộc Workspace A"
    );
    match res.unwrap_err() {
        dandori_server::run_service::RunServiceError::Forbidden(msg) => {
            assert!(msg.contains("không thuộc workspace"));
        }
        other => panic!("Kỳ vọng Forbidden, nhận được: {other:?}"),
    }
}

#[tokio::test]
async fn test_start_run_without_registered_adapter_leaves_task_untouched() {
    let ctx = setup_p1_test_context(DEFAULT_MAX_CONCURRENT_RUNS).await;
    ctx.lifecycle_manager
        .register_workdir_provider_for_test(Arc::new(MockWorkdirProvider {
            workdir_path: ctx.temp_dir.path().to_path_buf(),
        }))
        .await;

    // Chỉ còn một tài khoản của nhà cung cấp chưa đăng ký adapter nào.
    sqlx::query("UPDATE agent_account SET enabled = 0")
        .execute(&ctx.pool)
        .await
        .unwrap();
    AgentAccountRepo::new(&ctx.pool)
        .create_account(&AgentAccount {
            id: AgentAccountId::new(),
            provider: "ghost".into(),
            label: "Ghost".into(),
            home_dir: String::new(),
            auth_type: "cli".into(),
            secret_id: None,
            status: "ready".into(),
            enabled: true,
            priority: 1,
            exhausted_until: None,
            last_checked_at: None,
        })
        .await
        .unwrap();

    let result = ctx
        .run_service
        .start_run(ctx.task_id, AgentRole::Worker, ctx.user_id, Vec::new())
        .await;
    assert!(result.is_err(), "không có adapter thì phải báo lỗi");

    let task = TaskRepo::new(&ctx.pool)
        .get_task_by_id(&ctx.task_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        task.status, "todo",
        "lỗi trước khi chạy không được để task kẹt ở trạng thái running"
    );
    let running: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM run WHERE status = 'running'")
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    assert_eq!(running, 0, "không được để lại lần chạy ma");
}

#[tokio::test]
async fn test_two_simultaneous_starts_of_same_task_create_one_run() {
    let ctx = setup_p1_test_context(DEFAULT_MAX_CONCURRENT_RUNS).await;
    ctx.lifecycle_manager
        .register_workdir_provider_for_test(Arc::new(MockWorkdirProvider {
            workdir_path: ctx.temp_dir.path().to_path_buf(),
        }))
        .await;
    let scenario = json!({ "sleep_before_ms": 5000, "exit_code": 0 });
    TaskRepo::new(&ctx.pool)
        .update_task(
            &ctx.project_id,
            &ctx.task_id,
            "Task 1",
            &scenario.to_string(),
        )
        .await
        .unwrap();

    let (a, b) = tokio::join!(
        ctx.run_service
            .start_run(ctx.task_id, AgentRole::Worker, ctx.user_id, Vec::new()),
        ctx.run_service
            .start_run(ctx.task_id, AgentRole::Worker, ctx.user_id, Vec::new()),
    );
    let ok_count = usize::from(a.is_ok()) + usize::from(b.is_ok());
    assert_eq!(ok_count, 1, "chỉ một trong hai yêu cầu đồng thời được chạy");

    let total_runs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM run")
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    assert_eq!(total_runs, 1, "chỉ được tạo một bản ghi lần chạy");

    for run in [a, b].into_iter().flatten() {
        let _ = ctx.run_service.cancel_run(run.id, ctx.user_id).await;
    }
}

#[test]
fn test_redaction_hits_known_token_shapes_and_spares_ordinary_words() {
    use dandori_server::run_service::redact_sensitive_content;

    for secret in [
        "key=sk-ant-abcdefghijklmnop123",
        "token ghp_abcdefghijklmnop12345",
        "github_pat_11ABCDEFG0abcdefghijklmnop",
        "aws AKIAABCDEFGHIJKL here",
    ] {
        let out = redact_sensitive_content(secret);
        assert!(out.contains("[REDACTED]"), "phải che: {secret} -> {out}");
    }

    for ordinary in [
        "task-management-system",
        "disk-partition-layout-notes",
        "Đã sửa risk-assessment-module",
        "ask-the-team-about-this",
    ] {
        assert_eq!(
            redact_sensitive_content(ordinary),
            ordinary,
            "không được cắt nhầm văn bản thường"
        );
    }
}

//! Kiểm thử E2E tích hợp xuyên suốt kiểm chứng Cổng P1 (11_P1_CHAY_MOT_TASK.md mục P1-09).
//!
//! Kịch bản kiểm chứng:
//! 1. Khởi tạo từ DB rỗng và một git repo tạm có commit ban đầu.
//! 2. Thiết lập chủ sở hữu và áp dụng gói mẫu "Solo nhẹ" qua API tính năng.
//! 3. Tạo workspace và dự án trỏ tới repo tạm.
//! 4. Đăng ký tài khoản agent `fake` và kích hoạt sẵn sàng.
//! 5. Thêm prompt Worker vào `prompt-library` và mục hồ sơ kỹ thuật vào `tech-profile`.
//! 6. Tạo task, giao việc (dispatch), theo dõi tiến trình chạy.
//! 7. Kiểm tra worktree và nhánh được tạo riêng biệt; `fake-agent` ghi tệp bên trong worktree;
//!    cây làm việc chính của repo tạm hoàn toàn không đổi (băm HEAD giữ nguyên).
//! 8. Xác nhận prompt thực tế gửi cho agent chứa khối prompt thư viện và hồ sơ kỹ thuật.
//! 9. Thử kịch bản hủy giữa chừng -> chuyển sang `cancelled`.
//! 10. Thử tắt module `worktree` rồi giao việc -> trả về lỗi hướng dẫn `workdir_provider_missing`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use dandori_agent_adapters::FakeAdapter;
use dandori_agent_port::{AgentAdapter, AgentEvent, AgentPortError, AgentRunSpec, BoxFuture};
use dandori_events::EventBus;
use dandori_git::GitClient;
use dandori_registry::{FeatureLifecycleManager, FeatureRegistry};
use dandori_server::{AppState, ServerConfig, create_app};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tempfile::{TempDir, tempdir};
use tokio::sync::Mutex;
use tower::ServiceExt;

#[derive(Debug, Clone)]
struct CapturingAdapter {
    inner: Arc<FakeAdapter>,
    captured_prompts: Arc<Mutex<Vec<String>>>,
}

impl AgentAdapter for CapturingAdapter {
    fn name(&self) -> &'static str {
        "fake"
    }

    fn start_run(
        &self,
        spec: AgentRunSpec,
    ) -> BoxFuture<'_, Result<tokio::sync::mpsc::Receiver<AgentEvent>, AgentPortError>> {
        let prompt = spec.prompt.clone();
        let cap = Arc::clone(&self.captured_prompts);
        let inner = Arc::clone(&self.inner);
        Box::pin(async move {
            cap.lock().await.push(prompt);
            inner.start_run(spec).await
        })
    }
}

struct TestContext {
    app: axum::Router,
    pool: SqlitePool,
    owner_cookie: String,
    captured_prompts: Arc<Mutex<Vec<String>>>,
    _temp_data_dir: TempDir,
    _state: AppState,
}

impl TestContext {
    #[expect(clippy::unwrap_used, reason = "test helper")]
    async fn send_request(
        &self,
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
        let resp = self.app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
        (status, json)
    }

    async fn set_account_ready(&self, provider: &str) {
        let now = Utc::now().to_rfc3339();
        let _ = sqlx::query(
            "UPDATE agent_account SET status = 'ready', enabled = 1, updated_at = ? WHERE provider = ?",
        )
        .bind(&now)
        .bind(provider)
        .execute(&self.pool)
        .await;
    }
}

#[expect(clippy::unwrap_used, reason = "test setup helper")]
async fn init_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    let core_schema = include_str!("../../../crates/dandori-db/migrations/0001_core_schema.sql");
    sqlx::query(core_schema).execute(&pool).await.unwrap();

    let auth_schema = include_str!("../../../crates/mod-auth-login/migrations/0001_auth_login.sql");
    sqlx::query(auth_schema).execute(&pool).await.unwrap();

    let pl_schema =
        include_str!("../../../crates/mod-prompt-library/migrations/0001_prompt_library.sql");
    sqlx::query(pl_schema).execute(&pool).await.unwrap();

    let tp_schema =
        include_str!("../../../crates/mod-tech-profile/migrations/0001_tech_profile.sql");
    sqlx::query(tp_schema).execute(&pool).await.unwrap();

    pool
}

#[expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "test setup helper"
)]
async fn setup_test_context() -> TestContext {
    let pool = init_test_db().await;
    let temp_data = tempdir().unwrap();

    let event_bus = Arc::new(EventBus::new(100));
    let registry = FeatureRegistry::new(pool.clone());
    registry.seed_modules_and_packs().await.unwrap();

    let lifecycle_manager = Arc::new(FeatureLifecycleManager::new(
        registry,
        pool.clone(),
        Arc::clone(&event_bus),
    ));

    for m in dandori_modules::registered_modules() {
        lifecycle_manager.register_module(m).await;
    }
    lifecycle_manager.restore_active_modules().await.unwrap();

    let config = ServerConfig {
        data_dir: temp_data.path().to_path_buf(),
        ..Default::default()
    };
    let state = AppState::new(
        pool.clone(),
        Arc::clone(&event_bus),
        Arc::clone(&lifecycle_manager),
        config,
    );

    // Tìm binary fake-agent đã biên dịch
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

    let raw_fake_adapter = Arc::new(FakeAdapter::new(fake_bin));
    let captured_prompts = Arc::new(Mutex::new(Vec::new()));
    let capturing_adapter = Arc::new(CapturingAdapter {
        inner: raw_fake_adapter,
        captured_prompts: Arc::clone(&captured_prompts),
    });

    state
        .run_service
        .register_adapter("fake", capturing_adapter.clone())
        .await;
    state
        .run_service
        .register_adapter("fake-agent", capturing_adapter)
        .await;

    let app = create_app(state.clone());

    // Thiết lập chủ sở hữu hệ thống (Setup Owner)
    let req_init = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "p1_owner", "password": "OwnerPassword123!" }).to_string(),
        ))
        .unwrap();
    let resp_init = app.clone().oneshot(req_init).await.unwrap();
    assert_eq!(resp_init.status(), StatusCode::OK);

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

    TestContext {
        app,
        pool,
        owner_cookie: cookie,
        captured_prompts,
        _temp_data_dir: temp_data,
        _state: state,
    }
}

#[expect(clippy::expect_used, reason = "test repo helper")]
async fn init_git_repo(path: &Path) {
    let client = GitClient::new(path);
    client.init_repo("main").await.expect("git init");
    client
        .config("user.name", "P1 Gate Committer")
        .await
        .expect("git config user.name");
    client
        .config("user.email", "p1gate@example.com")
        .await
        .expect("git config user.email");
}

#[expect(clippy::expect_used, reason = "test repo helper")]
async fn commit_sample_files(path: &Path) {
    fs::write(
        path.join("README.md"),
        "# Baseline Project\nInitial commit for P1 Gate test.\n",
    )
    .expect("write README.md");
    let client = GitClient::new(path);
    client
        .commit_all("Initial baseline commit")
        .await
        .expect("commit 1");

    fs::write(
        path.join("config.toml"),
        "[project]\nname = \"p1-gate\"\nversion = \"0.1.0\"\n",
    )
    .expect("write config.toml");
    client
        .commit_all("Add project configuration")
        .await
        .expect("commit 2");
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "comprehensive P1 gate E2E verification scenario"
)]
async fn test_p1_gate_full_lifecycle_dispatch_and_worktree() {
    let ctx = setup_test_context().await;

    // 1. Lấy danh sách gói mẫu qua API và áp dụng gói "Solo nhẹ" (solo-light)
    let (status, packs_body) = ctx
        .send_request(
            "GET",
            "/api/v1/features/packs",
            Some(&ctx.owner_cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let packs = packs_body["packs"]
        .as_array()
        .expect("Phải có danh sách packs");
    let solo_pack = packs
        .iter()
        .find(|p| p["id"] == "solo-light")
        .expect("Phải tìm thấy gói solo-light");

    let modules = solo_pack["modules"]
        .as_array()
        .expect("Phải có danh sách modules trong pack");

    // Áp dụng từng module trong gói "Solo nhẹ" qua API
    for m in modules {
        let mod_id = m.as_str().unwrap();
        let (apply_status, apply_body) = ctx
            .send_request(
                "POST",
                "/api/v1/features/apply",
                Some(&ctx.owner_cookie),
                Some(json!({
                    "module_id": mod_id,
                    "scope": "system",
                    "target_id": "system",
                    "enable": true,
                    "risk_acknowledged": true
                })),
            )
            .await;
        assert_eq!(
            apply_status,
            StatusCode::OK,
            "Kích hoạt module {mod_id} thất bại: {apply_body:?}"
        );
    }

    // 2. Chuẩn bị repository Git tạm thời có 2 commit
    let repo_dir = tempdir().unwrap();
    init_git_repo(repo_dir.path()).await;
    commit_sample_files(repo_dir.path()).await;

    let repo_path_str = repo_dir.path().to_string_lossy().to_string();
    let git_client = GitClient::new(repo_dir.path());
    let head_before = git_client
        .rev_parse("HEAD")
        .await
        .expect("Lấy băm HEAD ban đầu");
    println!("[P1-GATE] Băm HEAD ban đầu của repo chính: {head_before}");

    // 3. Tạo Workspace và Dự án trỏ tới repo tạm
    let (status, ws_body) = ctx
        .send_request(
            "POST",
            "/api/v1/workspaces",
            Some(&ctx.owner_cookie),
            Some(json!({ "name": "Workspace P1 Gate" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let ws_id = ws_body["workspace"]["id"].as_str().unwrap().to_string();

    let (status, proj_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/projects"),
            Some(&ctx.owner_cookie),
            Some(json!({
                "name": "Dự án P1 Gate",
                "repo_path": repo_path_str,
                "default_branch": "main"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let proj_id = proj_body["project"]["id"].as_str().unwrap().to_string();

    // 4. Đăng ký tài khoản nhà cung cấp fake và đưa vào trạng thái ready
    let (status, acc_body) = ctx
        .send_request(
            "POST",
            "/api/v1/m/subscription-connect/accounts",
            Some(&ctx.owner_cookie),
            Some(json!({
                "provider": "fake",
                "label": "Fake Agent P1 Account"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "Tạo account: {acc_body:?}");
    ctx.set_account_ready("fake").await;

    // 5. Thêm prompt Worker vào prompt-library và hồ sơ kỹ thuật vào tech-profile
    let (status, prm_body) = ctx
        .send_request(
            "POST",
            "/api/v1/m/prompt-library/prompts",
            Some(&ctx.owner_cookie),
            Some(json!({
                "workspace_id": ws_id,
                "project_id": proj_id,
                "name": "Chỉ thị Worker TDD",
                "role": "worker",
                "skill_tag": "rust",
                "content": "Quy tắc bắt buộc: Viết mã theo chuẩn TDD cho nhiệm vụ {{task.title}}."
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "Tạo prompt: {prm_body:?}");

    let (status, entry_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/m/tech-profile/projects/{proj_id}/entries?workspace_id={ws_id}"),
            Some(&ctx.owner_cookie),
            Some(json!({
                "workspace_id": ws_id,
                "category": "stack",
                "key": "language",
                "value": "Rust 1.83 (stable)",
                "sort_order": 10
            })),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Tạo tech-profile entry: {entry_body:?}"
    );

    // 6. Tạo Task (trạng thái ban đầu backlog)
    let scenario_spec = json!({
        "write_file": {
            "path": "p1_gate_result.txt",
            "content": "P1 Gate Execution Success by Fake Agent"
        },
        "exit_code": 0
    });
    let task_desc = scenario_spec.to_string();

    let (status, task_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/projects/{proj_id}/tasks"),
            Some(&ctx.owner_cookie),
            Some(json!({
                "title": "Nhiệm vụ kiểm thử Cổng P1",
                "description": task_desc
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let task_id = task_body["task"]["id"].as_str().unwrap().to_string();
    assert_eq!(task_body["task"]["status"], "backlog");

    // 7. Giao việc (dispatch task)
    let (status, dispatch_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/tasks/{task_id}/dispatch"),
            Some(&ctx.owner_cookie),
            None,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Dispatch task phải thành công: {dispatch_body:?}"
    );

    // 8. Chờ tác vụ agent nền thực thi xong (fake-agent hoàn tất trong < 500ms)
    tokio::time::sleep(Duration::from_millis(800)).await;

    // 9. Kiểm tra thông tin worktree và nhánh được tạo
    let (status, wt_body) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/m/worktree/tasks/{task_id}/worktree"),
            Some(&ctx.owner_cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(wt_body["exists"], true);
    let wt_path_str = wt_body["workdir_path"].as_str().unwrap();
    let wt_branch = wt_body["branch"].as_str().unwrap();

    println!("[P1-GATE] Worktree path: {wt_path_str}");
    println!("[P1-GATE] Branch: {wt_branch}");
    assert!(wt_branch.starts_with("task/nhiem-vu-kiem-thu-cong-p1"));

    // 10. Kiểm tra tệp sinh ra NẰM TRONG WORKTREE, KHÔNG NẰM TRONG REPO GỐC
    let file_in_wt = Path::new(wt_path_str).join("p1_gate_result.txt");
    assert!(
        file_in_wt.exists(),
        "Tệp p1_gate_result.txt phải được tạo bên trong thư mục worktree"
    );
    let written_content = fs::read_to_string(&file_in_wt).unwrap();
    assert_eq!(written_content, "P1 Gate Execution Success by Fake Agent");

    let file_in_main = repo_dir.path().join("p1_gate_result.txt");
    assert!(
        !file_in_main.exists(),
        "Tệp p1_gate_result.txt TUYỆT ĐỐI KHÔNG ĐƯỢC xuất hiện trong repo gốc"
    );

    // 11. Kiểm tra băm commit HEAD và cây làm việc của repo chính KHÔNG THAY ĐỔI
    let head_after = git_client
        .rev_parse("HEAD")
        .await
        .expect("Lấy băm HEAD sau khi chạy");
    println!("[P1-GATE] Băm HEAD sau khi chạy của repo chính: {head_after}");
    assert_eq!(
        head_before, head_after,
        "Băm commit HEAD của repo chính phải giữ nguyên hoàn toàn"
    );

    let main_dirty = git_client
        .status_porcelain(repo_dir.path())
        .await
        .expect("Kiểm tra status repo chính");
    assert!(
        main_dirty.is_empty(),
        "Cây làm việc repo gốc phải sạch hoàn toàn"
    );

    // 12. Kiểm tra task đã chuyển trạng thái từ backlog -> running -> in_review
    let (status, task_after) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/projects/{proj_id}/tasks"),
            Some(&ctx.owner_cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let tasks = task_after["tasks"].as_array().unwrap();
    let current_task = tasks.iter().find(|t| t["id"] == task_id).unwrap();
    assert_eq!(
        current_task["status"], "in_review",
        "Task sau khi chạy thành công phải chuyển sang in_review"
    );

    // 13. Kiểm tra prompt gửi cho agent thực tế chứa khối prompt library và tech profile
    {
        let captured = ctx.captured_prompts.lock().await;
        assert!(
            !captured.is_empty(),
            "Phải có ít nhất một prompt được gửi tới agent"
        );
        let sent_prompt = &captured[0];
        assert!(
            sent_prompt.contains("Chỉ thị Worker TDD"),
            "Prompt phải chứa nội dung từ prompt-library: {sent_prompt}"
        );
        assert!(
            sent_prompt.contains("Rust 1.83 (stable)"),
            "Prompt phải chứa nội dung hồ sơ kỹ thuật từ tech-profile: {sent_prompt}"
        );
    }

    // 14. Kịch bản bổ sung 1: Hủy chạy giữa chừng (Cancel mid-flight)
    let slow_scenario = json!({
        "sleep_before_ms": 3000,
        "exit_code": 0
    });
    let (status, task2_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/projects/{proj_id}/tasks"),
            Some(&ctx.owner_cookie),
            Some(json!({
                "title": "Nhiệm vụ kiểm thử Hủy chạy",
                "description": slow_scenario.to_string()
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let task2_id = task2_body["task"]["id"].as_str().unwrap();

    let (status, disp2_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/tasks/{task2_id}/dispatch"),
            Some(&ctx.owner_cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let run2_id = disp2_body["run_id"].as_str().unwrap();

    // Chờ 100ms cho task bắt đầu chạy rồi gửi lệnh hủy
    tokio::time::sleep(Duration::from_millis(100)).await;
    let (status, cancel_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/runs/{run2_id}/cancel"),
            Some(&ctx.owner_cookie),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "Hủy run: {cancel_body:?}");

    // Chờ tiến trình nhận tín hiệu hủy và hoàn tất cập nhật trạng thái
    let mut finished = false;
    for _ in 0..50 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let (status, run2_query) = ctx
            .send_request(
                "GET",
                &format!("/api/v1/runs/{run2_id}"),
                Some(&ctx.owner_cookie),
                None,
            )
            .await;
        if status == StatusCode::OK && run2_query["run"]["status"] == "cancelled" {
            finished = true;
            break;
        }
    }
    assert!(
        finished,
        "Run sau khi gửi lệnh hủy phải chuyển sang trạng thái 'cancelled'"
    );

    // 15. Kịch bản bổ sung 2: Tắt module worktree rồi giao việc -> lỗi hướng dẫn workdir_provider_missing
    let (status, disable_resp) = ctx
        .send_request(
            "POST",
            "/api/v1/features/apply",
            Some(&ctx.owner_cookie),
            Some(json!({
                "module_id": "worktree",
                "scope": "system",
                "target_id": "system",
                "enable": false,
                "risk_acknowledged": true
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "Tắt worktree: {disable_resp:?}");

    let (status, task3_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/projects/{proj_id}/tasks"),
            Some(&ctx.owner_cookie),
            Some(json!({
                "title": "Nhiệm vụ khi thiếu worktree",
                "description": "Thử nghiệm lỗi thiếu worktree"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let task3_id = task3_body["task"]["id"].as_str().unwrap();

    let (status, disp3_body) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/tasks/{task3_id}/dispatch"),
            Some(&ctx.owner_cookie),
            None,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "Dispatch khi không có workdir_provider phải trả về 400"
    );
    assert_eq!(disp3_body["error"]["code"], "workdir_provider_missing");
    let error_msg = disp3_body["error"]["message"].as_str().unwrap();
    assert!(
        error_msg.contains("worktree"),
        "Thông điệp phải hướng dẫn người dùng kích hoạt module worktree: {error_msg}"
    );
}

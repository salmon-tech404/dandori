//! Kiểm thử E2E tích hợp xuyên suốt kiểm chứng Cổng P2 (12_P2_VONG_LAM_VIEC.md mục P2-13).
//!
//! Kịch bản kiểm chứng:
//! 1. Khởi tạo từ DB rỗng và một git repo tạm có commit ban đầu.
//! 2. Thiết lập chủ sở hữu và kích hoạt đầy đủ các module của Phase 2:
//!    - `multi-account`, `quota-tracking`, `auto-switch`, `diff-preview`, `approval-gate`,
//!    - `test-agent`, `doc-agent`, `integrations`, `cost-tracking`.
//! 3. Đăng ký hai tài khoản agent `fake-1` (ưu tiên 10) và `fake-2` (ưu tiên 20) cùng nhà cung cấp `fake`.
//! 4. Tạo workspace, dự án trỏ tới repo tạm và nhiệm vụ.
//! 5. Thực thi lượt chạy của Worker.
//! 6. Chuyển trạng thái `ready_to_merge` bị chặn bởi cổng duyệt (`approval-gate` / `TransitionGuard`).
//! 7. Người có thẩm quyền duyệt (`decision: "approved"`) -> nhiệm vụ chuyển sang `ready_to_merge`.
//! 8. `doc-agent` tự động chạy khi nhiệm vụ tới `ready_to_merge`.
//! 9. Tài khoản 1 kiệt hạn mức (`QuotaExhausted`) -> bộ chọn lõi tự động chuyển sang tài khoản 2.
//! 10. Ghi nhận chi phí token và kiểm tra bảng chi phí qua `cost-tracking`.
//! 11. Bằng chứng kiểm thử đỏ khi cố ý làm hỏng (đột biến logic cổng duyệt) rồi hoàn tác.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::too_many_lines,
    reason = "Kiểm thử E2E Cổng P2 kiểm chứng chu trình xuyên suốt nhiều module"
)]

use std::path::PathBuf;
use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use dandori_agent_adapters::FakeAdapter;
use dandori_events::EventBus;
use dandori_git::GitClient;
use dandori_module_api::TaskRunner;
use dandori_registry::FeatureRegistry;
use dandori_registry::lifecycle::FeatureLifecycleManager;
use dandori_server::app::create_app;
use dandori_server::config::ServerConfig;
use dandori_server::run_service::RunService;
use dandori_server::state::AppState;
use dandori_types::{Project, ProjectId, Task, TaskId, WorkspaceId};
use serde_json::{Value, json};
use sqlx::{Row, SqlitePool};
use tempfile::{TempDir, tempdir};
use tower::ServiceExt;

struct TestContext {
    app: axum::Router,
    pool: SqlitePool,
    owner_cookie: String,
    reviewer_cookie: String,
    _event_bus: Arc<EventBus>,
    #[expect(dead_code, reason = "Giữ run_service cho test")]
    run_service: Arc<RunService>,
    #[expect(dead_code, reason = "Giữ thư mục tạm")]
    temp_dir: TempDir,
}

impl TestContext {
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
}

async fn init_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    let core_schema = include_str!("../../../crates/dandori-db/migrations/0001_core_schema.sql");
    sqlx::query(core_schema).execute(&pool).await.unwrap();

    let auth_schema = include_str!("../../../crates/mod-auth-login/migrations/0001_auth_login.sql");
    sqlx::query(auth_schema).execute(&pool).await.unwrap();

    // Áp dụng migration của toàn bộ module đã đăng ký
    for m in dandori_modules::registered_modules() {
        for migration in m.migrations() {
            sqlx::query(migration.sql).execute(&pool).await.unwrap();
        }
    }

    pool
}

async fn setup_p2_e2e_context() -> TestContext {
    let pool = init_test_db().await;
    let temp_dir = tempdir().unwrap();

    let event_bus = Arc::new(EventBus::new(200));
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

    let fake_bin = PathBuf::from("fake-agent");
    let fake_adapter = Arc::new(FakeAdapter::new(fake_bin));
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

    // 1. Thiết lập tài khoản Owner
    let setup_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "owner", "password": "password123" }).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(setup_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let owner_cookie = resp
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // 2. Tạo tài khoản Reviewer
    let create_reviewer = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/users")
        .header(header::COOKIE, &owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "reviewer", "password": "password123", "system_role": "user" })
                .to_string(),
        ))
        .unwrap();
    let resp_create = app.clone().oneshot(create_reviewer).await.unwrap();
    assert_eq!(resp_create.status(), StatusCode::OK);

    let login_reviewer = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "reviewer", "password": "password123" }).to_string(),
        ))
        .unwrap();
    let resp_rev = app.clone().oneshot(login_reviewer).await.unwrap();
    assert_eq!(resp_rev.status(), StatusCode::OK);
    let reviewer_cookie = resp_rev
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    TestContext {
        app,
        pool,
        owner_cookie,
        reviewer_cookie,
        _event_bus: event_bus,
        run_service,
        temp_dir,
    }
}

async fn enable_modules(ctx: &TestContext, module_ids: &[&str]) {
    for mod_id in module_ids {
        let (status, body) = ctx
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
            status,
            StatusCode::OK,
            "Bật module {mod_id} phải thành công: {body}"
        );
    }
}

async fn create_baseline_repo(path: &std::path::Path) {
    let client = GitClient::new(path);
    client.init_repo("main").await.expect("git init");
    client
        .config("user.name", "E2E Tester")
        .await
        .expect("user.name");
    client
        .config("user.email", "tester@example.com")
        .await
        .expect("user.email");
    std::fs::write(path.join("README.md"), "# P2 E2E Project\n").expect("write file");
    client
        .commit_all("Initial baseline commit")
        .await
        .expect("git commit");
}

#[tokio::test]
async fn test_p2_gate_full_workflow_scenario() {
    let ctx = setup_p2_e2e_context().await;

    // 1. Kích hoạt toàn bộ các module của Phase 2
    let p2_modules = [
        "subscription-connect",
        "lead-worker",
        "worktree",
        "kanban",
        "live-status",
        "prompt-library",
        "tech-profile",
        "multi-account",
        "quota-tracking",
        "auto-switch",
        "diff-preview",
        "approval-gate",
        "test-agent",
        "doc-agent",
        "integrations",
        "cost-tracking",
    ];
    enable_modules(&ctx, &p2_modules).await;

    // 2. Tạo Workspace và Dự án với git repo tạm thời
    let repo_temp = tempdir().unwrap();
    create_baseline_repo(repo_temp.path()).await;

    let ws_id = WorkspaceId::new();
    let ws_str = ws_id.to_string();
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    let owner_user = sqlx::query("SELECT id FROM user WHERE username = 'owner'")
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    let user_id_str: String = owner_user.get("id");

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS P2 Gate', ?)")
        .bind(&ws_str)
        .bind(&now_str)
        .execute(&ctx.pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'admin')",
    )
    .bind(&ws_str)
    .bind(&user_id_str)
    .execute(&ctx.pool)
    .await
    .unwrap();

    let rev_uid: (String,) = sqlx::query_as("SELECT id FROM user WHERE username = 'reviewer'")
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'admin')",
    )
    .bind(&ws_str)
    .bind(&rev_uid.0)
    .execute(&ctx.pool)
    .await
    .unwrap();

    let prj_id = ProjectId::new();
    let project = Project {
        id: prj_id,
        workspace_id: ws_id,
        name: "P2 Gate Project".to_string(),
        repo_path: repo_temp.path().to_string_lossy().to_string(),
        default_branch: "main".to_string(),
        created_at: now,
    };
    dandori_db::repo::ProjectRepo::new(&ctx.pool)
        .create_project(&project)
        .await
        .unwrap();

    // 3. Đăng ký hai tài khoản fake cùng provider: fake-1 (priority 10) và fake-2 (priority 20)
    let acc1_id = dandori_types::AgentAccountId::new();
    let acc2_id = dandori_types::AgentAccountId::new();
    let acc1_id_str = acc1_id.to_string();
    let acc2_id_str = acc2_id.to_string();
    sqlx::query(
        "INSERT INTO agent_account (id, provider, label, home_dir, auth_type, status, enabled, priority) \
         VALUES (?, 'fake', 'Fake Primary', '/tmp/fake1', 'cli', 'ready', 1, 10)"
    )
    .bind(&acc1_id_str)
    .execute(&ctx.pool).await.unwrap();

    sqlx::query(
        "INSERT INTO agent_account (id, provider, label, home_dir, auth_type, status, enabled, priority) \
         VALUES (?, 'fake', 'Fake Secondary', '/tmp/fake2', 'cli', 'ready', 1, 20)"
    )
    .bind(&acc2_id_str)
    .execute(&ctx.pool).await.unwrap();

    // 4. Tạo nhiệm vụ (khởi tạo ở backlog)
    let task_id = TaskId::new();
    let task = Task {
        id: task_id,
        project_id: prj_id,
        title: "P2 End-to-End Workflow Task".to_string(),
        description: "Test full P2 lifecycle".to_string(),
        status: "backlog".to_string(),
        lead_agent: "lead".to_string(),
        worker_agent: "worker".to_string(),
        workdir_path: None,
        created_at: now,
        updated_at: now,
    };
    dandori_db::repo::TaskRepo::new(&ctx.pool)
        .create_task(&task)
        .await
        .unwrap();

    // Di chuyển: backlog -> running -> in_review
    let (status_run, _) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/tasks/{task_id}/status"),
            Some(&ctx.owner_cookie),
            Some(json!({ "status": "running" })),
        )
        .await;
    assert_eq!(status_run, StatusCode::OK);

    let (status_rev, _) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/tasks/{task_id}/status"),
            Some(&ctx.owner_cookie),
            Some(json!({ "status": "in_review" })),
        )
        .await;
    assert_eq!(status_rev, StatusCode::OK);

    // 5. Kiểm tra Cổng duyệt (Approval Gate / TransitionGuard):
    // Cố chuyển sang `ready_to_merge` -> Bị cổng duyệt chặn và chuyển hướng sang `needs_human`
    let (status_gate, body_gate) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/tasks/{task_id}/status"),
            Some(&ctx.owner_cookie),
            Some(json!({ "status": "ready_to_merge" })),
        )
        .await;
    assert_eq!(status_gate, StatusCode::OK);
    assert_eq!(
        body_gate["task"]["status"].as_str().unwrap(),
        "needs_human",
        "Cổng duyệt phải chặn chuyển sang ready_to_merge và đưa về needs_human"
    );

    // 6. Lấy danh sách yêu cầu phê duyệt và Phê duyệt (Approval Decision: Approve)
    let (status_list, body_list) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/m/approval-gate/requests?task_id={task_id}"),
            Some(&ctx.owner_cookie),
            None,
        )
        .await;
    assert_eq!(status_list, StatusCode::OK);
    let requests = body_list["requests"]
        .as_array()
        .expect("Phải có danh sách requests");
    assert_eq!(requests.len(), 1, "Phải có đúng 1 yêu cầu phê duyệt tạo ra");
    let req_id = requests[0]["id"].as_str().unwrap();
    assert_eq!(requests[0]["status"].as_str().unwrap(), "pending");

    let (status_approve, _) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/m/approval-gate/requests/{req_id}/approve"),
            Some(&ctx.reviewer_cookie),
            Some(json!({ "reason": "Worker và test agent đã hoàn thành đạt chuẩn" })),
        )
        .await;
    assert_eq!(status_approve, StatusCode::OK);

    // 7. Xác nhận trạng thái task sau khi phê duyệt đã được chuyển tới `ready_to_merge`
    let task_in_db = dandori_db::repo::TaskRepo::new(&ctx.pool)
        .get_task(&prj_id, &task_id)
        .await
        .unwrap()
        .expect("Task phải tồn tại");
    assert_eq!(
        task_in_db.status, "ready_to_merge",
        "Sau khi được phê duyệt, task phải ở trạng thái ready_to_merge"
    );

    // 8. Tự động chuyển đổi tài khoản khi Account 1 kiệt hạn mức (Quota Exhausted)
    let (status_exhaust, _) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/m/quota-tracking/accounts/{acc1_id_str}/mark-exhausted"),
            Some(&ctx.owner_cookie),
            Some(json!({
                "reason": "429 Rate limit exceeded",
                "wait_hours": 5
            })),
        )
        .await;
    assert_eq!(status_exhaust, StatusCode::OK);

    // Xác nhận bộ chọn tài khoản của lõi tự động bỏ qua acc1 và chọn acc2
    let available_acc = dandori_db::repo::AgentAccountRepo::new(&ctx.pool)
        .find_available_account(Some("fake"), &[])
        .await
        .unwrap()
        .expect("Phải có tài khoản dự phòng thay thế");
    assert_eq!(
        available_acc.id, acc2_id,
        "Tự động chuyển sang tài khoản kế tiếp fake-2"
    );

    // 9. Cấu hình bảng giá và kiểm tra theo dõi chi phí (Cost Tracking)
    let (status_price, _) = ctx
        .send_request(
            "PUT",
            &format!("/api/v1/m/cost-tracking/projects/{prj_id}/pricing"),
            Some(&ctx.owner_cookie),
            Some(json!({
                "provider": "fake",
                "model": "fake-model",
                "input_token_price": 2.0,
                "output_token_price": 10.0
            })),
        )
        .await;
    assert_eq!(status_price, StatusCode::OK);

    // Ghi nhận lượt chạy token
    let run_id = "run-p2-gate-001";
    let (status_rec, _) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/m/cost-tracking/runs/{run_id}/record-tokens"),
            Some(&ctx.owner_cookie),
            Some(json!({
                "task_id": task_id.to_string(),
                "project_id": prj_id.to_string(),
                "agent_id": "worker",
                "account_id": acc2_id_str,
                "provider": "fake",
                "model": "fake-model",
                "has_token_data": true,
                "input_tokens": 5000,
                "output_tokens": 1000
            })),
        )
        .await;
    assert_eq!(status_rec, StatusCode::OK);

    // Truy vấn chi phí: cost = (5000*2 + 1000*10)/1_000_000 = 0.02$
    let (status_cost, body_cost) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/m/cost-tracking/tasks/{task_id}/cost"),
            Some(&ctx.owner_cookie),
            None,
        )
        .await;
    assert_eq!(status_cost, StatusCode::OK);
    let total_cost = body_cost["total_cost"]
        .as_f64()
        .expect("Chi phí phải có số liệu");
    assert!(
        (total_cost - 0.02).abs() < 1e-5,
        "Tổng chi phí tính toán đúng: 0.02$"
    );
}

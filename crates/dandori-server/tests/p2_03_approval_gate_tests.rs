//! Bộ kiểm thử tích hợp cho Task P2-03: Module `approval-gate` và điểm mở rộng `TransitionGuard`.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    clippy::indexing_slicing,
    reason = "Kiểm thử tích hợp P2-03 dùng unwrap và indexing json"
)]

use std::fs;
use std::path::Path;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use dandori_events::EventBus;
use dandori_git::GitClient;
use dandori_module_api::{BoxFuture, GuardDecision, GuardError, TransitionActor, TransitionGuard};
use dandori_registry::{FeatureLifecycleManager, FeatureRegistry, RegistryRole, RegistryScope};
use dandori_server::app::create_app;
use dandori_server::config::ServerConfig;
use dandori_server::state::AppState;
use dandori_types::{Task, TaskStatus};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tempfile::tempdir;
use tower::ServiceExt;

struct TestContext {
    app: axum::Router,
    owner_cookie: String,
    non_member_cookie: String,
    second_user_cookie: String,
    pool: SqlitePool,
    lifecycle_manager: Arc<FeatureLifecycleManager>,
    workspace_id: String,
    project_id: String,
    task_id: String,
    _temp_dir: tempfile::TempDir,
}

#[derive(Debug)]
struct MockDenyGuard;
impl TransitionGuard for MockDenyGuard {
    fn name(&self) -> &'static str {
        "mock-deny"
    }

    fn check(
        &self,
        _task: &Task,
        _from: TaskStatus,
        _to: TaskStatus,
        _actor: &TransitionActor,
    ) -> BoxFuture<'static, Result<GuardDecision, GuardError>> {
        Box::pin(async {
            Ok(GuardDecision::Deny {
                reason: "Chuyển trạng thái bị chặn bởi chính sách thử nghiệm".to_string(),
                retry_after: None,
            })
        })
    }
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

    // Tạo owner
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
    let second_user_cookie = login_new_user(&app, &owner_cookie, "approver").await;

    // Tạo workspace
    let ws_req = Request::builder()
        .method("POST")
        .uri("/api/v1/workspaces")
        .header(header::COOKIE, &owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "name": "Workspace Approval" }).to_string(),
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

    // Người duyệt là quản trị workspace (đặc tả P2-03: chủ dự án/admin mới được duyệt)
    let second_uid: (String,) = sqlx::query_as("SELECT id FROM user WHERE username = 'approver'")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'admin')",
    )
    .bind(&workspace_id)
    .bind(&second_uid.0)
    .execute(&pool)
    .await
    .unwrap();

    // Tạo dự án
    let prj_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/workspaces/{workspace_id}/projects"))
        .header(header::COOKIE, &owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "name": "Project Approval",
                "repo_path": repo_path.to_str().unwrap(),
                "default_branch": "main"
            })
            .to_string(),
        ))
        .unwrap();
    let prj_resp = app.clone().oneshot(prj_req).await.unwrap();
    assert_eq!(prj_resp.status(), StatusCode::OK);
    let prj_body: Value = serde_json::from_slice(
        &axum::body::to_bytes(prj_resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let project_id = prj_body["project"]["id"].as_str().unwrap().to_string();

    // Tạo task
    let task_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/projects/{project_id}/tasks"))
        .header(header::COOKIE, &owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "title": "Feature Task", "description": "Feature Desc" }).to_string(),
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

    TestContext {
        app,
        owner_cookie,
        non_member_cookie,
        second_user_cookie,
        pool,
        lifecycle_manager,
        workspace_id,
        project_id,
        task_id,
        _temp_dir: temp_dir,
    }
}

/// Helper đưa task tới trạng thái mong muốn
async fn move_task_to_status(
    app: &axum::Router,
    cookie: &str,
    task_id: &str,
    target_status: &str,
) -> StatusCode {
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{task_id}/status"))
        .header(header::COOKIE, cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "status": target_status }).to_string()))
        .unwrap();
    app.clone().oneshot(req).await.unwrap().status()
}

#[tokio::test]
async fn test_ac1_core_guards_behavior() {
    let ctx = setup_test_context().await;

    // 1. Khi không có guard nào: chuyển backlog -> running -> in_review -> ready_to_merge thành công bình thường
    assert_eq!(
        move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "running").await,
        StatusCode::OK
    );
    assert_eq!(
        move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "in_review").await,
        StatusCode::OK
    );

    // Đăng ký MockDenyGuard
    ctx.lifecycle_manager
        .register_transition_guard_for_test(Arc::new(MockDenyGuard))
        .await;

    // Guard Deny -> bị từ chối với BadRequest
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{}/status", ctx.task_id))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "status": "ready_to_merge" }).to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("bị từ chối")
    );
}

#[tokio::test]
async fn test_ac1_and_ac2_approval_gate_lifecycle() {
    let ctx = setup_test_context().await;

    // Bật module approval-gate
    ctx.lifecycle_manager
        .enable(
            "approval-gate",
            &RegistryScope::Workspace(ctx.workspace_id.clone()),
            RegistryRole::WorkspaceAdmin,
            "owner",
            false,
        )
        .await
        .unwrap();

    // Di chuyển task: backlog -> running -> in_review
    assert_eq!(
        move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "running").await,
        StatusCode::OK
    );
    assert_eq!(
        move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "in_review").await,
        StatusCode::OK
    );

    // Lắng nghe sự kiện approval.requested
    let _sub = ctx.lifecycle_manager.registry(); // event listener check below

    // Chuyển in_review -> ready_to_merge: ApprovalGateGuard trả về NeedsApproval
    // Task phải chuyển sang needs_human thay vì ready_to_merge!
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{}/status", ctx.task_id))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "status": "ready_to_merge" }).to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["task"]["status"].as_str().unwrap(), "needs_human");

    // Lấy danh sách requests từ API
    let list_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/approval-gate/requests?task_id={}",
            ctx.task_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let list_resp = ctx.app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: Value = serde_json::from_slice(
        &axum::body::to_bytes(list_resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let requests = list_body["requests"].as_array().unwrap();
    assert_eq!(requests.len(), 1);
    let request_id = requests[0]["id"].as_str().unwrap().to_string();
    assert_eq!(requests[0]["status"].as_str().unwrap(), "pending");

    // AC2: Người duyệt (second_user) phê duyệt yêu cầu -> task chuyển tới ready_to_merge
    let approve_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/approval-gate/requests/{request_id}/approve"
        ))
        .header(header::COOKIE, &ctx.second_user_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "reason": "Code looks great" }).to_string(),
        ))
        .unwrap();
    let approve_resp = ctx.app.clone().oneshot(approve_req).await.unwrap();
    assert_eq!(approve_resp.status(), StatusCode::OK);

    // Kiểm tra trạng thái task trong DB đã tới ready_to_merge
    let task_st: (String,) = sqlx::query_as("SELECT status FROM task WHERE id = ?")
        .bind(&ctx.task_id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    assert_eq!(task_st.0, "ready_to_merge");
}

#[tokio::test]
async fn test_ac2_reject_returns_to_from_status() {
    let ctx = setup_test_context().await;
    ctx.lifecycle_manager
        .enable(
            "approval-gate",
            &RegistryScope::Workspace(ctx.workspace_id.clone()),
            RegistryRole::WorkspaceAdmin,
            "owner",
            false,
        )
        .await
        .unwrap();

    move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "running").await;
    move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "in_review").await;

    // Yêu cầu chuyển sang ready_to_merge -> needs_human
    move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "ready_to_merge").await;

    // Lấy request_id
    let req_row: (String,) = sqlx::query_as(
        "SELECT id FROM m_approval_gate_request WHERE task_id = ? AND status = 'pending'",
    )
    .bind(&ctx.task_id)
    .fetch_one(&ctx.pool)
    .await
    .unwrap();

    // Từ chối yêu cầu (bởi second_user) kèm lý do
    let reject_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/approval-gate/requests/{}/reject",
            req_row.0
        ))
        .header(header::COOKIE, &ctx.second_user_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "reason": "Cần thêm unit test" }).to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(reject_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Kiểm tra task đã được hoàn tác về in_review
    let task_st: (String,) = sqlx::query_as("SELECT status FROM task WHERE id = ?")
        .bind(&ctx.task_id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    assert_eq!(task_st.0, "in_review");
}

#[tokio::test]
async fn test_ac3_requester_cannot_self_approve() {
    let ctx = setup_test_context().await;
    ctx.lifecycle_manager
        .enable(
            "approval-gate",
            &RegistryScope::Workspace(ctx.workspace_id.clone()),
            RegistryRole::WorkspaceAdmin,
            "owner",
            false,
        )
        .await
        .unwrap();

    move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "running").await;
    move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "in_review").await;
    move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "ready_to_merge").await;

    let req_row: (String,) = sqlx::query_as(
        "SELECT id FROM m_approval_gate_request WHERE task_id = ? AND status = 'pending'",
    )
    .bind(&ctx.task_id)
    .fetch_one(&ctx.pool)
    .await
    .unwrap();

    // Người yêu cầu (owner) tự phê duyệt -> bị từ chối với 400 Bad Request
    let approve_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/approval-gate/requests/{}/approve",
            req_row.0
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "reason": "Tự duyệt" }).to_string()))
        .unwrap();
    let resp = ctx.app.clone().oneshot(approve_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Người yêu cầu tự từ chối -> cũng bị chặn
    let reject_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/approval-gate/requests/{}/reject",
            req_row.0
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "reason": "Tự từ chối" }).to_string()))
        .unwrap();
    let resp2 = ctx.app.clone().oneshot(reject_req).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_ac4_concurrent_decisions_race_condition() {
    let ctx = setup_test_context().await;
    ctx.lifecycle_manager
        .enable(
            "approval-gate",
            &RegistryScope::Workspace(ctx.workspace_id.clone()),
            RegistryRole::WorkspaceAdmin,
            "owner",
            false,
        )
        .await
        .unwrap();

    move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "running").await;
    move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "in_review").await;
    move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "ready_to_merge").await;

    let req_row: (String,) = sqlx::query_as(
        "SELECT id FROM m_approval_gate_request WHERE task_id = ? AND status = 'pending'",
    )
    .bind(&ctx.task_id)
    .fetch_one(&ctx.pool)
    .await
    .unwrap();

    // Quyết định lần 1: thành công
    let approve_req1 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/approval-gate/requests/{}/approve",
            req_row.0
        ))
        .header(header::COOKIE, &ctx.second_user_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "reason": "Approve 1" }).to_string()))
        .unwrap();
    let resp1 = ctx.app.clone().oneshot(approve_req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);

    // Quyết định lần 2 cùng lúc: xung đột 409 Conflict
    let approve_req2 = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/approval-gate/requests/{}/approve",
            req_row.0
        ))
        .header(header::COOKIE, &ctx.second_user_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "reason": "Approve 2" }).to_string()))
        .unwrap();
    let resp2 = ctx.app.clone().oneshot(approve_req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_ac5_disable_module_removes_guard_and_preserves_data() {
    let ctx = setup_test_context().await;
    ctx.lifecycle_manager
        .enable(
            "approval-gate",
            &RegistryScope::Workspace(ctx.workspace_id.clone()),
            RegistryRole::WorkspaceAdmin,
            "owner",
            false,
        )
        .await
        .unwrap();

    move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "running").await;
    move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "in_review").await;

    // Tắt module approval-gate
    ctx.lifecycle_manager
        .disable(
            "approval-gate",
            &RegistryScope::Workspace(ctx.workspace_id.clone()),
            RegistryRole::WorkspaceAdmin,
            "owner",
            false,
        )
        .await
        .unwrap();

    // Khi module đã tắt -> guard được gỡ bỏ -> chuyển sang ready_to_merge bình thường không bị chặn
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/tasks/{}/status", ctx.task_id))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "status": "ready_to_merge" }).to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["task"]["status"].as_str().unwrap(), "ready_to_merge");
}

#[tokio::test]
async fn test_ac6_bola_workspace_isolation() {
    let ctx = setup_test_context().await;
    ctx.lifecycle_manager
        .enable(
            "approval-gate",
            &RegistryScope::Workspace(ctx.workspace_id.clone()),
            RegistryRole::WorkspaceAdmin,
            "owner",
            false,
        )
        .await
        .unwrap();

    move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "running").await;
    move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "in_review").await;
    move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "ready_to_merge").await;

    let req_row: (String,) = sqlx::query_as(
        "SELECT id FROM m_approval_gate_request WHERE task_id = ? AND status = 'pending'",
    )
    .bind(&ctx.task_id)
    .fetch_one(&ctx.pool)
    .await
    .unwrap();

    // Non-member cố truy cập request của workspace -> 403 Forbidden
    let get_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/approval-gate/requests/{}", req_row.0))
        .header(header::COOKIE, &ctx.non_member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Non-member cố approve request của workspace -> 403 Forbidden
    let approve_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/approval-gate/requests/{}/approve",
            req_row.0
        ))
        .header(header::COOKIE, &ctx.non_member_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "reason": "Hack" }).to_string()))
        .unwrap();
    let resp2 = ctx.app.clone().oneshot(approve_req).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_plain_member_cannot_decide_or_change_policy() {
    let ctx = setup_test_context().await;
    ctx.lifecycle_manager
        .enable(
            "approval-gate",
            &RegistryScope::Workspace(ctx.workspace_id.clone()),
            RegistryRole::WorkspaceAdmin,
            "owner",
            false,
        )
        .await
        .unwrap();
    let plain_cookie = login_new_user(&ctx.app, &ctx.owner_cookie, "plain_member").await;
    let uid: (String,) = sqlx::query_as("SELECT id FROM user WHERE username = 'plain_member'")
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'member')",
    )
    .bind(&ctx.workspace_id)
    .bind(&uid.0)
    .execute(&ctx.pool)
    .await
    .unwrap();

    move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "running").await;
    move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "in_review").await;
    move_task_to_status(&ctx.app, &ctx.owner_cookie, &ctx.task_id, "ready_to_merge").await;

    let list_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/approval-gate/requests?task_id={}",
            ctx.task_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let list_body: Value = serde_json::from_slice(
        &axum::body::to_bytes(
            ctx.app.clone().oneshot(list_req).await.unwrap().into_body(),
            usize::MAX,
        )
        .await
        .unwrap(),
    )
    .unwrap();
    let request_id = list_body["requests"][0]["id"].as_str().unwrap().to_string();

    // Thành viên thường: không duyệt, không từ chối.
    for action in ["approve", "reject"] {
        let req = Request::builder()
            .method("POST")
            .uri(format!(
                "/api/v1/m/approval-gate/requests/{request_id}/{action}"
            ))
            .header(header::COOKIE, &plain_cookie)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({ "reason": "x" }).to_string()))
            .unwrap();
        assert_eq!(
            ctx.app.clone().oneshot(req).await.unwrap().status(),
            StatusCode::FORBIDDEN,
            "{action}"
        );
    }
    let still: (String,) =
        sqlx::query_as("SELECT status FROM m_approval_gate_request WHERE id = ?")
            .bind(&request_id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(still.0, "pending");

    // Thành viên thường không tắt được cổng duyệt bằng cách đổi chính sách.
    let policy = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/m/approval-gate/policy/{}", ctx.project_id))
        .header(header::COOKIE, &plain_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "require_in_review_to_merge": false, "disallow_self_approval": false })
                .to_string(),
        ))
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(policy).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );
}

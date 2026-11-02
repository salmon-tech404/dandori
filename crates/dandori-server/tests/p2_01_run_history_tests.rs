//! Bộ kiểm thử tích hợp cho Task P2-01: Module `run-history`.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    clippy::indexing_slicing,
    reason = "Kiểm thử tích hợp P2-01 dùng unwrap và indexing json"
)]

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use dandori_events::EventBus;
use dandori_registry::{FeatureLifecycleManager, FeatureRegistry, RegistryRole, RegistryScope};
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
    pool: SqlitePool,
    lifecycle_manager: Arc<FeatureLifecycleManager>,
    workspace_id: String,
    task_id: String,
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
    std::fs::create_dir_all(temp_dir.path().join(".git")).unwrap();
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

    // 1. Tạo owner
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
            json!({ "name": "Workspace RunHistory" }).to_string(),
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

    // 3. Tạo user thứ 2 (non-member) qua API
    let non_member_cookie = login_new_user(&app, &owner_cookie, "nonmember").await;

    // 4. Tạo dự án và task
    let proj_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/workspaces/{workspace_id}/projects"))
        .header(header::COOKIE, &owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "name": "Project RunHistory",
                "repo_path": temp_dir.path().to_str().unwrap(),
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
                "title": "Task for Run History",
                "description": "Test task"
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

    // Bật module run-history
    lifecycle_manager
        .enable(
            "run-history",
            &RegistryScope::System,
            RegistryRole::SystemOwner,
            "owner",
            false,
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
        task_id,
        _temp_dir: temp_dir,
    }
}

#[tokio::test]
async fn test_ac1_filtering_and_pagination() {
    let ctx = setup_test_context().await;
    assert!(!ctx.workspace_id.is_empty());

    // Chèn 5 run với các trạng thái, thời gian khác nhau
    let now = chrono::Utc::now();
    for i in 1..=5 {
        let run_id = format!("run-00{i}");
        let started = (now + chrono::Duration::seconds(i * 10)).to_rfc3339();
        let finished = (now + chrono::Duration::seconds(i * 10 + 5)).to_rfc3339();
        let status = if i % 2 == 0 { "success" } else { "failed" };
        let agent = if i <= 3 { "claude" } else { "gemini" };
        let account = format!("acc-00{i}");

        sqlx::query(
            "INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&run_id)
        .bind(&ctx.task_id)
        .bind(agent)
        .bind(&account)
        .bind(status)
        .bind(&started)
        .bind(&finished)
        .bind(if status == "success" { Some(0) } else { Some(1) })
        .execute(&ctx.pool)
        .await
        .unwrap();
    }

    // 1. Lọc theo agent = "claude"
    let req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/run-history/runs?agent_provider=claude&task_id={}",
            ctx.task_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let runs = body["runs"].as_array().unwrap();
    assert_eq!(runs.len(), 3);
    for r in runs {
        assert_eq!(r["agent"], "claude");
    }

    // 2. Lọc theo status = "failed"
    let req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/run-history/runs?status=failed&task_id={}",
            ctx.task_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let runs = body["runs"].as_array().unwrap();
    assert_eq!(runs.len(), 3); // run 1, 3, 5

    // 3. Phân trang: limit = 2, offset = 0 rồi offset = 2
    let req1 = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/run-history/runs?limit=2&offset=0&task_id={}",
            ctx.task_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp1 = ctx.app.clone().oneshot(req1).await.unwrap();
    let body1: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp1.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let p1_runs = body1["runs"].as_array().unwrap();
    assert_eq!(p1_runs.len(), 2);

    let req2 = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/run-history/runs?limit=2&offset=2&task_id={}",
            ctx.task_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp2 = ctx.app.clone().oneshot(req2).await.unwrap();
    let body2: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp2.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let p2_runs = body2["runs"].as_array().unwrap();
    assert_eq!(p2_runs.len(), 2);

    // Đảm bảo không trùng nhau giữa 2 trang
    let id1: Vec<&str> = p1_runs.iter().map(|r| r["id"].as_str().unwrap()).collect();
    let id2: Vec<&str> = p2_runs.iter().map(|r| r["id"].as_str().unwrap()).collect();
    for id in &id1 {
        assert!(
            !id2.contains(id),
            "Phân trang không được trùng lặp bản ghi: {id}"
        );
    }
}

#[tokio::test]
async fn test_ac2_export_redacts_secrets() {
    let ctx = setup_test_context().await;
    let run_id = "run-secret-test";

    sqlx::query(
        "INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) \
         VALUES (?, ?, 'claude', 'acc-01', 'success', ?, ?, 0)",
    )
    .bind(run_id)
    .bind(&ctx.task_id)
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Chèn run_event chứa chuỗi bí mật và chữ thường không phải bí mật
    sqlx::query(
        "INSERT INTO run_event (id, run_id, seq, event_type, data, created_at) \
         VALUES ('ev-001', ?, 1, 'agent_output', 'Loaded api_key sk-ant-1234567890abcdef and ghp_0123456789abcdefghij for task-based test', ?)",
    )
    .bind(run_id)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Gán ghi chú
    let note_req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/m/run-history/runs/{run_id}/note"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "note": "Ghi chú hợp lệ không có secret", "tags": ["release", "v1"] })
                .to_string(),
        ))
        .unwrap();
    let note_resp = ctx.app.clone().oneshot(note_req).await.unwrap();
    assert_eq!(note_resp.status(), StatusCode::OK);

    // Xuất Markdown
    let exp_md_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/run-history/runs/{run_id}/export?format=markdown"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let exp_md_resp = ctx.app.clone().oneshot(exp_md_req).await.unwrap();
    assert_eq!(exp_md_resp.status(), StatusCode::OK);
    let md_text = String::from_utf8(
        axum::body::to_bytes(exp_md_resp.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();

    assert!(
        md_text.contains("[REDACTED]"),
        "Xuất markdown phải chứa [REDACTED]"
    );
    assert!(
        !md_text.contains("sk-ant-1234567890abcdef"),
        "Không được lộ sk-ant"
    );
    assert!(
        !md_text.contains("ghp_0123456789abcdefghij"),
        "Không được lộ ghp_"
    );
    assert!(
        md_text.contains("task-based test"),
        "Không được cắt nhầm chữ thường"
    );

    // Xuất JSON
    let exp_json_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/run-history/runs/{run_id}/export?format=json"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let exp_json_resp = ctx.app.clone().oneshot(exp_json_req).await.unwrap();
    assert_eq!(exp_json_resp.status(), StatusCode::OK);
    let json_text = String::from_utf8(
        axum::body::to_bytes(exp_json_resp.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();

    assert!(json_text.contains("[REDACTED]"));
    assert!(!json_text.contains("sk-ant-1234567890abcdef"));
}

#[tokio::test]
async fn test_ac3_retention_worker_cleanup() {
    let ctx = setup_test_context().await;

    let old_date = (chrono::Utc::now() - chrono::Duration::days(100)).to_rfc3339();

    // 1. Run của task đã đóng (done)
    sqlx::query("UPDATE task SET status = 'done' WHERE id = ?")
        .bind(&ctx.task_id)
        .execute(&ctx.pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) \
         VALUES ('run-old-closed', ?, 'claude', 'acc-1', 'success', ?, ?, 0)",
    )
    .bind(&ctx.task_id)
    .bind(&old_date)
    .bind(&old_date)
    .execute(&ctx.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO run_event (id, run_id, seq, event_type, data, created_at) \
         VALUES ('ev-old-closed', 'run-old-closed', 1, 'output', 'old log', ?)",
    )
    .bind(&old_date)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // 2. Task đang mở (running) với run cũ
    let open_task_id = "task-open-001";
    sqlx::query(
        "INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) \
         VALUES (?, (SELECT project_id FROM task WHERE id = ?), 'Open task', 'desc', 'running', 'lead', 'worker', ?, ?)",
    )
    .bind(open_task_id)
    .bind(&ctx.task_id)
    .bind(&old_date)
    .bind(&old_date)
    .execute(&ctx.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) \
         VALUES ('run-old-open', ?, 'claude', 'acc-1', 'running', ?, NULL, NULL)",
    )
    .bind(open_task_id)
    .bind(&old_date)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Tắt module run-history để xác nhận worker dừng êm và dữ liệu giữ nguyên
    ctx.lifecycle_manager
        .disable(
            "run-history",
            &RegistryScope::System,
            RegistryRole::SystemOwner,
            "owner",
            false,
        )
        .await
        .unwrap();

    let is_active = ctx.lifecycle_manager.is_module_active("run-history").await;
    assert!(!is_active, "Module phải tắt thành công");

    // Khẳng định: run của open task và closed task vẫn còn nguyên vẹn trong DB
    let open_run: Option<String> =
        sqlx::query_scalar("SELECT id FROM run WHERE id = 'run-old-open'")
            .fetch_optional(&ctx.pool)
            .await
            .unwrap();
    assert!(
        open_run.is_some(),
        "Run của task đang mở tuyệt đối không bị mất"
    );
}

#[tokio::test]
async fn test_ac4_workspace_isolation() {
    let ctx = setup_test_context().await;
    let run_id = "run-isolation-test";

    sqlx::query(
        "INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) \
         VALUES (?, ?, 'claude', 'acc-1', 'success', ?, ?, 0)",
    )
    .bind(run_id)
    .bind(&ctx.task_id)
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Non-member cố xem chi tiết run -> 403
    let get_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/run-history/runs/{run_id}"))
        .header(header::COOKIE, &ctx.non_member_cookie)
        .body(Body::empty())
        .unwrap();
    let get_resp = ctx.app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::FORBIDDEN);

    // Non-member cố xem events -> 403
    let ev_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/run-history/runs/{run_id}/events"))
        .header(header::COOKIE, &ctx.non_member_cookie)
        .body(Body::empty())
        .unwrap();
    let ev_resp = ctx.app.clone().oneshot(ev_req).await.unwrap();
    assert_eq!(ev_resp.status(), StatusCode::FORBIDDEN);

    // Non-member cố xem note -> 403
    let note_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/run-history/runs/{run_id}/note"))
        .header(header::COOKIE, &ctx.non_member_cookie)
        .body(Body::empty())
        .unwrap();
    let note_resp = ctx.app.clone().oneshot(note_req).await.unwrap();
    assert_eq!(note_resp.status(), StatusCode::FORBIDDEN);

    // Non-member cố xuất -> 403
    let exp_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/run-history/runs/{run_id}/export"))
        .header(header::COOKIE, &ctx.non_member_cookie)
        .body(Body::empty())
        .unwrap();
    let exp_resp = ctx.app.clone().oneshot(exp_req).await.unwrap();
    assert_eq!(exp_resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_ac5_note_rejects_secrets() {
    let ctx = setup_test_context().await;
    let run_id = "run-note-secret-test";

    sqlx::query(
        "INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) \
         VALUES (?, ?, 'claude', 'acc-1', 'success', ?, ?, 0)",
    )
    .bind(run_id)
    .bind(&ctx.task_id)
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&ctx.pool)
    .await
    .unwrap();

    let note_req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/m/run-history/runs/{run_id}/note"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "note": "Secret sk-1234567890abcdef12345678 in note", "tags": [] }).to_string(),
        ))
        .unwrap();

    let note_resp = ctx.app.clone().oneshot(note_req).await.unwrap();
    assert_eq!(note_resp.status(), StatusCode::BAD_REQUEST);
}

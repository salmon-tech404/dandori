//! Các bài kiểm thử tích hợp cho Task P2-11: Module `api-key-fallback`.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P2-11 dùng unwrap/expect và có fixture kiểm tra nhiều tiêu chí"
)]

use std::str::FromStr;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use dandori_agent_port::AgentRole;
use dandori_db::repo::ProjectRepo;
use dandori_events::{EventBus, EventFilter};
use dandori_module_api::{MemoryKeyStore, SqliteSecretStorage, TaskRunner, Vault, VaultError};
use dandori_registry::FeatureRegistry;
use dandori_registry::lifecycle::FeatureLifecycleManager;
use dandori_server::app::create_app;
use dandori_server::config::ServerConfig;
use dandori_server::run_service::RunService;
use dandori_server::state::AppState;
use dandori_types::{Project, ProjectId, SecretId, Task, TaskId, WorkspaceId};
use serde_json::{Value, json};
use sqlx::{Row, SqlitePool};
use tempfile::tempdir;
use tower::ServiceExt;

struct TestContext {
    app: axum::Router,
    pool: SqlitePool,
    #[expect(dead_code, reason = "Giữ lifecycle manager trong bộ nhớ")]
    lifecycle_manager: Arc<FeatureLifecycleManager>,
    event_bus: Arc<EventBus>,
    owner_cookie: String,
    run_service: Arc<RunService>,
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
    run_service.register_adapter("claude", fake_adapter).await;

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

    // Thiết lập tài khoản Owner
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

    TestContext {
        app,
        pool,
        lifecycle_manager,
        event_bus,
        owner_cookie,
        run_service,
        temp_dir,
    }
}

async fn enable_modules(ctx: &TestContext, module_ids: &[&str]) {
    for mod_id in module_ids {
        let enable_req = Request::builder()
            .method("POST")
            .uri("/api/v1/features/apply")
            .header(header::COOKIE, &ctx.owner_cookie)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "module_id": mod_id,
                    "scope": "system",
                    "target_id": "system",
                    "enable": true,
                    "risk_acknowledged": true
                })
                .to_string(),
            ))
            .unwrap();
        let enable_resp = ctx.app.clone().oneshot(enable_req).await.unwrap();
        let st = enable_resp.status();
        let body = axum::body::to_bytes(enable_resp.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(
            st,
            StatusCode::OK,
            "Failed to enable {mod_id}: {}",
            String::from_utf8_lossy(&body)
        );
    }
}

async fn init_test_git_repo(path: &std::path::Path) {
    let client = dandori_git::GitClient::new(path);
    client.init_repo("main").await.expect("git init");
    client
        .config("user.name", "Test Committer")
        .await
        .expect("git config user.name");
    client
        .config("user.email", "committer@example.com")
        .await
        .expect("git config user.email");
    std::fs::write(path.join("README.md"), "# Test Project\n").expect("write file");
    client
        .commit_all("Initial baseline commit")
        .await
        .expect("git commit");
}

async fn setup_test_workspace_and_project(
    pool: &SqlitePool,
    repo_path: &str,
) -> (WorkspaceId, ProjectId, dandori_types::UserId) {
    let ws_id = WorkspaceId::new();
    let prj_id = ProjectId::new();
    let ws_str = ws_id.to_string();
    let now = Utc::now().to_rfc3339();

    let owner_user = sqlx::query("SELECT id FROM user WHERE username = 'owner'")
        .fetch_one(pool)
        .await
        .unwrap();
    let user_id_str: String = owner_user.get("id");
    let user_id = dandori_types::UserId::from_str(&user_id_str).unwrap();

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'WS 1', ?)")
        .bind(&ws_str)
        .bind(&now)
        .execute(pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'admin')",
    )
    .bind(&ws_str)
    .bind(&user_id_str)
    .execute(pool)
    .await
    .unwrap();

    let prj = Project {
        id: prj_id,
        workspace_id: ws_id,
        name: "Test Project".to_string(),
        repo_path: repo_path.to_string(),
        default_branch: "main".to_string(),
        created_at: Utc::now(),
    };
    ProjectRepo::new(pool).create_project(&prj).await.unwrap();

    (ws_id, prj_id, user_id)
}

#[tokio::test]
async fn test_ac1_api_key_not_in_db_log_events_only_child_process_env() {
    let ctx = setup_test_context().await;
    enable_modules(&ctx, &["api-key-fallback"]).await;

    let secret_raw = "test-key-very-confidential-secret-1234567890";

    // 1. Gửi yêu cầu tạo tài khoản API key
    let create_payload = json!({
        "provider": "claude",
        "label": "Claude API Key Primary",
        "api_key": secret_raw,
        "spending_limit": {
            "period": "daily",
            "limit_type": "currency",
            "limit_value": 20.0,
            "alert_threshold": 80.0
        }
    });

    let req = Request::post("/api/v1/m/api-key-fallback/accounts")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();

    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();
    let acc_id = json_res["id"].as_str().unwrap().to_string();

    // Xác nhận API trả về masked_key che giấu, không lộ raw key
    assert_eq!(json_res["masked_key"], "••••••••••••7890");
    assert!(!json_res.to_string().contains(secret_raw));

    // 2. Quét kiểm tra DB: Khóa raw tuyệt đối không xuất hiện dưới dạng plaintext ở bất kỳ cột nào
    let secret_row = sqlx::query(
        "SELECT ciphertext, nonce, label FROM secret WHERE id = (SELECT secret_id FROM agent_account WHERE id = ?)"
    )
    .bind(&acc_id)
    .fetch_one(&ctx.pool)
    .await
    .unwrap();

    let label: String = secret_row.get("label");
    let ciphertext: Vec<u8> = secret_row.get("ciphertext");

    // Nhãn trong secret là chuỗi đã che
    assert_eq!(label, "••••••••••••7890");
    // Ciphertext không chứa chuỗi bí mật dạng raw bytes
    assert!(
        !ciphertext
            .windows(secret_raw.len())
            .any(|w| w == secret_raw.as_bytes())
    );

    // 3. GET /accounts cũng chỉ trả về masked_key
    let req_list = Request::get("/api/v1/m/api-key-fallback/accounts")
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();

    let res_list = ctx.app.oneshot(req_list).await.unwrap();
    assert_eq!(res_list.status(), StatusCode::OK);
    let body_list = axum::body::to_bytes(res_list.into_body(), usize::MAX)
        .await
        .unwrap();
    let list_json: Value = serde_json::from_slice(&body_list).unwrap();
    assert_eq!(list_json["accounts"][0]["masked_key"], "••••••••••••7890");
    assert!(!list_json.to_string().contains(secret_raw));
}

#[tokio::test]
async fn test_ac2_cannot_enable_without_spending_limit_and_stops_on_cap() {
    let ctx = setup_test_context().await;
    enable_modules(&ctx, &["api-key-fallback"]).await;

    // 1. Tạo tài khoản KHÔNG có spending_limit
    let create_payload = json!({
        "provider": "claude",
        "label": "Claude Key No Limit",
        "api_key": "test-key-without-limit-1111",
        "spending_limit": null
    });

    let req = Request::post("/api/v1/m/api-key-fallback/accounts")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();

    let res = ctx.app.clone().oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let acc_id = json["id"].as_str().unwrap().to_string();
    assert_eq!(json["enabled"], false);

    // 2. Thử bật kích hoạt tài khoản khi chưa có trần -> BỊ TỪ CHỐI (HTTP 400)
    let req_toggle = Request::patch(format!(
        "/api/v1/m/api-key-fallback/accounts/{acc_id}/toggle"
    ))
    .header(header::COOKIE, &ctx.owner_cookie)
    .body(Body::empty())
    .unwrap();

    let res_toggle = ctx.app.clone().oneshot(req_toggle).await.unwrap();
    assert_eq!(res_toggle.status(), StatusCode::BAD_REQUEST);

    // 3. Đặt trần chi tiêu bắt buộc (limit = 10.0 currency, alert = 80%)
    let limit_payload = json!({
        "period": "daily",
        "limit_type": "currency",
        "limit_value": 10.0,
        "alert_threshold": 80.0
    });
    let req_limit = Request::put(format!(
        "/api/v1/m/api-key-fallback/accounts/{acc_id}/spending-limit"
    ))
    .header(header::COOKIE, &ctx.owner_cookie)
    .header(header::CONTENT_TYPE, "application/json")
    .body(Body::from(limit_payload.to_string()))
    .unwrap();

    let res_limit = ctx.app.clone().oneshot(req_limit).await.unwrap();
    assert_eq!(res_limit.status(), StatusCode::OK);

    // 4. Kích hoạt tài khoản -> THÀNH CÔNG
    let req_toggle2 = Request::patch(format!(
        "/api/v1/m/api-key-fallback/accounts/{acc_id}/toggle"
    ))
    .header(header::COOKIE, &ctx.owner_cookie)
    .body(Body::empty())
    .unwrap();

    let res_toggle2 = ctx.app.clone().oneshot(req_toggle2).await.unwrap();
    assert_eq!(res_toggle2.status(), StatusCode::OK);

    // Đăng ký nhận sự kiện cảnh báo
    let mut rx = ctx.event_bus.subscribe(EventFilter::all());

    // 5. Ghi nhận sử dụng chạm ngưỡng 85% (cost = 8.5) -> Phát cảnh báo vượt ngưỡng
    let usage_payload1 = json!({ "cost": 8.5 });
    let req_usage1 = Request::post(format!(
        "/api/v1/m/api-key-fallback/accounts/{acc_id}/record-usage"
    ))
    .header(header::COOKIE, &ctx.owner_cookie)
    .header(header::CONTENT_TYPE, "application/json")
    .body(Body::from(usage_payload1.to_string()))
    .unwrap();

    let res_usage1 = ctx.app.clone().oneshot(req_usage1).await.unwrap();
    assert_eq!(res_usage1.status(), StatusCode::OK);

    let event1 = rx.recv().await.unwrap();
    assert_eq!(
        event1.payload.kind(),
        dandori_types::EventKind::NotificationRequested
    );

    // 6. Ghi nhận chạm trần chi tiêu (cost = 2.0 -> tổng 10.5 >= 10.0) -> Tự động dừng tài khoản
    let usage_payload2 = json!({ "cost": 2.0 });
    let req_usage2 = Request::post(format!(
        "/api/v1/m/api-key-fallback/accounts/{acc_id}/record-usage"
    ))
    .header(header::COOKIE, &ctx.owner_cookie)
    .header(header::CONTENT_TYPE, "application/json")
    .body(Body::from(usage_payload2.to_string()))
    .unwrap();

    let res_usage2 = ctx.app.clone().oneshot(req_usage2).await.unwrap();
    assert_eq!(res_usage2.status(), StatusCode::OK);

    let event2 = rx.recv().await.unwrap();
    assert_eq!(
        event2.payload.kind(),
        dandori_types::EventKind::NotificationRequested
    );

    // Xác nhận tài khoản trong DB đã bị tắt (enabled = 0)
    let acc_status = sqlx::query("SELECT enabled, exhausted_until FROM agent_account WHERE id = ?")
        .bind(&acc_id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    let enabled_val: i64 = acc_status.get("enabled");
    let exhausted_val: Option<String> = acc_status.get("exhausted_until");
    assert_eq!(enabled_val, 0);
    assert!(exhausted_val.is_some());

    // 7. Cố bật lại tài khoản khi đã chạm trần -> BỊ TỪ CHỐI
    let req_toggle3 = Request::patch(format!(
        "/api/v1/m/api-key-fallback/accounts/{acc_id}/toggle"
    ))
    .header(header::COOKIE, &ctx.owner_cookie)
    .body(Body::empty())
    .unwrap();

    let res_toggle3 = ctx.app.oneshot(req_toggle3).await.unwrap();
    assert_eq!(res_toggle3.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_ac3_fallback_policy_cli_vs_api_key() {
    let ctx = setup_test_context().await;
    enable_modules(
        &ctx,
        &["worktree", "subscription-connect", "api-key-fallback"],
    )
    .await;

    let repo_temp = tempdir().unwrap();
    init_test_git_repo(repo_temp.path()).await;
    let repo_path = repo_temp.path().to_string_lossy().to_string();

    let (_, prj_id, user_id) = setup_test_workspace_and_project(&ctx.pool, &repo_path).await;

    // 1. Tạo tài khoản CLI chính thức (ready, priority 100)
    let cli_acc_id = "00000000-0000-0000-0000-000000000001";
    sqlx::query(
        "INSERT INTO agent_account (id, provider, label, home_dir, auth_type, status, enabled, priority) \
         VALUES (?, 'claude', 'Claude CLI Account', '', 'cli', 'ready', 1, 100)"
    )
    .bind(cli_acc_id)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // 2. Tạo tài khoản API key dự phòng (ready, priority 150)
    let api_acc_id = "00000000-0000-0000-0000-000000000002";
    let storage = SqliteSecretStorage::new(ctx.pool.clone());
    let key_store = MemoryKeyStore::shared();
    let vault = Vault::new(storage, key_store);
    let secret_id = vault
        .put(
            "masked",
            dandori_module_api::SecretBox::from("test-key-fallback-for-claude"),
        )
        .await
        .unwrap();
    let sec_str = secret_id.to_string();

    sqlx::query(
        "INSERT INTO agent_account (id, provider, label, home_dir, auth_type, secret_id, status, enabled, priority) \
         VALUES (?, 'claude', 'Claude Fallback API', '', 'api_key', ?, 'ready', 1, 150)"
    )
    .bind(api_acc_id)
    .bind(&sec_str)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Gán spending limit hợp lệ cho API key
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO m_api_key_fallback_spending_limit \
         (account_id, period, limit_type, limit_value, used_value, period_start, alert_threshold, created_at, updated_at) \
         VALUES (?, 'daily', 'currency', 50.0, 0.0, ?, 80.0, ?, ?)"
    )
    .bind(api_acc_id)
    .bind(&now)
    .bind(&now)
    .bind(&now)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Tạo Task
    let task_id = TaskId::new();
    let temp_dir = tempdir().unwrap();
    let workdir_path = temp_dir.path().to_string_lossy().to_string();

    let task = Task {
        id: task_id,
        project_id: prj_id,
        title: "Test Task AC3".to_string(),
        description: "Test".to_string(),
        status: "backlog".to_string(),
        lead_agent: String::new(),
        worker_agent: String::new(),
        workdir_path: Some(workdir_path),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    dandori_db::TaskRepo::new(&ctx.pool)
        .create_task(&task)
        .await
        .unwrap();

    // 3. Khi CLI account còn khả dụng: RunService PHẢI chọn CLI account (không chọn API key)
    let run1 = ctx
        .run_service
        .start_run(task_id, AgentRole::Worker, user_id, vec![])
        .await
        .unwrap();
    assert_eq!(run1.account, "Claude CLI Account");

    // Đánh dấu lần chạy 1 kết thúc và reset task status về backlog để test lần 2
    sqlx::query("UPDATE run SET status = 'success' WHERE id = ?")
        .bind(run1.id.to_string())
        .execute(&ctx.pool)
        .await
        .unwrap();
    sqlx::query("UPDATE task SET status = 'backlog' WHERE id = ?")
        .bind(task_id.to_string())
        .execute(&ctx.pool)
        .await
        .unwrap();

    // 4. Đánh dấu CLI account hết hạn mức (exhausted_until trong tương lai)
    let future_exhausted = (Utc::now() + chrono::Duration::hours(4)).to_rfc3339();
    sqlx::query("UPDATE agent_account SET exhausted_until = ? WHERE id = ?")
        .bind(&future_exhausted)
        .bind(cli_acc_id)
        .execute(&ctx.pool)
        .await
        .unwrap();

    // Dự án CHƯA bật fallback -> start_run phải thất bại (không tự ý dùng API key)
    let err_run = ctx
        .run_service
        .start_run(task_id, AgentRole::Worker, user_id, vec![])
        .await;
    assert!(err_run.is_err());

    // 5. Dự án BẬT dự phòng (fallback_enabled = true)
    let update_cfg_payload = json!({
        "fallback_enabled": true,
        "allowed_accounts": []
    });
    let req_cfg = Request::put(format!(
        "/api/v1/m/api-key-fallback/projects/{prj_id}/config"
    ))
    .header(header::COOKIE, &ctx.owner_cookie)
    .header(header::CONTENT_TYPE, "application/json")
    .body(Body::from(update_cfg_payload.to_string()))
    .unwrap();

    let res_cfg = ctx.app.oneshot(req_cfg).await.unwrap();
    assert_eq!(res_cfg.status(), StatusCode::OK);

    // start_run lại -> BÂY GIỜ PHẢI CHỌN TÀI KHOẢN API KEY DỰ PHÒNG THÀNH CÔNG!
    let run2 = ctx
        .run_service
        .start_run(task_id, AgentRole::Worker, user_id, vec![])
        .await
        .unwrap();
    assert_eq!(run2.account, "Claude Fallback API");
}

#[tokio::test]
async fn test_ac4_delete_account_makes_reveal_fail() {
    let ctx = setup_test_context().await;
    enable_modules(&ctx, &["api-key-fallback"]).await;

    // 1. Tạo tài khoản API key
    let create_payload = json!({
        "provider": "claude",
        "label": "Key To Be Deleted",
        "api_key": "test-key-to-be-deleted-safely-9999",
        "spending_limit": {
            "period": "daily",
            "limit_type": "currency",
            "limit_value": 5.0,
            "alert_threshold": 80.0
        }
    });

    let req = Request::post("/api/v1/m/api-key-fallback/accounts")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();

    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let acc_id = json["id"].as_str().unwrap().to_string();

    // Lấy secret_id từ DB
    let acc_row = sqlx::query("SELECT secret_id FROM agent_account WHERE id = ?")
        .bind(&acc_id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    let sec_id_str: String = acc_row.get("secret_id");
    let sec_id = SecretId::from_str(&sec_id_str).unwrap();

    // 2. Trước khi xóa: reveal thành công
    let storage = SqliteSecretStorage::new(ctx.pool.clone());
    let key_store = MemoryKeyStore::shared();
    let vault = Vault::new(storage, key_store);

    let revealed = vault.reveal(&sec_id).await;
    assert!(revealed.is_ok());
    assert_eq!(
        revealed.unwrap().expose_secret(),
        "test-key-to-be-deleted-safely-9999"
    );

    // 3. Gọi DELETE /accounts/:id
    let req_del = Request::delete(format!("/api/v1/m/api-key-fallback/accounts/{acc_id}"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();

    let res_del = ctx.app.oneshot(req_del).await.unwrap();
    assert_eq!(res_del.status(), StatusCode::OK);

    // 4. Sau khi xóa: reveal PHẢI THẤT BẠI (NotFound)
    let revealed_after = vault.reveal(&sec_id).await;
    assert!(matches!(revealed_after, Err(VaultError::NotFound(_))));

    // Và bản ghi trong agent_account cũng biến mất
    let check_acc = sqlx::query("SELECT id FROM agent_account WHERE id = ?")
        .bind(&acc_id)
        .fetch_optional(&ctx.pool)
        .await
        .unwrap();
    assert!(check_acc.is_none());
}

#[tokio::test]
async fn test_ac5_workspace_isolation_bola() {
    let ctx = setup_test_context().await;
    enable_modules(&ctx, &["api-key-fallback"]).await;

    let repo_temp = tempdir().unwrap();
    init_test_git_repo(repo_temp.path()).await;
    let repo_path = repo_temp.path().to_string_lossy().to_string();

    let (_, prj_id, _) = setup_test_workspace_and_project(&ctx.pool, &repo_path).await;

    // Tạo user2 "stranger" không thuộc workspace
    let stranger_id = dandori_types::UserId::new();
    let stranger_str = stranger_id.to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, 'stranger', 'hash', 'user', ?)"
    )
    .bind(&stranger_str).bind(&now)
    .execute(&ctx.pool).await.unwrap();

    let token = "stranger_session_token";
    let token_hash = dandori_db::hash_session_token(token);
    let expires_at = (Utc::now() + chrono::Duration::hours(24)).to_rfc3339();
    sqlx::query("INSERT INTO session (id, user_id, expires_at, token_hash) VALUES (?, ?, ?, ?)")
        .bind(dandori_types::SessionId::new().to_string())
        .bind(&stranger_str)
        .bind(expires_at)
        .bind(token_hash)
        .execute(&ctx.pool)
        .await
        .unwrap();

    let stranger_cookie = format!("dandori_session={token}");

    // 1. Stranger cố xem cấu hình project của người khác -> 403 Forbidden
    let req_get = Request::get(format!(
        "/api/v1/m/api-key-fallback/projects/{prj_id}/config"
    ))
    .header(header::COOKIE, &stranger_cookie)
    .body(Body::empty())
    .unwrap();
    let res_get = ctx.app.clone().oneshot(req_get).await.unwrap();
    assert_eq!(res_get.status(), StatusCode::FORBIDDEN);

    // 2. Stranger cố sửa cấu hình project của người khác -> 403 Forbidden
    let req_put = Request::put(format!(
        "/api/v1/m/api-key-fallback/projects/{prj_id}/config"
    ))
    .header(header::COOKIE, &stranger_cookie)
    .header(header::CONTENT_TYPE, "application/json")
    .body(Body::from(json!({ "fallback_enabled": true }).to_string()))
    .unwrap();
    let res_put = ctx.app.clone().oneshot(req_put).await.unwrap();
    assert_eq!(res_put.status(), StatusCode::FORBIDDEN);

    // 3. Stranger (role user, không phải admin) cố tạo API key hệ thống -> 403 Forbidden
    let req_create = Request::post("/api/v1/m/api-key-fallback/accounts")
        .header(header::COOKIE, &stranger_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "claude",
                "label": "Hacked Key",
                "api_key": "sk-ant-test",
                "spending_limit": {
                    "period": "daily",
                    "limit_type": "currency",
                    "limit_value": 10.0,
                    "alert_threshold": 80.0
                }
            })
            .to_string(),
        ))
        .unwrap();
    let res_create = ctx.app.oneshot(req_create).await.unwrap();
    assert_eq!(res_create.status(), StatusCode::FORBIDDEN);
}

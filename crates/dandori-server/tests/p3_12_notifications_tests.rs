//! Các bài kiểm thử tích hợp cho Task P3-12: Module `notifications` (Telegram, Discord, Slack).

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P3-12 dùng unwrap và kiểm tra nhiều tiêu chí nghiệm thu"
)]

use std::sync::Arc;
use std::time::Duration;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use dandori_events::EventBus;
use dandori_module_api::set_allow_local_endpoints_for_tests;
use dandori_registry::FeatureRegistry;
use dandori_registry::lifecycle::FeatureLifecycleManager;
use dandori_server::app::create_app;
use dandori_server::config::ServerConfig;
use dandori_server::run_service::RunService;
use dandori_server::state::AppState;
use dandori_types::{Envelope, EventPayload, EventScope, ProjectId, TaskId, WorkspaceId};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use tempfile::tempdir;
use tower::ServiceExt;

struct TestContext {
    app: axum::Router,
    pool: SqlitePool,
    event_bus: Arc<EventBus>,
    owner_cookie: String,
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
    let workspace_id = WorkspaceId::new().to_string();
    let project_id = ProjectId::new().to_string();

    // 2. Setup Outsider User trực tiếp
    let outsider_user_id = "user_outsider_99";
    sqlx::query("INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, 'outsider', 'hash', 'user', ?)")
        .bind(outsider_user_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let outsider_token = "outsider_token_notifications_99";
    let outsider_token_hash = dandori_server::auth::hash_token(outsider_token);
    let expires = (chrono::Utc::now() + chrono::Duration::days(1)).to_rfc3339();

    sqlx::query(
        "INSERT INTO session (id, user_id, expires_at, token_hash) VALUES ('sess_outsider_notif', ?, ?, ?)",
    )
    .bind(outsider_user_id)
    .bind(&expires)
    .bind(&outsider_token_hash)
    .execute(&pool)
    .await
    .unwrap();
    let outsider_cookie = format!("dandori_session={outsider_token}");

    // 3. Tạo Workspace và gán Owner là admin
    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'Notifications WS', ?)")
        .bind(&workspace_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let owner_user_id: String = sqlx::query_scalar("SELECT id FROM user WHERE username = 'owner'")
        .fetch_one(&pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'admin')",
    )
    .bind(&workspace_id)
    .bind(&owner_user_id)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'Notifications PRJ', '/tmp/repo', 'main', ?)")
        .bind(&project_id)
        .bind(&workspace_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    // 4. Kích hoạt module notifications
    let enable_req = Request::builder()
        .method("POST")
        .uri("/api/v1/features/apply")
        .header(header::COOKIE, &owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "module_id": "notifications",
                "scope": "workspace",
                "target_id": workspace_id,
                "enable": true,
                "risk_acknowledged": true
            })
            .to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(enable_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    TestContext {
        app,
        pool,
        event_bus,
        owner_cookie,
        outsider_cookie,
        workspace_id,
        project_id,
        temp_dir,
    }
}

#[tokio::test]
async fn test_ac1_channels_crud_and_vault_secret_isolation() {
    let ctx = setup_test_context().await;
    let ws = &ctx.workspace_id;
    let bot_token = "123456789:ABCdefGHIjklMNOpqrsTUVwxyz";

    // 1. Owner tạo kênh Telegram
    let create_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/notifications/workspaces/{ws}/notifications/channels"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "name": "DevOps Alerts",
                "channel_type": "telegram",
                "secret": bot_token,
                "chat_id": "-100987654321",
                "notify_on_needs_human": true,
                "notify_on_failed": true
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let channel_id = body_json["id"].as_str().unwrap();

    // Kiểm tra API response: KHÔNG chứa token rõ ràng
    assert_eq!(body_json["has_secret"], true);
    assert!(body_json.get("secret").is_none());
    assert!(body_json.get("vault_secret_id").is_none());

    // Kiểm tra trong database: Cột vault_secret_id lưu dạng `vault:<id>`, không có plaintext token
    let db_row = sqlx::query("SELECT vault_secret_id FROM m_notifications_channel WHERE id = ?")
        .bind(channel_id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    let vault_ref: String = db_row.get("vault_secret_id");
    assert!(vault_ref.starts_with("vault:"));
    assert!(!vault_ref.contains(bot_token));

    // 2. Outsider cố gắng truy cập kênh -> Bị từ chối 403 Forbidden
    let outsider_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/notifications/workspaces/{ws}/notifications/channels"
        ))
        .header(header::COOKIE, &ctx.outsider_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(outsider_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 3. Owner truy cập danh sách kênh -> Thành công 200 OK
    let list_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/notifications/workspaces/{ws}/notifications/channels"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let list_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list_json: serde_json::Value = serde_json::from_slice(&list_bytes).unwrap();
    assert_eq!(list_json.as_array().unwrap().len(), 1);
    assert_eq!(list_json[0]["name"], "DevOps Alerts");
    assert_eq!(list_json[0]["has_secret"], true);
}

#[tokio::test]
async fn test_ac2_ssrf_and_domain_restriction() {
    set_allow_local_endpoints_for_tests(false);
    let ctx = setup_test_context().await;
    let ws = &ctx.workspace_id;

    // Tạo kênh Discord với webhook domain giả mạo bên ngoài
    let create_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/notifications/workspaces/{ws}/notifications/channels"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "name": "Malicious Webhook",
                "channel_type": "discord",
                "secret": "https://malicious-domain.com/webhook/123",
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let channel_id = body_json["id"].as_str().unwrap();

    // Gửi thử nghiệm tới kênh giả mạo -> Phải bị chặn bởi SSRF domain validation (502 Bad Gateway)
    let test_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/notifications/workspaces/{ws}/notifications/channels/{channel_id}/test"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();

    let resp = ctx.app.clone().oneshot(test_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
    let err_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let err_json: serde_json::Value = serde_json::from_slice(&err_bytes).unwrap();
    assert_eq!(err_json["success"], false);
    assert!(
        err_json["error"]
            .as_str()
            .unwrap()
            .contains("không nằm trong danh sách cho phép")
    );
}

#[tokio::test]
async fn test_ac3_ac4_ac5_worker_dedup_and_failure_isolation() {
    let ctx = setup_test_context().await;
    let ws = &ctx.workspace_id;
    let task_id = TaskId::new();

    // 1. Tạo task trong database
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) \
         VALUES (?, ?, 'Fix issue with auth', 'Detailed description', 'needs_human', 'lead', 'worker', ?, ?)",
    )
    .bind(task_id.to_string())
    .bind(&ctx.project_id)
    .bind(&now)
    .bind(&now)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // 2. Tạo kênh qua REST endpoint
    let create_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/notifications/workspaces/{ws}/notifications/channels"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "name": "Alerts Channel",
                "channel_type": "telegram",
                "secret": "dummy-token-12345",
                "chat_id": "123456",
                "notify_on_needs_human": true,
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let channel_id = body_json["id"].as_str().unwrap();

    // 3. Giả lập một bản ghi lịch sử gửi thành công trong vòng 300s
    sqlx::query(
        r#"
        INSERT INTO m_notifications_history (
            id, workspace_id, channel_id, event_type, task_id, title, status, created_at
        ) VALUES (?, ?, ?, 'needs_human', ?, 'Title', 'sent', datetime('now'))
        "#,
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(ws)
    .bind(channel_id)
    .bind(task_id.to_string())
    .execute(&ctx.pool)
    .await
    .unwrap();

    // 4. Phát sự kiện TaskStatusChanged với cùng task_id và trạng thái needs_human
    let ws_uuid = ws.parse::<WorkspaceId>().unwrap();
    let env = Envelope::new(
        chrono::Utc::now(),
        EventScope::workspace(ws_uuid),
        EventPayload::TaskStatusChanged {
            task_id,
            old_status: "in_progress".to_string(),
            new_status: "needs_human".to_string(),
        },
    );

    ctx.event_bus.publish(env);
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Kiểm tra Deduplication: Phải ghi nhận bản ghi với status = 'deduplicated'
    let dedup_row: Option<(String,)> = sqlx::query_as(
        "SELECT status FROM m_notifications_history WHERE channel_id = ? AND status = 'deduplicated'",
    )
    .bind(channel_id)
    .fetch_optional(&ctx.pool)
    .await
    .unwrap();

    assert_eq!(dedup_row.unwrap().0, "deduplicated");

    // Kiểm tra Failure Isolation: Trạng thái của task trong bảng task vẫn là `needs_human`, hoàn toàn không bị ảnh hưởng!
    let task_status: (String,) = sqlx::query_as("SELECT status FROM task WHERE id = ?")
        .bind(task_id.to_string())
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    assert_eq!(task_status.0, "needs_human");
}

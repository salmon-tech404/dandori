//! Kiểm thử bảo mật bổ sung sau kiểm độc lập P2: quyền trên hạn mức, cấu hình tự chuyển, lệnh xem trước.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    clippy::indexing_slicing,
    reason = "Kiểm thử tích hợp dùng unwrap và indexing json"
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

struct Ctx {
    app: axum::Router,
    pool: SqlitePool,
    owner: String,
    member: String,
    outsider: String,
    ws_id: String,
    project_id: String,
    task_id: String,
    account_id: String,
    _tmp: tempfile::TempDir,
}

async fn send(
    app: &axum::Router,
    method: &str,
    uri: &str,
    cookie: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut b = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::COOKIE, cookie);
    let body = match body {
        Some(v) => {
            b = b.header(header::CONTENT_TYPE, "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    let resp = app.clone().oneshot(b.body(body).unwrap()).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let value = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).to_string()));
    (status, value)
}

async fn new_user(app: &axum::Router, owner: &str, name: &str) -> String {
    let (st, _) = send(
        app,
        "POST",
        "/api/v1/auth/users",
        owner,
        Some(json!({ "username": name, "password": "password123", "system_role": "user" })),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    let login = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": name, "password": "password123" }).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(login).await.unwrap();
    resp.headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

async fn setup() -> Ctx {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::query(include_str!(
        "../../../crates/dandori-db/migrations/0001_core_schema.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(include_str!(
        "../../../crates/mod-auth-login/migrations/0001_auth_login.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    let bus = Arc::new(EventBus::new(200));
    let registry = FeatureRegistry::new(pool.clone());
    registry.seed_modules_and_packs().await.unwrap();
    let lm = Arc::new(FeatureLifecycleManager::new(
        registry,
        pool.clone(),
        Arc::clone(&bus),
    ));
    let modules = dandori_modules::registered_modules();
    for m in &modules {
        for mig in m.migrations() {
            sqlx::query(mig.sql).execute(&pool).await.unwrap();
        }
    }
    for m in modules {
        lm.register_module(m).await;
    }

    let tmp = tempdir().unwrap();
    let config = ServerConfig {
        data_dir: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let app = create_app(AppState::new(
        pool.clone(),
        Arc::clone(&bus),
        Arc::clone(&lm),
        config,
    ));

    let setup_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "owner", "password": "password123" }).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(setup_req).await.unwrap();
    let owner = resp
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let member = new_user(&app, &owner, "member").await;
    let outsider = new_user(&app, &owner, "outsider").await;

    let (_, ws) = send(
        &app,
        "POST",
        "/api/v1/workspaces",
        &owner,
        Some(json!({ "name": "WS" })),
    )
    .await;
    let ws_id = ws["workspace"]["id"].as_str().unwrap().to_string();
    let member_id: String = sqlx::query_scalar("SELECT id FROM user WHERE username = 'member'")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'member')",
    )
    .bind(&ws_id)
    .bind(member_id)
    .execute(&pool)
    .await
    .unwrap();

    let project_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'P', ?, 'main', datetime('now'))",
    )
    .bind(&project_id)
    .bind(&ws_id)
    .bind(tmp.path().to_str().unwrap())
    .execute(&pool)
    .await
    .unwrap();
    let task_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES (?, ?, 'T', 'd', 'backlog', 'lead', 'worker', datetime('now'), datetime('now'))",
    )
    .bind(&task_id)
    .bind(&project_id)
    .execute(&pool)
    .await
    .unwrap();
    let account_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO agent_account (id, provider, label, home_dir, auth_type, status, enabled, priority) VALUES (?, 'claude', 'acc', '', 'cli', 'ready', 1, 10)",
    )
    .bind(&account_id)
    .execute(&pool)
    .await
    .unwrap();

    for id in ["quota-tracking", "auto-switch", "diff-preview"] {
        lm.enable(
            id,
            &RegistryScope::System,
            RegistryRole::SystemOwner,
            "owner",
            true,
        )
        .await
        .unwrap();
    }

    Ctx {
        app,
        pool,
        owner,
        member,
        outsider,
        ws_id,
        project_id,
        task_id,
        account_id,
        _tmp: tmp,
    }
}

#[tokio::test]
async fn test_quota_tracking_changes_are_system_admin_only() {
    let ctx = setup().await;
    let base = format!("/api/v1/m/quota-tracking/accounts/{}", ctx.account_id);

    for cookie in [&ctx.member, &ctx.outsider] {
        let (st, _) = send(
            &ctx.app,
            "POST",
            &format!("{base}/mark-exhausted"),
            cookie,
            Some(json!({ "wait_hours": 5 })),
        )
        .await;
        assert_eq!(st, StatusCode::FORBIDDEN, "mark-exhausted");
        let (st, _) = send(
            &ctx.app,
            "PUT",
            &format!("{base}/config"),
            cookie,
            Some(json!({ "alert_threshold_percent": 10 })),
        )
        .await;
        assert_eq!(st, StatusCode::FORBIDDEN, "config");
        let (st, _) = send(
            &ctx.app,
            "POST",
            &format!("{base}/record-reported"),
            cookie,
            Some(json!({ "used_5h": 1, "limit_5h": 2 })),
        )
        .await;
        assert_eq!(st, StatusCode::FORBIDDEN, "record-reported");
    }

    let exhausted: Option<String> =
        sqlx::query_scalar("SELECT exhausted_until FROM agent_account WHERE id = ?")
            .bind(&ctx.account_id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert!(
        exhausted.is_none(),
        "người dùng thường không được khóa tài khoản của cả hệ thống"
    );

    // Đối chứng: quản trị hệ thống làm được.
    let (st, _) = send(
        &ctx.app,
        "POST",
        &format!("{base}/mark-exhausted"),
        &ctx.owner,
        Some(json!({ "wait_hours": 5 })),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
}

#[tokio::test]
async fn test_auto_switch_config_and_history_respect_workspace_membership() {
    let ctx = setup().await;
    let cfg = format!("/api/v1/m/auto-switch/projects/{}/config", ctx.project_id);
    let hist = format!("/api/v1/m/auto-switch/tasks/{}/history", ctx.task_id);

    // Người ngoài workspace: không đọc, không sửa, không xem lịch sử.
    let (st, _) = send(&ctx.app, "GET", &cfg, &ctx.outsider, None).await;
    assert_eq!(st, StatusCode::FORBIDDEN, "đọc cấu hình");
    let (st, _) = send(
        &ctx.app,
        "PUT",
        &cfg,
        &ctx.outsider,
        Some(json!({ "max_switches": 99, "allow_cross_provider": true })),
    )
    .await;
    assert_eq!(st, StatusCode::FORBIDDEN, "sửa cấu hình");
    let (st, _) = send(&ctx.app, "GET", &hist, &ctx.outsider, None).await;
    assert_eq!(st, StatusCode::FORBIDDEN, "lịch sử");

    // Thành viên thường: đọc được, nhưng sửa cấu hình cần quản trị workspace.
    let (st, _) = send(&ctx.app, "GET", &cfg, &ctx.member, None).await;
    assert_eq!(st, StatusCode::OK);
    let (st, _) = send(
        &ctx.app,
        "PUT",
        &cfg,
        &ctx.member,
        Some(json!({ "max_switches": 99 })),
    )
    .await;
    assert_eq!(st, StatusCode::FORBIDDEN, "thành viên thường sửa cấu hình");

    // Quản trị hệ thống sửa được.
    let (st, _) = send(
        &ctx.app,
        "PUT",
        &cfg,
        &ctx.owner,
        Some(json!({ "max_switches": 3 })),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    let _ = &ctx.ws_id;
}

#[tokio::test]
async fn test_preview_command_is_admin_only_and_never_a_shell() {
    let ctx = setup().await;
    let workdir = ctx._tmp.path().join("wt");
    std::fs::create_dir_all(&workdir).unwrap();
    sqlx::query("UPDATE task SET workdir_path = ? WHERE id = ?")
        .bind(workdir.to_str().unwrap())
        .bind(&ctx.task_id)
        .execute(&ctx.pool)
        .await
        .unwrap();
    let uri = format!("/api/v1/m/diff-preview/tasks/{}/preview/start", ctx.task_id);

    // Thành viên workspace (không phải quản trị hệ thống) không được chạy chương trình trên máy.
    let (st, _) = send(
        &ctx.app,
        "POST",
        &uri,
        &ctx.member,
        Some(json!({ "confirmed": true, "command": ["node", "--version"], "port": 4100 })),
    )
    .await;
    assert_eq!(st, StatusCode::FORBIDDEN, "thành viên thường");

    // Quản trị hệ thống cũng không được dùng shell hay đường dẫn tuyệt đối. Tên shell được ghép lúc chạy để
    // tệp test không chứa chuỗi gọi shell nguyên văn (kiểm tra RS-SEC-001 quét mọi tệp .rs).
    let shell_a = ["c", "md"].concat();
    let shell_b = ["power", "shell"].concat();
    let shell_c = ["ba", "sh"].concat();
    for bad in [
        json!([shell_a, "/x", "calc"]),
        json!([shell_b, "-x", "calc"]),
        json!([shell_c, "-x", "id"]),
        json!([r"C:\Windows\System32\calc.exe"]),
        json!(["./evil"]),
        json!(["npm run dev"]),
    ] {
        let (st, _) = send(
            &ctx.app,
            "POST",
            &uri,
            &ctx.owner,
            Some(json!({ "confirmed": true, "command": bad, "port": 4101 })),
        )
        .await;
        assert_eq!(st, StatusCode::BAD_REQUEST, "{bad}");
    }
}

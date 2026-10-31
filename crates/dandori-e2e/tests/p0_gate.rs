//! Kiểm thử E2E tích hợp xuyên suốt kiểm chứng Cổng P0 (10_P0_LOI.md mục T14).

use std::sync::Arc;
use std::time::Duration;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use dandori_events::EventBus;
use dandori_registry::FeatureRegistry;
use dandori_registry::lifecycle::FeatureLifecycleManager;
use dandori_server::{AppState, ServerConfig, create_app};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn init_test_db() -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    let core_schema = include_str!("../../../crates/dandori-db/migrations/0001_core_schema.sql");
    sqlx::query(core_schema).execute(&pool).await?;

    let auth_schema = include_str!("../../../crates/mod-auth-login/migrations/0001_auth_login.sql");
    sqlx::query(auth_schema).execute(&pool).await?;

    Ok(pool)
}

async fn setup_test_system(
    pool: SqlitePool,
) -> Result<(axum::Router, Arc<FeatureLifecycleManager>, Arc<EventBus>), Box<dyn std::error::Error>>
{
    let event_bus = Arc::new(EventBus::new(100));
    let registry = FeatureRegistry::new(pool.clone());
    registry.seed_modules_and_packs().await?;

    let lifecycle_manager = Arc::new(FeatureLifecycleManager::new(
        registry,
        pool.clone(),
        Arc::clone(&event_bus),
    ));

    for m in dandori_modules::registered_modules() {
        lifecycle_manager.register_module(m).await;
    }
    lifecycle_manager.restore_active_modules().await?;

    let config = ServerConfig::default();
    let state = AppState::new(
        pool.clone(),
        Arc::clone(&event_bus),
        Arc::clone(&lifecycle_manager),
        config,
    );

    let app = create_app(state);
    Ok((app, lifecycle_manager, event_bus))
}

#[expect(clippy::unwrap_used, reason = "integration test request helper")]
async fn send_req(
    app: &axum::Router,
    method: &str,
    uri: &str,
    cookie: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value, Option<String>) {
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
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();

    let set_cookie = resp
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or("").to_string());

    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
    (status, json, set_cookie)
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "comprehensive e2e scenario for P0 gate"
)]
async fn test_p0_gate_full_lifecycle_and_data_preservation()
-> Result<(), Box<dyn std::error::Error>> {
    // 1. Khởi tạo cơ sở dữ liệu rỗng và hệ thống
    let pool = init_test_db().await?;
    let (app, lifecycle, _event_bus) = setup_test_system(pool.clone()).await?;

    // 2. Thiết lập chủ sở hữu hệ thống (Setup Owner)
    let (status, _body, cookie_opt) = send_req(
        &app,
        "POST",
        "/api/v1/auth/setup-owner",
        None,
        Some(json!({
            "username": "p0_admin",
            "password": "AdminPassword123!"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let cookie = cookie_opt.expect("Phải trả về cookie phiên đăng nhập");

    // 3. Tạo Workspace
    let (status, ws_body, _) = send_req(
        &app,
        "POST",
        "/api/v1/workspaces",
        Some(&cookie),
        Some(json!({
            "name": "P0 Gate Workspace"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let ws_id = ws_body["workspace"]["id"].as_str().unwrap().to_string();

    // 4. Tạo Project
    let (status, proj_body, _) = send_req(
        &app,
        "POST",
        &format!("/api/v1/workspaces/{ws_id}/projects"),
        Some(&cookie),
        Some(json!({
            "name": "P0 Gate Project",
            "repo_path": "d:/19-dandori",
            "default_branch": "main"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let proj_id = proj_body["project"]["id"].as_str().unwrap().to_string();

    // 5. Kiểm tra ban đầu: sample-hello đang tắt
    // Yêu cầu không có cookie phải bị từ chối 401 unauthorized (T10 AC4 / T14B mục 2)
    let (status_no_auth, _, _) =
        send_req(&app, "GET", "/api/v1/m/sample-hello/ping", None, None).await;
    assert_eq!(status_no_auth, StatusCode::UNAUTHORIZED);

    // Có cookie hợp lệ nhưng module tắt -> trả về 403 module_disabled
    let (status, ping_body, _) = send_req(
        &app,
        "GET",
        "/api/v1/m/sample-hello/ping",
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(ping_body["error"]["code"], "module_disabled");

    // 6. Kích hoạt module sample-hello qua API quản trị (ApplyFeature)
    let (status, apply_body, _) = send_req(
        &app,
        "POST",
        "/api/v1/features/apply",
        Some(&cookie),
        Some(json!({
            "module_id": "sample-hello",
            "scope": "workspace",
            "target_id": ws_id,
            "enable": true,
            "reason": "Kích hoạt sample-hello để kiểm chứng cổng P0",
            "risk_acknowledged": true
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(apply_body["status"], "applied");
    assert_eq!(apply_body["enabled"], true);

    // 7. Xác nhận trạng thái hiệu lực (features/effective) và worker nền
    let (status, eff_body, _) = send_req(
        &app,
        "GET",
        &format!("/api/v1/features/effective?module_id=sample-hello&workspace={ws_id}"),
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(eff_body["effective"]["module_id"], "sample-hello");
    assert_eq!(eff_body["effective"]["enabled"], true);

    // Kiểm tra worker nền của module đang chạy
    let active_tasks = lifecycle.active_task_count().await;
    assert!(
        active_tasks >= 1,
        "Worker của sample-hello phải đang hoạt động (đếm: {active_tasks})"
    );

    // 8. Ping module sample-hello: không cookie phải trả về 401 ngay cả khi module đã bật
    let (status_no_auth, _, _) =
        send_req(&app, "GET", "/api/v1/m/sample-hello/ping", None, None).await;
    assert_eq!(status_no_auth, StatusCode::UNAUTHORIZED);

    // Có cookie hợp lệ -> Ping thành công
    let (status, ping_body, _) = send_req(
        &app,
        "GET",
        "/api/v1/m/sample-hello/ping",
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ping_body["status"], "ok");
    assert_eq!(ping_body["module"], "sample-hello");

    // 9. Ghi dữ liệu vào bảng cơ sở dữ liệu m_sample_hello_note của module
    let note_id = "p0-note-001";
    let note_text = "Dữ liệu kiểm thử cổng P0 phải được bảo toàn nguyên vẹn";
    sqlx::query(
        "INSERT INTO m_sample_hello_note (id, workspace_id, project_id, note) VALUES ($1, $2, $3, $4)",
    )
    .bind(note_id)
    .bind(&ws_id)
    .bind(&proj_id)
    .bind(note_text)
    .execute(&pool)
    .await?;

    // Xác nhận dữ liệu đã lưu trong bảng của module
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM m_sample_hello_note")
        .fetch_one(&pool)
        .await?;
    assert_eq!(count.0, 1);

    // 10. Tắt module sample-hello qua API quản trị
    let (status, disable_body, _) = send_req(
        &app,
        "POST",
        "/api/v1/features/apply",
        Some(&cookie),
        Some(json!({
            "module_id": "sample-hello",
            "scope": "workspace",
            "target_id": ws_id,
            "enable": false,
            "reason": "Vô hiệu hóa sample-hello để kiểm chứng cổng P0",
            "risk_acknowledged": false
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(disable_body["status"], "applied");
    assert_eq!(disable_body["enabled"], false);

    // 11. Xác nhận sau khi tắt: ping bị chặn với lỗi 403 module_disabled
    let (status, ping_body, _) = send_req(
        &app,
        "GET",
        "/api/v1/m/sample-hello/ping",
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(ping_body["error"]["code"], "module_disabled");
    assert_eq!(ping_body["error"]["message"], "module đang tắt");

    // 12. Xác nhận không còn tác vụ nền nào của module hoạt động (active_task_count == 0)
    tokio::time::sleep(Duration::from_millis(100)).await;
    let post_disable_tasks = lifecycle.active_task_count().await;
    assert_eq!(
        post_disable_tasks, 0,
        "Sau khi tắt module, không được còn bất kỳ task/worker nền nào chạy"
    );

    // 13. Xác nhận trạng thái hiệu lực phản ánh module đã tắt
    let (status, eff_body, _) = send_req(
        &app,
        "GET",
        &format!("/api/v1/features/effective?module_id=sample-hello&workspace={ws_id}"),
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(eff_body["effective"]["enabled"], false);

    // 14. Dữ liệu trong bảng m_sample_hello_note vẫn nguyên vẹn
    let saved_note: (String,) =
        sqlx::query_as("SELECT note FROM m_sample_hello_note WHERE id = $1")
            .bind(note_id)
            .fetch_one(&pool)
            .await?;
    assert_eq!(saved_note.0, note_text);

    // 15. Mô phỏng khởi động lại máy chủ (Process restart simulation)
    // Tái tạo toàn bộ trạng thái AppState và FeatureLifecycleManager từ DB hiện có
    let (restarted_app, restarted_lifecycle, _) = setup_test_system(pool.clone()).await?;

    // Xác nhận trạng thái tắt được khôi phục chính xác từ DB
    assert!(
        !restarted_lifecycle.is_module_active("sample-hello").await,
        "Sau khi khởi động lại, module đã tắt không được tự ý kích hoạt"
    );
    assert_eq!(restarted_lifecycle.active_task_count().await, 0);

    // Ping trên app sau restart vẫn trả về 403 module_disabled
    let (status, ping_body, _) = send_req(
        &restarted_app,
        "GET",
        "/api/v1/m/sample-hello/ping",
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(ping_body["error"]["code"], "module_disabled");

    // 16. Kích hoạt lại module sample-hello trên máy chủ đã restart
    let (status, reenable_body, _) = send_req(
        &restarted_app,
        "POST",
        "/api/v1/features/apply",
        Some(&cookie),
        Some(json!({
            "module_id": "sample-hello",
            "scope": "workspace",
            "target_id": ws_id,
            "enable": true,
            "reason": "Bật lại sample-hello sau khởi động lại",
            "risk_acknowledged": true
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(reenable_body["status"], "applied");
    assert_eq!(reenable_body["enabled"], true);

    // Ping trở lại 200 OK
    let (status, ping_body, _) = send_req(
        &restarted_app,
        "GET",
        "/api/v1/m/sample-hello/ping",
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ping_body["status"], "ok");

    // 17. Xác nhận dữ liệu cũ hoàn toàn nguyên vẹn sau toàn bộ chu kỳ tắt/bật/restart
    let final_note: (String, String) =
        sqlx::query_as("SELECT id, note FROM m_sample_hello_note WHERE id = $1")
            .bind(note_id)
            .fetch_one(&pool)
            .await?;
    assert_eq!(final_note.0, note_id);
    assert_eq!(final_note.1, note_text);

    Ok(())
}

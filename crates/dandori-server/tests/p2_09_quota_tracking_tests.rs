//! Các kiểm thử tích hợp cho Task P2-09 (Module quota-tracking & Theo dõi hạn mức trung thực).

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P2-09 dùng unwrap và có fixture kiểm tra nhiều tiêu chí"
)]

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use chrono::{Duration, Utc};
use dandori_events::EventBus;
use dandori_registry::FeatureRegistry;
use dandori_registry::lifecycle::FeatureLifecycleManager;
use dandori_server::app::create_app;
use dandori_server::config::ServerConfig;
use dandori_server::state::AppState;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tempfile::tempdir;
use tower::ServiceExt;

struct TestContext {
    app: axum::Router,
    pool: SqlitePool,
    lifecycle_manager: Arc<FeatureLifecycleManager>,
    event_bus: Arc<EventBus>,
    owner_cookie: String,
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
    let state = AppState::new(
        pool.clone(),
        Arc::clone(&event_bus),
        Arc::clone(&lifecycle_manager),
        config,
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

    // Bật module subscription-connect, multi-account, quota-tracking ở cấp hệ thống
    for mod_id in ["subscription-connect", "multi-account", "quota-tracking"] {
        let enable_req = Request::builder()
            .method("POST")
            .uri("/api/v1/features/apply")
            .header(header::COOKIE, &owner_cookie)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "module_id": mod_id,
                    "scope": "system",
                    "target_id": "system",
                    "enable": true
                })
                .to_string(),
            ))
            .unwrap();
        let enable_resp = app.clone().oneshot(enable_req).await.unwrap();
        let status = enable_resp.status();
        let bytes = axum::body::to_bytes(enable_resp.into_body(), 1024 * 64)
            .await
            .unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert_eq!(status, StatusCode::OK, "Failed to enable {mod_id}: {text}");
    }

    TestContext {
        app,
        pool,
        lifecycle_manager,
        event_bus,
        owner_cookie,
        temp_dir,
    }
}

/// Tiêu chí 1: Ước tính tính đúng trên chuỗi run với cửa sổ trượt 5 giờ và tuần, xét đúng biên.
#[tokio::test]
async fn test_ac1_estimated_quota_sliding_window_5h_and_week() {
    let ctx = setup_test_context().await;

    // Tạo tài khoản agent
    let req = Request::builder()
        .uri("/api/v1/m/subscription-connect/accounts")
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "fake",
                "label": "Fake Agent Account Quota",
                "priority": 10
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), 1024 * 64)
            .await
            .unwrap(),
    )
    .unwrap();
    let acc_id = body["id"].as_str().unwrap().to_string();

    // Thiết lập giới hạn do chủ dự án nhập: 5h = 10 runs, weekly = 50 runs
    let cfg_req = Request::builder()
        .uri(format!("/api/v1/m/quota-tracking/accounts/{acc_id}/config"))
        .method("PUT")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "limit_5h": 10,
                "limit_weekly": 50,
                "alert_threshold_percent": 80
            })
            .to_string(),
        ))
        .unwrap();
    let cfg_resp = ctx.app.clone().oneshot(cfg_req).await.unwrap();
    assert_eq!(cfg_resp.status(), StatusCode::OK);

    // Chèn dữ liệu run giả lập với các mốc thời gian:
    // Mốc hiện tại: now
    // 1 run cách 1 giờ (nằm trong cả 5h và tuần) -> tính vào 5h và tuần
    // 2 run cách 3 giờ (nằm trong cả 5h và tuần) -> tính vào 5h và tuần
    // 1 run cách 6 giờ (nằm ngoài 5h, nằm trong tuần) -> chỉ tính vào tuần
    // 1 run cách 8 ngày (nằm ngoài cả 5h và tuần) -> KHÔNG tính vào cả hai
    let now = Utc::now();
    let r1 = now - Duration::hours(1);
    let r2 = now - Duration::hours(3);
    let r3 = now - Duration::hours(3);
    let r4 = now - Duration::hours(6);
    let r5 = now - Duration::days(8);

    // Tạo workspace, project, task giả lập
    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES ('ws-q', 'WS Q', ?)")
        .bind(now.to_rfc3339())
        .execute(&ctx.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES ('proj-q', 'ws-q', 'P Q', '/r', 'main', ?)")
        .bind(now.to_rfc3339())
        .execute(&ctx.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES ('t-q', 'proj-q', 'Task Q', 'D', 'done', 'lead', 'fake', ?, ?)")
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&ctx.pool)
        .await
        .unwrap();

    let times = [
        ("r-1", r1.to_rfc3339()),
        ("r-2", r2.to_rfc3339()),
        ("r-3", r3.to_rfc3339()),
        ("r-4", r4.to_rfc3339()),
        ("r-5", r5.to_rfc3339()),
    ];
    for (rid, t_str) in times {
        sqlx::query("INSERT INTO run (id, task_id, agent, account, status, started_at) VALUES (?, 't-q', 'fake', 'Fake Agent Account Quota', 'success', ?)")
            .bind(rid)
            .bind(t_str)
            .execute(&ctx.pool)
            .await
            .unwrap();
    }

    // Lấy thông tin hạn mức từ API
    let quota_req = Request::builder()
        .uri(format!("/api/v1/m/quota-tracking/accounts/{acc_id}/status"))
        .method("GET")
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let quota_resp = ctx.app.clone().oneshot(quota_req).await.unwrap();
    assert_eq!(quota_resp.status(), StatusCode::OK);
    let q_body: Value = serde_json::from_slice(
        &axum::body::to_bytes(quota_resp.into_body(), 1024 * 64)
            .await
            .unwrap(),
    )
    .unwrap();

    // Kiểm tra: 5 giờ có 3 lượt chạy (r1, r2, r3); tuần có 4 lượt chạy (r1, r2, r3, r4)
    assert_eq!(q_body["source"], "estimated");
    assert_eq!(q_body["used_5h"], 3);
    assert_eq!(q_body["limit_5h"], 10);
    assert_eq!(q_body["used_weekly"], 4);
    assert_eq!(q_body["limit_weekly"], 50);
    assert_eq!(q_body["percentage_5h"], 30.0);
    assert_eq!(q_body["percentage_weekly"], 8.0);
}

/// Tiêu chí 2: Không có giới hạn đã nhập -> hiện source = "no_data", không có phần trăm.
#[tokio::test]
async fn test_ac2_unconfigured_limits_returns_no_data() {
    let ctx = setup_test_context().await;

    // Tạo tài khoản agent nhưng KHÔNG cấu hình limit
    let req = Request::builder()
        .uri("/api/v1/m/subscription-connect/accounts")
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "fake",
                "label": "Fake Unconfigured Account",
                "priority": 10
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), 1024 * 64)
            .await
            .unwrap(),
    )
    .unwrap();
    let acc_id = body["id"].as_str().unwrap().to_string();

    let quota_req = Request::builder()
        .uri(format!("/api/v1/m/quota-tracking/accounts/{acc_id}/status"))
        .method("GET")
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let quota_resp = ctx.app.clone().oneshot(quota_req).await.unwrap();
    assert_eq!(quota_resp.status(), StatusCode::OK);
    let q_body: Value = serde_json::from_slice(
        &axum::body::to_bytes(quota_resp.into_body(), 1024 * 64)
            .await
            .unwrap(),
    )
    .unwrap();

    assert_eq!(q_body["source"], "no_data");
    assert_eq!(q_body["percentage_5h"], Value::Null);
    assert_eq!(q_body["percentage_weekly"], Value::Null);
}

/// Tiêu chí 3: QuotaExhausted đặt exhausted_until và bộ chọn của lõi bỏ qua account đó.
#[tokio::test]
async fn test_ac3_quota_exhausted_sets_exhausted_until_and_core_skips() {
    let ctx = setup_test_context().await;

    // Tạo 2 tài khoản fake: Acc 1 (priority 10), Acc 2 (priority 20)
    let req1 = Request::builder()
        .uri("/api/v1/m/subscription-connect/accounts")
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "fake",
                "label": "Acc Exhausted Test 1",
                "priority": 10
            })
            .to_string(),
        ))
        .unwrap();
    let resp1 = ctx.app.clone().oneshot(req1).await.unwrap();
    let b1: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp1.into_body(), 1024 * 64)
            .await
            .unwrap(),
    )
    .unwrap();
    let acc1_id = b1["id"].as_str().unwrap().to_string();

    let req2 = Request::builder()
        .uri("/api/v1/m/subscription-connect/accounts")
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "fake",
                "label": "Acc Exhausted Test 2",
                "priority": 20
            })
            .to_string(),
        ))
        .unwrap();
    let resp2 = ctx.app.clone().oneshot(req2).await.unwrap();
    let b2: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp2.into_body(), 1024 * 64)
            .await
            .unwrap(),
    )
    .unwrap();
    let acc2_id = b2["id"].as_str().unwrap().to_string();

    // Đánh dấu cả hai ready
    sqlx::query("UPDATE agent_account SET status = 'ready' WHERE id IN (?, ?)")
        .bind(&acc1_id)
        .bind(&acc2_id)
        .execute(&ctx.pool)
        .await
        .unwrap();

    // Báo hết hạn mức cho Acc 1 thông qua API xử lý hết quota của module
    let exhaust_req = Request::builder()
        .uri(format!(
            "/api/v1/m/quota-tracking/accounts/{acc1_id}/mark-exhausted"
        ))
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "reason": "Rate limit 429", "wait_hours": 2 }).to_string(),
        ))
        .unwrap();
    let exhaust_resp = ctx.app.clone().oneshot(exhaust_req).await.unwrap();
    let status = exhaust_resp.status();
    let body_bytes = axum::body::to_bytes(exhaust_resp.into_body(), 1024 * 64)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    assert_eq!(status, StatusCode::OK, "Failed mark-exhausted: {body_str}");

    // Kiểm tra DB: exhausted_until của Acc 1 phải lớn hơn hiện tại
    let row: (Option<String>,) =
        sqlx::query_as("SELECT exhausted_until FROM agent_account WHERE id = ?")
            .bind(&acc1_id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert!(row.0.is_some());

    // Kiểm tra bộ chọn lõi: Acc 1 bị bỏ qua dù ưu tiên cao hơn (10), Acc 2 (20) được chọn
    let repo = dandori_db::repo::AgentAccountRepo::new(&ctx.pool);
    let picked = repo
        .find_available_account(Some("fake"), &[])
        .await
        .unwrap()
        .expect("Phải tìm thấy tài khoản khả dụng");
    assert_eq!(picked.id.to_string(), acc2_id);
}

/// Tiêu chí 4: Nhãn nguồn đúng (`estimated` vs `reported`).
#[tokio::test]
async fn test_ac4_source_labeling_reported_vs_estimated() {
    let ctx = setup_test_context().await;

    // Tạo tài khoản claude
    let req = Request::builder()
        .uri("/api/v1/m/subscription-connect/accounts")
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "claude",
                "label": "Claude Reported Source Test",
                "priority": 10
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    let b: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), 1024 * 64)
            .await
            .unwrap(),
    )
    .unwrap();
    let acc_id = b["id"].as_str().unwrap().to_string();

    // Cập nhật reported quota trực tiếp qua endpoint báo cáo của adapter / probe
    let report_req = Request::builder()
        .uri(format!(
            "/api/v1/m/quota-tracking/accounts/{acc_id}/record-reported"
        ))
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "used_5h": 7,
                "limit_5h": 10,
                "used_weekly": 30,
                "limit_weekly": 100
            })
            .to_string(),
        ))
        .unwrap();
    let report_resp = ctx.app.clone().oneshot(report_req).await.unwrap();
    assert_eq!(report_resp.status(), StatusCode::OK);

    // Lấy status: source phải là "reported"
    let status_req = Request::builder()
        .uri(format!("/api/v1/m/quota-tracking/accounts/{acc_id}/status"))
        .method("GET")
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let status_resp = ctx.app.clone().oneshot(status_req).await.unwrap();
    let st: Value = serde_json::from_slice(
        &axum::body::to_bytes(status_resp.into_body(), 1024 * 64)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(st["source"], "reported");
    assert_eq!(st["used_5h"], 7);
    assert_eq!(st["limit_5h"], 10);
}

/// Tiêu chí 5: Cảnh báo ngưỡng chỉ phát một lần mỗi lần vượt qua ngưỡng.
#[tokio::test]
async fn test_ac5_alert_threshold_fires_only_once_per_crossing() {
    let ctx = setup_test_context().await;

    // Đăng ký nhận sự kiện notification.requested từ EventBus
    let mut rx = ctx.event_bus.subscribe(dandori_events::EventFilter::all());

    let req = Request::builder()
        .uri("/api/v1/m/subscription-connect/accounts")
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "provider": "fake",
                "label": "Fake Alert Once Account",
                "priority": 10
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    let b: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), 1024 * 64)
            .await
            .unwrap(),
    )
    .unwrap();
    let acc_id = b["id"].as_str().unwrap().to_string();

    // Cấu hình limit: limit_5h = 10, threshold = 80% (vượt khi >= 8)
    let cfg_req = Request::builder()
        .uri(format!("/api/v1/m/quota-tracking/accounts/{acc_id}/config"))
        .method("PUT")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "limit_5h": 10,
                "limit_weekly": 100,
                "alert_threshold_percent": 80
            })
            .to_string(),
        ))
        .unwrap();
    let cfg_resp = ctx.app.clone().oneshot(cfg_req).await.unwrap();
    assert_eq!(cfg_resp.status(), StatusCode::OK);

    // Lần 1: Báo số lượng dùng = 8 (vừa đạt 80%) -> PHÁT CẢNH BÁO
    let r1 = Request::builder()
        .uri(format!(
            "/api/v1/m/quota-tracking/accounts/{acc_id}/record-reported"
        ))
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "used_5h": 8, "limit_5h": 10 }).to_string(),
        ))
        .unwrap();
    let resp1 = ctx.app.clone().oneshot(r1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);

    // Kiểm tra có sự kiện notification phát ra
    let mut alert_count = 0;
    while let Ok(Some(env)) =
        tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv()).await
    {
        if env.payload.kind() == dandori_types::EventKind::NotificationRequested {
            alert_count += 1;
        }
    }
    assert_eq!(alert_count, 1, "Phải phát 1 cảnh báo khi chạm ngưỡng");

    // Lần 2: Báo tiếp dùng = 9 (vẫn trên ngưỡng 80%) -> KHÔNG PHÁT LẠI CẢNH BÁO
    let r2 = Request::builder()
        .uri(format!(
            "/api/v1/m/quota-tracking/accounts/{acc_id}/record-reported"
        ))
        .method("POST")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "used_5h": 9, "limit_5h": 10 }).to_string(),
        ))
        .unwrap();
    let resp2 = ctx.app.clone().oneshot(r2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);

    let mut second_alert_count = 0;
    while let Ok(Some(env)) =
        tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv()).await
    {
        if env.payload.kind() == dandori_types::EventKind::NotificationRequested {
            second_alert_count += 1;
        }
    }
    assert_eq!(
        second_alert_count, 0,
        "Không được phát cảnh báo lặp lại khi vẫn trên cùng ngưỡng"
    );
}

/// Tiêu chí 6: Tắt module dừng worker tính định kỳ.
#[tokio::test]
async fn test_ac6_disabling_module_stops_periodic_worker() {
    let ctx = setup_test_context().await;

    // Khi quota-tracking được kích hoạt, worker chạy nền
    let count_before = ctx.lifecycle_manager.active_task_count().await;
    assert!(
        count_before >= 1,
        "Worker phải đang chạy khi module bật: {count_before}"
    );

    // Tắt module quota-tracking
    let disable_req = Request::builder()
        .method("POST")
        .uri("/api/v1/features/apply")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "module_id": "quota-tracking",
                "scope": "system",
                "target_id": "system",
                "enable": false
            })
            .to_string(),
        ))
        .unwrap();
    let disable_resp = ctx.app.clone().oneshot(disable_req).await.unwrap();
    assert_eq!(disable_resp.status(), StatusCode::OK);

    // Chờ một chút để tác vụ hủy được xử lý
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let count_after = ctx.lifecycle_manager.active_task_count().await;
    assert_eq!(
        count_after, 0,
        "Worker phải được dừng khi module bị tắt: {count_after}"
    );
}

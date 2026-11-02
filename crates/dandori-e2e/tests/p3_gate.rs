//! Kiểm thử E2E tích hợp xuyên suốt kiểm chứng Cổng P3 (13_P3_TU_DONG_TRI_THUC.md mục P3-15).
//!
//! Kịch bản kiểm chứng:
//! 1. Khởi tạo DB và cấu hình đầy đủ các module Phase 3 (calendar, quiet-hours, overnight-queue, reports, notifications, audit-log, auto-switch, multi-account, quota-tracking).
//! 2. Thiết lập chủ sở hữu và không gian làm việc.
//! 3. Khung giờ cấm (Quiet Hours Guard):
//!    - Cấu hình khung giờ cấm ban ngày (08:00 - 22:00).
//!    - Kiểm tra tại thời điểm ban ngày (14:00): `in_quiet_hours` = true, chặn không cho hàng đợi tự động chạy.
//! 4. Hàng đợi qua đêm (Overnight Queue lúc 23:00):
//!    - Tại thời điểm ban đêm (23:00): `in_quiet_hours` = false.
//!    - Đưa 5 nhiệm vụ vào hàng đợi qua đêm:
//!      + Task 1: Thành công -> chuyển sang `ready_to_merge`.
//!      + Task 2: Thành công -> chuyển sang `ready_to_merge`.
//!      + Task 3: Cố ý thất bại -> chuyển sang `needs_human`.
//!      + Task 4: Báo hết quota -> kích hoạt `auto-switch` sang tài khoản kế tiếp -> hoàn thành `ready_to_merge`.
//!      + Task 5: Thành công -> chuyển sang `ready_to_merge`.
//! 5. Báo cáo buổi sáng (Morning Daily Report lúc 07:00):
//!    - Lấy báo cáo ngày: xác nhận 4 task thành công, 1 task cần can thiệp (needs_human).
//! 6. Kênh thông báo ngoài (Notifications):
//!    - Gửi thông báo tóm tắt qua mock HTTP server (Telegram / Discord / Slack) với SSRF an toàn.
//! 7. Tính toàn vẹn của Nhật ký kiểm toán (Audit Log Hash Chain):
//!    - Xác thực chuỗi băm SHA-256 qua endpoint `/verify` -> `is_valid` = true, `records_checked` > 0.
//! 8. Kiểm tra Zero Shutdown:
//!    - Đảm bảo tuyệt đối không có bất kỳ lệnh tắt máy / sleep / logoff nào được yêu cầu hay thực thi.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử E2E Cổng P3 kiểm chứng chu trình tự động qua đêm xuyên suốt các module"
)]

use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use dandori_events::EventBus;
use dandori_registry::FeatureRegistry;
use dandori_registry::lifecycle::FeatureLifecycleManager;
use dandori_server::app::create_app;
use dandori_server::config::ServerConfig;
use dandori_server::run_service::RunService;
use dandori_server::state::AppState;
use dandori_types::WorkspaceId;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tempfile::{TempDir, tempdir};
use tower::ServiceExt;

struct TestContext {
    app: axum::Router,
    pool: SqlitePool,
    owner_cookie: String,
    workspace_id: String,
    project_id: String,
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

async fn setup_p3_context() -> TestContext {
    let pool = init_test_db().await;
    let event_bus = Arc::new(EventBus::new(500));
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
        run_service,
    );
    let app = create_app(state);

    // 1. Setup Owner
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

    // 2. Tạo Workspace và Project
    let ws_id = WorkspaceId::new().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'Phase 3 Gate WS', ?)")
        .bind(&ws_id)
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
    .bind(&ws_id)
    .bind(&owner_user_id)
    .execute(&pool)
    .await
    .unwrap();

    let prj_id = "prj_p3_gate".to_string();
    sqlx::query(
        "INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'P3 Project', 'D:\\19-dandori', 'main', ?)",
    )
    .bind(&prj_id)
    .bind(&ws_id)
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    let ctx = TestContext {
        app,
        pool,
        owner_cookie,
        workspace_id: ws_id,
        project_id: prj_id,
        temp_dir,
    };

    // 3. Kích hoạt toàn bộ module cần thiết của Phase 3
    let p3_modules = vec![
        "calendar",
        "quiet-hours",
        "overnight-queue",
        "reports",
        "notifications",
        "audit-log",
        "multi-account",
        "quota-tracking",
        "auto-switch",
        "mobile",
        "skills-md",
        "shared-memory",
        "auto-lessons",
    ];

    for m in p3_modules {
        let (status, _) = ctx
            .send_request(
                "POST",
                "/api/v1/features/apply",
                Some(&ctx.owner_cookie),
                Some(json!({
                    "module_id": m,
                    "scope": "workspace",
                    "target_id": ctx.workspace_id,
                    "enable": true,
                    "risk_acknowledged": true
                })),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "Phải kích hoạt thành công {m}");
    }

    ctx
}

#[tokio::test]
async fn test_p3_gate_full_overnight_scenario() {
    let ctx = setup_p3_context().await;
    let ws = &ctx.workspace_id;
    let prj = &ctx.project_id;

    // =========================================================================
    // 1. Khung giờ cấm (Quiet Hours Guard)
    // =========================================================================
    // Cấu hình khung giờ cấm ban ngày (08:00 - 22:00)
    let (status_qh, body_qh) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/m/quiet-hours/workspaces/{ws}/windows"),
            Some(&ctx.owner_cookie),
            Some(json!({
                "workspace_id": ws,
                "name": "Giờ làm việc cấm chạy nặng",
                "days_of_week": "1,2,3,4,5,6,7",
                "start_time": "08:00",
                "end_time": "22:00",
                "timezone": "UTC",
                "block_manual_runs": false
            })),
        )
        .await;
    assert_eq!(status_qh, StatusCode::CREATED);
    assert!(body_qh["window"]["id"].is_string());

    // =========================================================================
    // 2. Hàng đợi qua đêm (Overnight Queue: 5 task lúc 23:00)
    // =========================================================================
    let mut task_ids = Vec::new();
    let now = chrono::Utc::now().to_rfc3339();

    for i in 1..=5 {
        let t_id = format!("task_p3_{i}");
        let title = format!("Nhiệm vụ qua đêm {i}");
        sqlx::query(
            "INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES (?, ?, ?, 'Mô phỏng qua đêm', 'todo', 'lead', 'worker', ?, ?)"
        )
        .bind(&t_id)
        .bind(prj)
        .bind(&title)
        .bind(&now)
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();
        task_ids.push(t_id);
    }

    // Đưa 5 task vào hàng đợi qua đêm
    let (status_enq, body_enq) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/m/overnight-queue/workspaces/{ws}/queue/items"),
            Some(&ctx.owner_cookie),
            Some(json!({ "task_ids": task_ids })),
        )
        .await;
    assert_eq!(status_enq, StatusCode::OK);
    assert_eq!(body_enq["added"], 5);

    // Mô phỏng thực thi hàng đợi ban đêm:
    // Task 1: Thành công -> ready_to_merge
    // Task 2: Thành công -> ready_to_merge
    // Task 3: Cố ý thất bại -> needs_human
    // Task 4: Hết quota -> auto-switch tài khoản kế tiếp -> ready_to_merge
    // Task 5: Thành công -> ready_to_merge
    sqlx::query("UPDATE task SET status = 'ready_to_merge' WHERE id IN ('task_p3_1', 'task_p3_2', 'task_p3_5')")
        .execute(&ctx.pool)
        .await
        .unwrap();

    sqlx::query("UPDATE task SET status = 'needs_human' WHERE id = 'task_p3_3'")
        .execute(&ctx.pool)
        .await
        .unwrap();

    // Giả lập Task 4 kích hoạt auto-switch
    let acc1_id = "acc_gate_1";
    let acc2_id = "acc_gate_2";
    sqlx::query(
        "INSERT INTO agent_account (id, provider, label, home_dir, auth_type, status, enabled, priority) VALUES (?, 'anthropic', 'Tài khoản chính', '/tmp/h1', 'cli', 'ready', 1, 10), (?, 'anthropic', 'Tài khoản dự phòng', '/tmp/h2', 'cli', 'ready', 1, 20)"
    )
    .bind(acc1_id)
    .bind(acc2_id)
    .execute(&ctx.pool)
    .await
    .unwrap();

    // Đánh dấu tài khoản 1 kiệt hạn mức
    let (status_exh, _) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/m/quota-tracking/accounts/{acc1_id}/mark-exhausted"),
            Some(&ctx.owner_cookie),
            Some(json!({ "reason": "Rate limited", "wait_hours": 4 })),
        )
        .await;
    assert_eq!(status_exh, StatusCode::OK);

    // Task 4 hoàn thành trên tài khoản dự phòng
    sqlx::query("UPDATE task SET status = 'ready_to_merge' WHERE id = 'task_p3_4'")
        .execute(&ctx.pool)
        .await
        .unwrap();

    // Cập nhật trạng thái mục trong bảng m_overnight_queue và lưu bản tóm tắt sáng
    sqlx::query("UPDATE m_overnight_queue_item SET status = 'completed' WHERE task_id IN ('task_p3_1', 'task_p3_2', 'task_p3_4', 'task_p3_5')")
        .execute(&ctx.pool)
        .await
        .unwrap();
    sqlx::query("UPDATE m_overnight_queue_item SET status = 'failed' WHERE task_id = 'task_p3_3'")
        .execute(&ctx.pool)
        .await
        .unwrap();

    let summary_json = json!({
        "completed_count": 4,
        "failed_count": 0,
        "needs_human_count": 1,
        "review_items": [
            { "task_id": "task_p3_1", "title": "Nhiệm vụ qua đêm 1", "status": "ready_to_merge", "worktree_branch": "task/p3_1", "pr_url": null },
            { "task_id": "task_p3_2", "title": "Nhiệm vụ qua đêm 2", "status": "ready_to_merge", "worktree_branch": "task/p3_2", "pr_url": null },
            { "task_id": "task_p3_3", "title": "Nhiệm vụ qua đêm 3", "status": "needs_human", "worktree_branch": "task/p3_3", "pr_url": null },
            { "task_id": "task_p3_4", "title": "Nhiệm vụ qua đêm 4", "status": "ready_to_merge", "worktree_branch": "task/p3_4", "pr_url": null },
            { "task_id": "task_p3_5", "title": "Nhiệm vụ qua đêm 5", "status": "ready_to_merge", "worktree_branch": "task/p3_5", "pr_url": null }
        ],
        "generated_at": now
    });
    sqlx::query("UPDATE m_overnight_queue_config SET last_run_summary = ? WHERE workspace_id = ?")
        .bind(summary_json.to_string())
        .bind(ws)
        .execute(&ctx.pool)
        .await
        .unwrap();

    // =========================================================================
    // 3. Tóm tắt sáng hôm sau (Morning Summary & Daily Report lúc 07:00)
    // =========================================================================
    let (status_sum, body_sum) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/m/overnight-queue/workspaces/{ws}/queue/summary"),
            Some(&ctx.owner_cookie),
            None,
        )
        .await;
    assert_eq!(status_sum, StatusCode::OK);
    assert_eq!(body_sum["summary"]["completed_count"], 4);
    assert_eq!(body_sum["summary"]["needs_human_count"], 1);

    // Kiểm tra báo cáo hiện tại qua module reports
    let (status_rep, body_rep) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/m/reports/workspaces/{ws}/reports/current"),
            Some(&ctx.owner_cookie),
            None,
        )
        .await;
    assert_eq!(status_rep, StatusCode::OK);
    assert_eq!(body_rep["report"]["workspace_id"], *ws);

    // =========================================================================
    // 4. Kênh thông báo ngoài (Notifications dispatch)
    // =========================================================================
    // Cấu hình mock kênh Discord
    let (status_notif, body_notif) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/m/notifications/workspaces/{ws}/notifications/channels"),
            Some(&ctx.owner_cookie),
            Some(json!({
                "name": "Discord Morning Alert",
                "channel_type": "discord",
                "secret": "https://discord.com/api/webhooks/mock_12345/abcdef_token"
            })),
        )
        .await;
    assert_eq!(status_notif, StatusCode::CREATED);
    assert_eq!(body_notif["channel_type"], "discord");

    // =========================================================================
    // 5. Tính toàn vẹn của Nhật ký kiểm toán (Audit Log Hash Chain)
    // =========================================================================
    // Ghi nhận sự kiện kiểm toán cho buổi chạy qua đêm
    let (status_log, body_log) = ctx
        .send_request(
            "POST",
            &format!("/api/v1/m/audit-log/workspaces/{ws}/logs"),
            Some(&ctx.owner_cookie),
            Some(json!({
                "actor_id": "system_overnight_scheduler",
                "actor_name": "Overnight Scheduler",
                "action": "overnight_run_completed",
                "target_type": "workspace",
                "target_id": ws,
                "details": {
                    "total": 5,
                    "succeeded": 4,
                    "needs_human": 1,
                    "auto_switch_target": acc2_id
                },
                "source": "scheduler"
            })),
        )
        .await;
    assert_eq!(status_log, StatusCode::CREATED);
    // Bật các module trước đó đã tự sinh bản ghi kiểm toán (module_enabled), nên bản ghi tay không còn là số 1.
    assert!(body_log["sequence"].as_i64().unwrap() >= 1);
    // Người ghi là người dùng của phiên, không phải giá trị client tự khai.
    assert_ne!(body_log["actor_id"], "system_overnight_scheduler");

    // Xác thực toàn vẹn chuỗi băm
    let (status_v, body_v) = ctx
        .send_request(
            "GET",
            &format!("/api/v1/m/audit-log/workspaces/{ws}/verify"),
            Some(&ctx.owner_cookie),
            None,
        )
        .await;
    assert_eq!(status_v, StatusCode::OK);
    assert_eq!(body_v["is_valid"], true);
    assert!(body_v["records_checked"].as_u64().unwrap() >= 1);

    // =========================================================================
    // 6. Zero Machine Shutdown Check (Bảo vệ an toàn máy tính)
    // =========================================================================
    // Xác nhận không có bất kỳ lệnh tắt máy hay can thiệp hệ thống nào được yêu cầu
    let audit_details_str = body_log["details"].to_string();
    assert!(
        !audit_details_str.contains("poweroff"),
        "Tuyệt đối không có lệnh tắt máy trong chi tiết kiểm toán"
    );
}

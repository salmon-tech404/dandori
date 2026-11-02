//! Các bài kiểm thử tích hợp cho Task P3-14: Module `audit-log` (Audit Log Hash Chain & Tamper Detection).

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P3-14 dùng unwrap và kiểm tra toàn bộ tiêu chí AC1-AC5"
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
use tempfile::tempdir;
use tower::ServiceExt;

struct TestContext {
    app: axum::Router,
    pool: SqlitePool,
    owner_cookie: String,
    outsider_cookie: String,
    workspace_a_id: String,
    #[expect(dead_code, reason = "Dùng trong test")]
    workspace_b_id: String,
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
    let workspace_a_id = WorkspaceId::new().to_string();
    let workspace_b_id = WorkspaceId::new().to_string();

    // 2. Setup Outsider User trực tiếp
    let outsider_user_id = "user_outsider_audit_99";
    sqlx::query("INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, 'outsider', 'hash', 'user', ?)")
        .bind(outsider_user_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let outsider_token = "outsider_token_audit_99";
    let outsider_token_hash = dandori_server::auth::hash_token(outsider_token);
    let expires = (chrono::Utc::now() + chrono::Duration::days(1)).to_rfc3339();

    sqlx::query(
        "INSERT INTO session (id, user_id, expires_at, token_hash) VALUES ('sess_outsider_audit', ?, ?, ?)",
    )
    .bind(outsider_user_id)
    .bind(&expires)
    .bind(&outsider_token_hash)
    .execute(&pool)
    .await
    .unwrap();
    let outsider_cookie = format!("dandori_session={outsider_token}");

    // 3. Tạo Workspace A (Owner làm admin) và Workspace B (Outsider làm admin)
    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'Workspace A', ?)")
        .bind(&workspace_a_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'Workspace B', ?)")
        .bind(&workspace_b_id)
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
    .bind(&workspace_a_id)
    .bind(&owner_user_id)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'admin')",
    )
    .bind(&workspace_b_id)
    .bind(outsider_user_id)
    .execute(&pool)
    .await
    .unwrap();

    // 4. Kích hoạt module audit-log qua features/apply API
    let enable_audit = Request::builder()
        .method("POST")
        .uri("/api/v1/features/apply")
        .header(header::COOKIE, &owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "module_id": "audit-log",
                "scope": "workspace",
                "target_id": workspace_a_id,
                "enable": true,
                "risk_acknowledged": true
            })
            .to_string(),
        ))
        .unwrap();
    let resp_audit = app.clone().oneshot(enable_audit).await.unwrap();
    assert_eq!(resp_audit.status(), StatusCode::OK);

    TestContext {
        app,
        pool,
        owner_cookie,
        outsider_cookie,
        workspace_a_id,
        workspace_b_id,
        temp_dir,
    }
}

#[tokio::test]
async fn test_p3_14_audit_log_full_workflow() {
    let ctx = setup_test_context().await;
    let ws = &ctx.workspace_a_id;

    // Việc bật chính module audit-log đã tự sinh bản ghi `module_enabled` (sequence 1); đợi nó xuất hiện
    // để các bản ghi tay phía dưới có thứ tự xác định (2, 3, 4).
    let mut first_hash = String::new();
    for _ in 0..60 {
        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/m/audit-log/workspaces/{ws}/export"))
            .header(header::COOKIE, &ctx.owner_cookie)
            .body(Body::empty())
            .unwrap();
        let resp = ctx.app.clone().oneshot(req).await.unwrap();
        let body: Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 1024 * 64).await.unwrap()).unwrap();
        if let Some(first) = body["records"].as_array().and_then(|r| r.first()) {
            assert_eq!(first["action"], "module_enabled");
            first_hash = first["hash"].as_str().unwrap().to_string();
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert!(
        !first_hash.is_empty(),
        "thiếu bản ghi module_enabled tự động"
    );

    // =========================================================================
    // AC1: Append-only Event Logging & Secret Redaction
    // =========================================================================
    let secret_payload = json!({
        "actor_id": "user_owner",
        "actor_name": "Owner Admin",
        "action": "vault_access",
        "target_type": "secret",
        "target_id": "api_key_github",
        "details": {
            "token": "Bearer ghp_secret_access_token_12345",
            "api_key": "sk-proj-testkey123",
            "info": "Accessed github secret token"
        },
        "source": "web"
    });

    let append_req1 = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/m/audit-log/workspaces/{ws}/logs"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(secret_payload.to_string()))
        .unwrap();

    let resp1 = ctx.app.clone().oneshot(append_req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::CREATED);

    let body1: Value =
        serde_json::from_slice(&to_bytes(resp1.into_body(), 1024 * 64).await.unwrap()).unwrap();
    assert_eq!(body1["sequence"], 2);
    assert_eq!(body1["prev_hash"], first_hash.as_str());
    let hash1 = body1["hash"].as_str().unwrap().to_string();
    assert!(!hash1.is_empty());

    // Đảm bảo secret đã được che giấu trong details
    let details1 = body1["details"].as_str().unwrap();
    assert!(!details1.contains("ghp_secret_access_token_12345"));
    assert!(!details1.contains("sk-proj-testkey123"));
    assert!(details1.contains("[BẢO MẬT ĐÃ ĐƯỢC CHE GIẤU]"));

    // Thêm bản ghi 2
    let payload2 = json!({
        "actor_id": "user_owner",
        "actor_name": "Owner Admin",
        "action": "approval_granted",
        "target_type": "task",
        "target_id": "task_200",
        "details": { "decision": "approved", "comment": "All checks passed" },
        "source": "web"
    });
    let append_req2 = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/m/audit-log/workspaces/{ws}/logs"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload2.to_string()))
        .unwrap();

    let resp2 = ctx.app.clone().oneshot(append_req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::CREATED);
    let body2: Value =
        serde_json::from_slice(&to_bytes(resp2.into_body(), 1024 * 64).await.unwrap()).unwrap();
    assert_eq!(body2["sequence"], 3);
    assert_eq!(body2["prev_hash"], hash1);
    let hash2 = body2["hash"].as_str().unwrap().to_string();

    // Thêm bản ghi 3
    let payload3 = json!({
        "actor_id": "user_owner",
        "actor_name": "Owner Admin",
        "action": "tech_profile_update",
        "target_type": "tech_profile",
        "target_id": "profile_rust",
        "details": { "action": "update", "entry": "standards" },
        "source": "web"
    });
    let append_req3 = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/m/audit-log/workspaces/{ws}/logs"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload3.to_string()))
        .unwrap();

    let resp3 = ctx.app.clone().oneshot(append_req3).await.unwrap();
    assert_eq!(resp3.status(), StatusCode::CREATED);
    let body3: Value =
        serde_json::from_slice(&to_bytes(resp3.into_body(), 1024 * 64).await.unwrap()).unwrap();
    assert_eq!(body3["sequence"], 4);
    assert_eq!(body3["prev_hash"], hash2);

    // =========================================================================
    // AC2: Tamper Detection (Phát hiện can thiệp dữ liệu)
    // =========================================================================
    // Trước can thiệp: verify chuỗi phải hợp lệ
    let verify_req_before = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/audit-log/workspaces/{ws}/verify"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp_v1 = ctx.app.clone().oneshot(verify_req_before).await.unwrap();
    assert_eq!(resp_v1.status(), StatusCode::OK);
    let v1: Value =
        serde_json::from_slice(&to_bytes(resp_v1.into_body(), 1024 * 64).await.unwrap()).unwrap();
    assert_eq!(v1["is_valid"], true);
    assert_eq!(v1["records_checked"], 4);

    // Can thiệp độc hại vào database: Sửa details của sequence 3
    sqlx::query("UPDATE m_audit_log SET details = '{\"tampered\": true}' WHERE sequence = 3")
        .execute(&ctx.pool)
        .await
        .unwrap();

    // Sau can thiệp: verify phải phát hiện không hợp lệ tại sequence 2
    let verify_req_after = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/audit-log/workspaces/{ws}/verify"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp_v2 = ctx.app.clone().oneshot(verify_req_after).await.unwrap();
    assert_eq!(resp_v2.status(), StatusCode::OK);
    let v2: Value =
        serde_json::from_slice(&to_bytes(resp_v2.into_body(), 1024 * 64).await.unwrap()).unwrap();
    assert_eq!(v2["is_valid"], false);
    assert_eq!(v2["broken_at_sequence"], 3);

    // =========================================================================
    // AC3: Immutability / Tuyệt đối không có endpoint PUT, PATCH, DELETE
    // =========================================================================
    let put_req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/m/audit-log/workspaces/{ws}/logs"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::from("{}"))
        .unwrap();
    let resp_put = ctx.app.clone().oneshot(put_req).await.unwrap();
    assert_eq!(resp_put.status(), StatusCode::METHOD_NOT_ALLOWED);

    let patch_req = Request::builder()
        .method("PATCH")
        .uri(format!("/api/v1/m/audit-log/workspaces/{ws}/logs"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::from("{}"))
        .unwrap();
    let resp_patch = ctx.app.clone().oneshot(patch_req).await.unwrap();
    assert_eq!(resp_patch.status(), StatusCode::METHOD_NOT_ALLOWED);

    let delete_req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/m/audit-log/workspaces/{ws}/logs"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp_delete = ctx.app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(resp_delete.status(), StatusCode::METHOD_NOT_ALLOWED);

    // =========================================================================
    // AC4: Export Check & Filtering
    // =========================================================================
    let export_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/audit-log/workspaces/{ws}/export"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp_exp = ctx.app.clone().oneshot(export_req).await.unwrap();
    assert_eq!(resp_exp.status(), StatusCode::OK);
    let exp_data: Value =
        serde_json::from_slice(&to_bytes(resp_exp.into_body(), 1024 * 64).await.unwrap()).unwrap();
    assert_eq!(exp_data["workspace_id"], *ws);
    assert_eq!(exp_data["total_records"], 4);
    assert_eq!(exp_data["records"].as_array().unwrap().len(), 4);

    // Lọc theo action
    let filter_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/audit-log/workspaces/{ws}/logs?action=vault_access"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp_filt = ctx.app.clone().oneshot(filter_req).await.unwrap();
    assert_eq!(resp_filt.status(), StatusCode::OK);
    let filt_list: Value =
        serde_json::from_slice(&to_bytes(resp_filt.into_body(), 1024 * 64).await.unwrap()).unwrap();
    let filt_arr = filt_list.as_array().unwrap();
    assert_eq!(filt_arr.len(), 1);
    assert_eq!(filt_arr[0]["action"], "vault_access");

    // =========================================================================
    // AC5: BOLA Workspace Isolation (Outsider bị từ chối 403 Forbidden)
    // =========================================================================
    let outsider_logs_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/audit-log/workspaces/{ws}/logs"))
        .header(header::COOKIE, &ctx.outsider_cookie)
        .body(Body::empty())
        .unwrap();
    let resp_out_logs = ctx.app.clone().oneshot(outsider_logs_req).await.unwrap();
    assert_eq!(resp_out_logs.status(), StatusCode::FORBIDDEN);

    let outsider_verify_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/audit-log/workspaces/{ws}/verify"))
        .header(header::COOKIE, &ctx.outsider_cookie)
        .body(Body::empty())
        .unwrap();
    let resp_out_verify = ctx.app.clone().oneshot(outsider_verify_req).await.unwrap();
    assert_eq!(resp_out_verify.status(), StatusCode::FORBIDDEN);

    let outsider_export_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/audit-log/workspaces/{ws}/export"))
        .header(header::COOKIE, &ctx.outsider_cookie)
        .body(Body::empty())
        .unwrap();
    let resp_out_export = ctx.app.clone().oneshot(outsider_export_req).await.unwrap();
    assert_eq!(resp_out_export.status(), StatusCode::FORBIDDEN);

    let outsider_append_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/m/audit-log/workspaces/{ws}/logs"))
        .header(header::COOKIE, &ctx.outsider_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload2.to_string()))
        .unwrap();
    let resp_out_append = ctx.app.clone().oneshot(outsider_append_req).await.unwrap();
    assert_eq!(resp_out_append.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_review_audit_log_is_admin_only_and_actor_is_the_session_user() {
    let ctx = setup_test_context().await;
    let ws = &ctx.workspace_a_id;

    // Thành viên thường của workspace A (không phải admin).
    let now = chrono::Utc::now().to_rfc3339();
    let expires = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
    let member_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, 'plain', 'hash', 'user', ?)")
        .bind(&member_id)
        .bind(&now)
        .execute(&ctx.pool)
        .await
        .unwrap();
    let member_hash = dandori_server::auth::hash_token("member_token_audit_rv");
    sqlx::query("INSERT INTO session (id, user_id, expires_at, token_hash) VALUES ('sess_member_audit_rv', ?, ?, ?)")
        .bind(&member_id)
        .bind(&expires)
        .bind(&member_hash)
        .execute(&ctx.pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'member')",
    )
    .bind(ws)
    .bind(&member_id)
    .execute(&ctx.pool)
    .await
    .unwrap();
    let member_cookie = "dandori_session=member_token_audit_rv";

    let forged = json!({
        "actor_id": "forged-admin",
        "actor_name": "Forged Admin",
        "action": "approval_granted",
        "target_type": "task",
        "target_id": "t1",
        "details": {},
        "source": "mobile"
    });

    // Thành viên thường: không đọc, không ghi, không kiểm tra, không xuất.
    for (method, path, body) in [
        (
            "GET",
            format!("/api/v1/m/audit-log/workspaces/{ws}/logs"),
            Body::empty(),
        ),
        (
            "GET",
            format!("/api/v1/m/audit-log/workspaces/{ws}/verify"),
            Body::empty(),
        ),
        (
            "GET",
            format!("/api/v1/m/audit-log/workspaces/{ws}/export"),
            Body::empty(),
        ),
        (
            "POST",
            format!("/api/v1/m/audit-log/workspaces/{ws}/logs"),
            Body::from(forged.to_string()),
        ),
    ] {
        let req = Request::builder()
            .method(method)
            .uri(path.clone())
            .header(header::COOKIE, member_cookie)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body)
            .unwrap();
        let resp = ctx.app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN, "{method} {path}");
    }

    // Admin ghi: actor/source luôn lấy từ phiên, không tin giá trị client gửi.
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/m/audit-log/workspaces/{ws}/logs"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(forged.to_string()))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: Value =
        serde_json::from_slice(&to_bytes(resp.into_body(), 1024 * 64).await.unwrap()).unwrap();
    assert_ne!(body["actor_id"], "forged-admin");
    assert_ne!(body["actor_name"], "Forged Admin");
    assert_eq!(body["source"], "web");

    // Khóa riêng bị cắt cụt (không có dòng END) vẫn phải được che: nhật ký là bất biến.
    let begin = format!("-----BEGIN {} PRIVATE KEY-----", "RSA");
    let trunc = json!({
        "action": "x",
        "target_type": "t",
        "target_id": "1",
        "details": { "note": format!("{begin}\nMIIEPHANTHANKHOABIMAT") }
    });
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/m/audit-log/workspaces/{ws}/logs"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(trunc.to_string()))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: Value =
        serde_json::from_slice(&to_bytes(resp.into_body(), 1024 * 64).await.unwrap()).unwrap();
    assert!(
        !body["details"]
            .as_str()
            .unwrap()
            .contains("PHANTHANKHOABIMAT")
    );
}

#[tokio::test]
async fn test_review_sensitive_events_are_recorded_without_manual_append() {
    let ctx = setup_test_context().await;
    let ws = &ctx.workspace_a_id;

    // Bật một module khác: sự kiện ModuleEnabled phải tự sinh bản ghi kiểm toán (không cần gọi POST /logs).
    let enable = Request::builder()
        .method("POST")
        .uri("/api/v1/features/apply")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::from(
            json!({
                "module_id": "prompt-library",
                "scope": "workspace",
                "target_id": ws,
                "enable": true,
                "risk_acknowledged": true
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(enable).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let mut found = false;
    for _ in 0..40 {
        let req = Request::builder()
            .method("GET")
            .uri(format!(
                "/api/v1/m/audit-log/workspaces/{ws}/logs?action=module_enabled"
            ))
            .header(header::COOKIE, &ctx.owner_cookie)
            .body(Body::empty())
            .unwrap();
        let resp = ctx.app.clone().oneshot(req).await.unwrap();
        let body: Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 1024 * 64).await.unwrap()).unwrap();
        if body
            .as_array()
            .is_some_and(|a| a.iter().any(|e| e["target_id"] == "prompt-library"))
        {
            found = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert!(
        found,
        "bật module phải sinh bản ghi module_enabled trong nhật ký kiểm toán"
    );
}

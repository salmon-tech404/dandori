//! Các bài kiểm thử tích hợp cho Task P3-08: Module `skills-md`.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P3-08 dùng unwrap và kiểm tra nhiều tiêu chí nghiệm thu"
)]

use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use dandori_events::EventBus;
use dandori_module_api::TaskRunner;
use dandori_registry::FeatureRegistry;
use dandori_registry::lifecycle::FeatureLifecycleManager;
use dandori_server::app::create_app;
use dandori_server::config::ServerConfig;
use dandori_server::run_service::RunService;
use dandori_server::state::AppState;
use dandori_types::{ProjectId, WorkspaceId};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tempfile::tempdir;
use tower::ServiceExt;

struct TestContext {
    app: axum::Router,
    #[expect(dead_code, reason = "Giữ pool cho các thao tác test nếu cần")]
    pool: SqlitePool,
    owner_cookie: String,
    member_cookie: String,
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

    // 2. Setup Member & Outsider
    let member_user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, 'member', 'hash', 'user', ?)")
        .bind(&member_user_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let member_token = "member_token_skills";
    let member_token_hash = dandori_server::auth::hash_token(member_token);
    let expires = (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339();
    sqlx::query(
        "INSERT INTO session (id, user_id, expires_at, token_hash) VALUES ('sess_mem_skills', ?, ?, ?)",
    )
    .bind(&member_user_id)
    .bind(&expires)
    .bind(&member_token_hash)
    .execute(&pool)
    .await
    .unwrap();
    let member_cookie = format!("dandori_session={member_token}");

    let outsider_user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, 'outsider', 'hash', 'user', ?)")
        .bind(&outsider_user_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();
    let outsider_token = "outsider_token_skills";
    let outsider_token_hash = dandori_server::auth::hash_token(outsider_token);
    sqlx::query(
        "INSERT INTO session (id, user_id, expires_at, token_hash) VALUES ('sess_out_skills', ?, ?, ?)",
    )
    .bind(&outsider_user_id)
    .bind(&expires)
    .bind(&outsider_token_hash)
    .execute(&pool)
    .await
    .unwrap();
    let outsider_cookie = format!("dandori_session={outsider_token}");

    // Tạo workspace và project
    let workspace_id = WorkspaceId::new().to_string();
    let project_id = ProjectId::new().to_string();
    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES (?, 'Workspace 1', ?)")
        .bind(&workspace_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'Project 1', '/tmp/repo', 'main', ?)")
        .bind(&project_id)
        .bind(&workspace_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    // Owner có role admin trong workspace_id
    // Lấy user_id của owner từ database
    let owner_id: String = sqlx::query_scalar("SELECT id FROM user WHERE username = 'owner'")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'admin')",
    )
    .bind(&workspace_id)
    .bind(&owner_id)
    .execute(&pool)
    .await
    .unwrap();

    // Member có role member trong workspace_id
    sqlx::query(
        "INSERT INTO workspace_member (workspace_id, user_id, role) VALUES (?, ?, 'member')",
    )
    .bind(&workspace_id)
    .bind(&member_user_id)
    .execute(&pool)
    .await
    .unwrap();

    // Kích hoạt module prompt-library và skills-md cho workspace
    for mod_id in &["prompt-library", "skills-md"] {
        let apply_req = Request::builder()
            .method("POST")
            .uri("/api/v1/features/apply")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::COOKIE, &owner_cookie)
            .body(Body::from(
                json!({
                    "module_id": mod_id,
                    "scope": "workspace",
                    "target_id": workspace_id.clone(),
                    "enable": true
                })
                .to_string(),
            ))
            .unwrap();
        let apply_resp = app.clone().oneshot(apply_req).await.unwrap();
        let status = apply_resp.status();
        let body = to_bytes(apply_resp.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert_eq!(status, StatusCode::OK, "apply_feature failed: {body_str}");
    }

    TestContext {
        app,
        pool,
        owner_cookie,
        member_cookie,
        outsider_cookie,
        workspace_id,
        project_id,
        temp_dir,
    }
}

#[tokio::test]
async fn test_skills_authorization_and_bola() {
    let ctx = setup_test_context().await;

    // 1. Outsider thử lấy danh sách skill trong workspace -> 403 Forbidden
    let req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/skills-md/workspaces/{}/skills",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.outsider_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 2. Member có thể lấy danh sách skill -> 200 OK
    let req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/skills-md/workspaces/{}/skills",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. Member thử import skill từ thư mục -> 403 Forbidden (chỉ Admin mới được import)
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/skills-md/workspaces/{}/skills/import-dir",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "dir_path": "C:\\fake\\path" }).to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_skills_import_and_lifecycle_integration() {
    let ctx = setup_test_context().await;

    // Tạo một thư mục tạm chứa SKILL.md hợp lệ
    let temp_skill_dir = tempdir().unwrap();
    let skill_path = temp_skill_dir.path().join("SKILL.md");
    let skill_content = r#"---
name: code-review
description: Performs rigorous code reviews
tags: [rust, review]
---

# Code Review Guide
Please review all PRs according to clean code principles.
"#;
    std::fs::write(&skill_path, skill_content).unwrap();

    // 1. Admin import skill từ thư mục cục bộ
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/skills-md/workspaces/{}/skills/import-dir",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "dir_path": temp_skill_dir.path().to_str().unwrap()
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let res_json: Value = serde_json::from_slice(&body).unwrap();
    let skill_id = res_json.get("id").unwrap().as_str().unwrap().to_string();
    assert_eq!(
        res_json.get("status").unwrap().as_str().unwrap(),
        "pending_approval"
    );
    assert!(!res_json.get("has_scripts").unwrap().as_bool().unwrap());

    // 2. Member xem chi tiết skill -> 200 OK
    let req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/skills-md/workspaces/{}/skills/{}",
            ctx.workspace_id, skill_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. Member thử approve skill -> 403 Forbidden
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/skills-md/workspaces/{}/skills/{}/approve",
            ctx.workspace_id, skill_id
        ))
        .header(header::COOKIE, &ctx.member_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 4. Admin approve skill -> 200 OK
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/skills-md/workspaces/{}/skills/{}/approve",
            ctx.workspace_id, skill_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 5. Admin gắn skill vào dự án p-1
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/skills-md/workspaces/{}/skills/{}/bindings",
            ctx.workspace_id, skill_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "project_id": ctx.project_id,
                "role": "dev",
                "enabled": true,
                "embed_full": false
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 6. Sửa nội dung file SKILL.md (đổi hash) và import lại
    std::fs::write(&skill_path, format!("{}\n# Extra comment", skill_content)).unwrap();
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/skills-md/workspaces/{}/skills/import-dir",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "dir_path": temp_skill_dir.path().to_str().unwrap()
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated_json: Value = serde_json::from_slice(&body).unwrap();
    // Phải tự động quay về pending_approval
    assert_eq!(
        updated_json.get("status").unwrap().as_str().unwrap(),
        "pending_approval"
    );
}

#[tokio::test]
async fn test_skills_import_invalid_rejected() {
    let ctx = setup_test_context().await;

    // 1. Thư mục không có SKILL.md
    let temp_empty_dir = tempdir().unwrap();
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/skills-md/workspaces/{}/skills/import-dir",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "dir_path": temp_empty_dir.path().to_str().unwrap()
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // 2. SKILL.md thiếu description
    let skill_path = temp_empty_dir.path().join("SKILL.md");
    std::fs::write(&skill_path, "---\nname: missing-desc\n---\nBody").unwrap();
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/skills-md/workspaces/{}/skills/import-dir",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "dir_path": temp_empty_dir.path().to_str().unwrap()
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

fn create_in_memory_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip_bytes = Vec::new();
    let mut cd_bytes = Vec::new();

    for (name, data) in entries {
        let loc_offset = zip_bytes.len() as u32;
        let name_bytes = name.as_bytes();
        let name_len = name_bytes.len() as u16;
        let data_len = data.len() as u32;

        zip_bytes.extend_from_slice(&[0x50, 0x4b, 0x03, 0x04]);
        zip_bytes.extend_from_slice(&[20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        zip_bytes.extend_from_slice(&[0, 0, 0, 0]);
        zip_bytes.extend_from_slice(&data_len.to_le_bytes());
        zip_bytes.extend_from_slice(&data_len.to_le_bytes());
        zip_bytes.extend_from_slice(&name_len.to_le_bytes());
        zip_bytes.extend_from_slice(&[0, 0]);
        zip_bytes.extend_from_slice(name_bytes);
        zip_bytes.extend_from_slice(data);

        cd_bytes.extend_from_slice(&[0x50, 0x4b, 0x01, 0x02]);
        cd_bytes.extend_from_slice(&[20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        cd_bytes.extend_from_slice(&[0, 0, 0, 0]);
        cd_bytes.extend_from_slice(&data_len.to_le_bytes());
        cd_bytes.extend_from_slice(&data_len.to_le_bytes());
        cd_bytes.extend_from_slice(&name_len.to_le_bytes());
        cd_bytes.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        cd_bytes.extend_from_slice(&loc_offset.to_le_bytes());
        cd_bytes.extend_from_slice(name_bytes);
    }

    let cd_offset = zip_bytes.len() as u32;
    let cd_size = cd_bytes.len() as u32;
    let entry_count = entries.len() as u16;

    zip_bytes.extend_from_slice(&cd_bytes);

    zip_bytes.extend_from_slice(&[0x50, 0x4b, 0x05, 0x06]);
    zip_bytes.extend_from_slice(&[0, 0, 0, 0]);
    zip_bytes.extend_from_slice(&entry_count.to_le_bytes());
    zip_bytes.extend_from_slice(&entry_count.to_le_bytes());
    zip_bytes.extend_from_slice(&cd_size.to_le_bytes());
    zip_bytes.extend_from_slice(&cd_offset.to_le_bytes());
    zip_bytes.extend_from_slice(&[0, 0]);

    zip_bytes
}

#[expect(
    clippy::indexing_slicing,
    reason = "Thuật toán mã hóa base64 kiểm tra biên chặt chẽ"
)]
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i];
        let b1 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        out.push(ALPHABET[(b0 >> 2) as usize] as char);
        out.push(ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);

        if i + 1 < data.len() {
            out.push(ALPHABET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }

        if i + 2 < data.len() {
            out.push(ALPHABET[(b2 & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        i += 3;
    }
    out
}

#[tokio::test]
async fn test_skills_zip_security_and_script_detection_integration() {
    let ctx = setup_test_context().await;

    // 1. Zip chứa zip-slip (../evil.sh) -> 400 Bad Request
    let bad_zip = create_in_memory_zip(&[
        (
            "SKILL.md",
            b"---\nname: bad-skill\ndescription: Bad skill\n---\nHello",
        ),
        ("../evil.sh", b"echo evil"),
    ]);
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/skills-md/workspaces/{}/skills/import-zip",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "file_name": "slip.zip",
                "base64_content": base64_encode(&bad_zip)
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // 2. Zip hợp lệ có kèm script python -> 200 OK với has_scripts = true, script_files liệt kê
    let script_zip = create_in_memory_zip(&[
        (
            "SKILL.md",
            b"---\nname: py-skill\ndescription: Skill with script\n---\nBody",
        ),
        (
            "scripts/run.py",
            b"print('safe script - never executed by dandori')",
        ),
    ]);
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/skills-md/workspaces/{}/skills/import-zip",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "file_name": "script.zip",
                "base64_content": base64_encode(&script_zip)
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let val: Value = serde_json::from_slice(&body).unwrap();
    assert!(val.get("has_scripts").unwrap().as_bool().unwrap());
    let scripts = val.get("script_files").unwrap().as_array().unwrap();
    assert_eq!(scripts.len(), 1);
    assert_eq!(scripts[0].as_str().unwrap(), "scripts/run.py");
}

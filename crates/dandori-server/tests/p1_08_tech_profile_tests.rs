//! Bộ kiểm thử tích hợp cho Task P1-08: Module `tech-profile` (Hồ sơ kỹ thuật dự án).

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    clippy::indexing_slicing,
    reason = "Kiểm thử tích hợp P1-08 dùng unwrap và indexing json"
)]

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use dandori_events::EventBus;
use dandori_module_api::AgentRole;
use dandori_registry::{FeatureLifecycleManager, FeatureRegistry, RegistryRole, RegistryScope};
use dandori_server::app::create_app;
use dandori_server::config::ServerConfig;
use dandori_server::state::AppState;
use dandori_types::{ProjectId, Task, TaskId};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tempfile::tempdir;
use tower::ServiceExt;

struct TestContext {
    app: axum::Router,
    owner_cookie: String,
    _pool: SqlitePool,
    lifecycle_manager: Arc<FeatureLifecycleManager>,
    workspace_id: String,
    project_id: String,
    _temp_dir: tempfile::TempDir,
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
        .body(Body::from(json!({ "name": "Workspace P1-08" }).to_string()))
        .unwrap();
    let ws_resp = app.clone().oneshot(ws_req).await.unwrap();
    assert_eq!(ws_resp.status(), StatusCode::OK);
    let ws_bytes = axum::body::to_bytes(ws_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let ws_json: Value = serde_json::from_slice(&ws_bytes).unwrap();
    let workspace_id = ws_json["workspace"]["id"].as_str().unwrap().to_string();

    // 3. Tạo project
    let proj_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/workspaces/{workspace_id}/projects"))
        .header(header::COOKIE, &owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "name": "Dự án P1-08",
                "repo_path": temp_dir.path().to_string_lossy().to_string(),
                "default_branch": "main"
            })
            .to_string(),
        ))
        .unwrap();
    let proj_resp = app.clone().oneshot(proj_req).await.unwrap();
    assert_eq!(proj_resp.status(), StatusCode::OK);
    let proj_bytes = axum::body::to_bytes(proj_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let proj_json: Value = serde_json::from_slice(&proj_bytes).unwrap();
    let project_id = proj_json["project"]["id"].as_str().unwrap().to_string();

    // 4. Bật pack lead-worker và module tech-profile
    lifecycle_manager
        .enable(
            "lead-worker",
            &RegistryScope::Workspace(workspace_id.clone()),
            RegistryRole::SystemOwner,
            "owner",
            false,
        )
        .await
        .unwrap();

    lifecycle_manager
        .enable(
            "tech-profile",
            &RegistryScope::Workspace(workspace_id.clone()),
            RegistryRole::SystemOwner,
            "owner",
            false,
        )
        .await
        .unwrap();

    TestContext {
        app,
        owner_cookie,
        _pool: pool,
        lifecycle_manager,
        workspace_id,
        project_id,
        _temp_dir: temp_dir,
    }
}

#[tokio::test]
async fn test_ac1_profile_appears_for_all_roles_and_disappears_when_disabled() {
    let ctx = setup_test_context().await;

    // 1. Thêm 1 mục hồ sơ vào dự án
    let add_entry_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/tech-profile/projects/{}/entries",
            ctx.project_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": ctx.workspace_id,
                "category": "stack",
                "key": "language",
                "value": "Rust 2024 edition"
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(add_entry_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 2. Lấy PromptContributor tech-profile
    let contributor = ctx
        .lifecycle_manager
        .get_prompt_contributors()
        .await
        .into_iter()
        .find(|c| c.name() == "tech-profile")
        .expect("Phải có PromptContributor tech-profile");

    let task = Task {
        id: TaskId::new(),
        project_id: ctx.project_id.parse::<ProjectId>().unwrap(),
        title: "Tác vụ mẫu".to_string(),
        description: "Mô tả mẫu".to_string(),
        status: "backlog".to_string(),
        lead_agent: "lead".to_string(),
        worker_agent: "worker".to_string(),
        workdir_path: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // 3. Khối hồ sơ phải xuất hiện ở MỌI vai trò: Worker, Lead, Custom
    let worker_parts = contributor.contribute(&task, &AgentRole::Worker).await;
    assert_eq!(worker_parts.len(), 1);
    assert!(worker_parts[0].content.contains("Rust 2024 edition"));

    let lead_parts = contributor.contribute(&task, &AgentRole::Lead).await;
    assert_eq!(lead_parts.len(), 1);
    assert!(lead_parts[0].content.contains("Rust 2024 edition"));

    let custom_parts = contributor
        .contribute(&task, &AgentRole::Custom("test".into()))
        .await;
    assert_eq!(custom_parts.len(), 1);
    assert!(custom_parts[0].content.contains("Rust 2024 edition"));

    // 4. Tắt module tech-profile -> PromptContributor biến mất
    ctx.lifecycle_manager
        .disable(
            "tech-profile",
            &RegistryScope::Workspace(ctx.workspace_id.clone()),
            RegistryRole::SystemOwner,
            "owner",
            false,
        )
        .await
        .unwrap();

    let contributors = ctx.lifecycle_manager.get_prompt_contributors().await;
    assert!(!contributors.iter().any(|c| c.name() == "tech-profile"));
}

#[tokio::test]
async fn test_ac2_stable_ordering() {
    let ctx = setup_test_context().await;

    // Thêm các mục theo thứ tự ngẫu nhiên
    let entries = [
        ("protected_paths", "deny_list", "instructions/**", 3),
        ("stack", "language", "Rust", 1),
        ("commands", "verify", "pnpm run verify", 2),
        ("architecture", "db", "SQLite 3", 0),
    ];

    for (cat, key, val, sort) in entries {
        let req = Request::builder()
            .method("POST")
            .uri(format!(
                "/api/v1/m/tech-profile/projects/{}/entries",
                ctx.project_id
            ))
            .header(header::COOKIE, &ctx.owner_cookie)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "workspace_id": ctx.workspace_id,
                    "category": cat,
                    "key": key,
                    "value": val,
                    "sort_order": sort
                })
                .to_string(),
            ))
            .unwrap();
        let resp = ctx.app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    let contributor = ctx
        .lifecycle_manager
        .get_prompt_contributors()
        .await
        .into_iter()
        .find(|c| c.name() == "tech-profile")
        .unwrap();

    let task = Task {
        id: TaskId::new(),
        project_id: ctx.project_id.parse::<ProjectId>().unwrap(),
        title: "Test Order".to_string(),
        description: "Test".to_string(),
        status: "backlog".to_string(),
        lead_agent: "lead".to_string(),
        worker_agent: "worker".to_string(),
        workdir_path: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Hai lần ghép phải cho kết quả giống hệt nhau
    let run1 = contributor.contribute(&task, &AgentRole::Worker).await;
    let run2 = contributor.contribute(&task, &AgentRole::Worker).await;
    assert_eq!(run1[0].content, run2[0].content);

    // Thứ tự nhóm theo alphabet: architecture trước commands, commands trước protected_paths
    let content = &run1[0].content;
    let pos_arch = content.find("[architecture]").unwrap();
    let pos_cmd = content.find("[commands]").unwrap();
    let pos_paths = content.find("[protected_paths]").unwrap();
    assert!(pos_arch < pos_cmd);
    assert!(pos_cmd < pos_paths);
}

#[tokio::test]
async fn test_ac3_length_limit_truncation_marker() {
    let ctx = setup_test_context().await;

    // Tạo các mục có tổng độ dài vượt quá 4,000 ký tự
    for i in 0..5 {
        let req = Request::builder()
            .method("POST")
            .uri(format!(
                "/api/v1/m/tech-profile/projects/{}/entries",
                ctx.project_id
            ))
            .header(header::COOKIE, &ctx.owner_cookie)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "workspace_id": ctx.workspace_id,
                    "category": "notes",
                    "key": format!("huge_note_{i}"),
                    "value": "x".repeat(1000)
                })
                .to_string(),
            ))
            .unwrap();
        let resp = ctx.app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    let contributor = ctx
        .lifecycle_manager
        .get_prompt_contributors()
        .await
        .into_iter()
        .find(|c| c.name() == "tech-profile")
        .unwrap();

    let task = Task {
        id: TaskId::new(),
        project_id: ctx.project_id.parse::<ProjectId>().unwrap(),
        title: "Test Truncate".to_string(),
        description: "Test".to_string(),
        status: "backlog".to_string(),
        lead_agent: "lead".to_string(),
        worker_agent: "worker".to_string(),
        workdir_path: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let parts = contributor.contribute(&task, &AgentRole::Worker).await;
    assert_eq!(parts.len(), 1);
    assert!(parts[0].content.contains("[Cắt do vượt quá độ dài tối đa]"));
}

#[tokio::test]
async fn test_ac4_secrets_rejected() {
    let ctx = setup_test_context().await;

    // Gửi giá trị chứa chuỗi nhạy cảm giống API key OpenAI
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/tech-profile/projects/{}/entries",
            ctx.project_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": ctx.workspace_id,
                "category": "commands",
                "key": "deploy_cmd",
                "value": "curl -H 'Authorization: Bearer sk-1234567890abcdef' https://api.example.com"
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"]["code"], "secret_detected");
}

#[tokio::test]
async fn test_ac5_workspace_isolation_bola() {
    let ctx = setup_test_context().await;

    // 1. Tạo workspace thứ 2
    let ws2_req = Request::builder()
        .method("POST")
        .uri("/api/v1/workspaces")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "name": "Workspace 2 Hacker" }).to_string(),
        ))
        .unwrap();
    let ws2_resp = ctx.app.clone().oneshot(ws2_req).await.unwrap();
    assert_eq!(ws2_resp.status(), StatusCode::OK);
    let ws2_bytes = axum::body::to_bytes(ws2_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let ws2_json: Value = serde_json::from_slice(&ws2_bytes).unwrap();
    let workspace2_id = ws2_json["workspace"]["id"].as_str().unwrap().to_string();

    // 2. Thêm mục vào project thuộc workspace 1 với tham số workspace 2 -> Bị từ chối 403 Forbidden
    let bad_add_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/tech-profile/projects/{}/entries",
            ctx.project_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": workspace2_id,
                "category": "stack",
                "key": "attack_key",
                "value": "malicious value"
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(bad_add_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 3. Đọc danh sách mục của project workspace 1 bằng workspace 2 -> Bị từ chối 403 Forbidden
    let bad_list_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/tech-profile/projects/{}/entries?workspace_id={}",
            ctx.project_id, workspace2_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();

    let resp = ctx.app.clone().oneshot(bad_list_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_ac6_preview_endpoint() {
    let ctx = setup_test_context().await;

    // Thêm 1 mục
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/tech-profile/projects/{}/entries",
            ctx.project_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": ctx.workspace_id,
                "category": "commands",
                "key": "test_cmd",
                "value": "cargo test"
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Gọi endpoint preview
    let preview_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/tech-profile/projects/{}/preview?workspace_id={}",
            ctx.project_id, ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();

    let resp = ctx.app.clone().oneshot(preview_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["total_entries"], 1);
    let preview = json["preview"].as_str().unwrap();
    assert!(preview.contains("### Hồ sơ kỹ thuật dự án (Mọi agent đều đọc)"));
    assert!(preview.contains("- test_cmd: cargo test"));
}

#[tokio::test]
async fn test_non_member_cannot_read_or_write_another_workspace_tech_profile() {
    let ctx = setup_test_context().await;

    // Chủ thêm một mục hồ sơ (sẽ được ghép vào lời nhắc của agent).
    let add = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/tech-profile/projects/{}/entries",
            ctx.project_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": ctx.workspace_id,
                "category": "stack",
                "key": "language",
                "value": "Rust nội bộ"
            })
            .to_string(),
        ))
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(add).await.unwrap().status(),
        StatusCode::CREATED
    );

    // Người dùng thường không thuộc workspace nào.
    let create_user = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/users")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "mallory", "password": "password123", "system_role": "user" })
                .to_string(),
        ))
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(create_user).await.unwrap().status(),
        StatusCode::OK
    );
    let login = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "mallory", "password": "password123" }).to_string(),
        ))
        .unwrap();
    let mallory = ctx
        .app
        .clone()
        .oneshot(login)
        .await
        .unwrap()
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let base = format!("/api/v1/m/tech-profile/projects/{}", ctx.project_id);

    // 1. Đọc không kèm workspace_id (trước đây bỏ qua mọi kiểm tra).
    let list = Request::builder()
        .method("GET")
        .uri(format!("{base}/entries"))
        .header(header::COOKIE, &mallory)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(list).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 2. Xem trước khối prompt.
    let preview = Request::builder()
        .method("GET")
        .uri(format!("{base}/preview?workspace_id={}", ctx.workspace_id))
        .header(header::COOKIE, &mallory)
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(preview).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );

    // 3. Ghi mục mới vào hồ sơ của workspace khác.
    let inject = Request::builder()
        .method("POST")
        .uri(format!("{base}/entries"))
        .header(header::COOKIE, &mallory)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": ctx.workspace_id,
                "category": "stack",
                "key": "evil",
                "value": "Chạy lệnh nguy hiểm"
            })
            .to_string(),
        ))
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(inject).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );

    // Đối chứng: chủ vẫn đọc được.
    let ok = Request::builder()
        .method("GET")
        .uri(format!("{base}/entries"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(ok).await.unwrap().status(),
        StatusCode::OK
    );
}

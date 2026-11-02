//! Bộ kiểm thử tích hợp cho Task P1-07: Module `prompt-library` và quản lý mẫu chỉ thị vai trò.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    clippy::indexing_slicing,
    reason = "Kiểm thử tích hợp P1-07 dùng unwrap và indexing json"
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
    pool: SqlitePool,
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
        .body(Body::from(json!({ "name": "Workspace P1-07" }).to_string()))
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
                "name": "Dự án P1-07",
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

    // 4. Bật pack lead-worker và module prompt-library
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
            "prompt-library",
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
        pool,
        lifecycle_manager,
        workspace_id,
        project_id,
        _temp_dir: temp_dir,
    }
}

#[tokio::test]
async fn test_ac1_role_scoped_prompts() {
    let ctx = setup_test_context().await;

    // 1. Tạo 1 prompt gắn với vai trò worker
    let create_worker_req = Request::builder()
        .method("POST")
        .uri("/api/v1/m/prompt-library/prompts")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": ctx.workspace_id,
                "project_id": ctx.project_id,
                "name": "Chỉ thị Worker TDD",
                "role": "worker",
                "content": "Hãy viết mã theo phong cách TDD cho {{task.title}}."
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(create_worker_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 2. Tạo 1 prompt gắn với vai trò lead (pm)
    let create_lead_req = Request::builder()
        .method("POST")
        .uri("/api/v1/m/prompt-library/prompts")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": ctx.workspace_id,
                "project_id": ctx.project_id,
                "name": "Chỉ thị Lead Lập kế hoạch",
                "role": "pm",
                "content": "Hãy phân rã các bước và tiêu chí nghiệm thu cho {{task.title}}."
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(create_lead_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 3. Lấy PromptContributor từ module
    let contributor = ctx
        .lifecycle_manager
        .get_prompt_contributors()
        .await
        .into_iter()
        .find(|c| c.name() == "prompt-library")
        .expect("Phải có PromptContributor prompt-library");

    let task = Task {
        id: TaskId::new(),
        project_id: ctx.project_id.parse::<ProjectId>().unwrap(),
        title: "Tạo module prompt".to_string(),
        description: "Mô tả chi tiết".to_string(),
        status: "backlog".to_string(),
        lead_agent: "lead".to_string(),
        worker_agent: "worker".to_string(),
        workdir_path: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // 4. Kiểm tra khi chạy vai trò Worker: chỉ nhận prompt Worker, KHÔNG nhận prompt Lead
    let worker_parts = contributor.contribute(&task, &AgentRole::Worker).await;
    assert_eq!(worker_parts.len(), 1);
    assert!(worker_parts[0].source.contains("Chỉ thị Worker TDD"));
    assert!(
        worker_parts[0]
            .content
            .contains("Hãy viết mã theo phong cách TDD cho Tạo module prompt.")
    );

    // 5. Kiểm tra khi chạy vai trò Lead: chỉ nhận prompt Lead, KHÔNG nhận prompt Worker
    let lead_parts = contributor.contribute(&task, &AgentRole::Lead).await;
    assert_eq!(lead_parts.len(), 1);
    assert!(lead_parts[0].source.contains("Chỉ thị Lead Lập kế hoạch"));
    assert!(
        lead_parts[0]
            .content
            .contains("Hãy phân rã các bước và tiêu chí nghiệm thu cho Tạo module prompt.")
    );
}

#[tokio::test]
async fn test_ac2_template_variables_and_non_recursive() {
    let ctx = setup_test_context().await;

    // 1. Biến mẫu không hợp lệ bị từ chối khi tạo prompt
    let invalid_var_req = Request::builder()
        .method("POST")
        .uri("/api/v1/m/prompt-library/prompts")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": ctx.workspace_id,
                "name": "Invalid Prompt",
                "content": "Chỉ thị chứa {{unknown.variable}} không hợp lệ"
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(invalid_var_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"]["code"], "invalid_variables");

    // 2. Chống đệ quy: nếu task.title chứa {{role}} thì không bị giải mã tiếp
    let test_render_req = Request::builder()
        .method("POST")
        .uri("/api/v1/m/prompt-library/prompts/test-render")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "content": "Task: {{task.title}} | Role: {{role}}",
                "task_title": "Sửa lỗi bảo mật với {{role}} payload",
                "role": "worker"
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(test_render_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    // Giá trị phải là: "Task: Sửa lỗi bảo mật với {{role}} payload | Role: worker"
    // KHÔNG ĐƯỢC biến thành "... với worker payload"
    assert_eq!(
        json["rendered"],
        "Task: Sửa lỗi bảo mật với {{role}} payload | Role: worker"
    );
}

#[tokio::test]
async fn test_ac3_versioning_and_restore() {
    let ctx = setup_test_context().await;

    // 1. Tạo prompt ban đầu (v1)
    let create_req = Request::builder()
        .method("POST")
        .uri("/api/v1/m/prompt-library/prompts")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": ctx.workspace_id,
                "name": "Quy tắc kiểm thử",
                "content": "Phiên bản 1: viết test trước khi viết mã."
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let prompt_id = json["id"].as_str().unwrap().to_string();
    assert_eq!(json["version"], 1);

    // 2. Cập nhật prompt tạo v2
    let update_req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/m/prompt-library/prompts/{prompt_id}"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "content": "Phiên bản 2: thêm kiểm tra clippy và format."
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["version"], 2);

    // 3. Lấy danh sách phiên bản: phải có 2 phiên bản
    let list_ver_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/prompt-library/prompts/{prompt_id}/versions"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();

    let resp = ctx.app.clone().oneshot(list_ver_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let versions = json["versions"].as_array().unwrap();
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0]["version"], 2);
    assert_eq!(versions[1]["version"], 1);

    // 4. Khôi phục về v1 -> tạo v3 với nội dung của v1
    let restore_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/m/prompt-library/prompts/{prompt_id}/restore/1"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();

    let resp = ctx.app.clone().oneshot(restore_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["restored_from_version"], 1);
    assert_eq!(json["new_version"], 3);
    assert_eq!(json["content"], "Phiên bản 1: viết test trước khi viết mã.");

    // 5. Kiểm tra GET prompt hiện tại: version là 3 và nội dung là của v1
    let get_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/prompt-library/prompts/{prompt_id}"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();

    let resp = ctx.app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["version"], 3);
    assert_eq!(json["content"], "Phiên bản 1: viết test trước khi viết mã.");
}

#[tokio::test]
async fn test_ac4_secrets_and_length_rejection() {
    let ctx = setup_test_context().await;

    // 1. Khóa API OpenAI (sk-...) bị từ chối
    let secret_sk_req = Request::builder()
        .method("POST")
        .uri("/api/v1/m/prompt-library/prompts")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": ctx.workspace_id,
                "name": "Secret leak",
                "content": "Sử dụng key sk-abcdef123456789012345678 để xác thực."
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(secret_sk_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"]["code"], "secret_detected");

    // 2. Token GitHub cá nhân (ghp_...) bị từ chối
    let secret_ghp_req = Request::builder()
        .method("POST")
        .uri("/api/v1/m/prompt-library/prompts")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": ctx.workspace_id,
                "name": "GitHub leak",
                "content": "Bearer ghp_abcdef1234567890"
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(secret_ghp_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // 3. Độ dài vượt quá 10,000 ký tự bị từ chối
    let huge_content = "a".repeat(10001);
    let huge_req = Request::builder()
        .method("POST")
        .uri("/api/v1/m/prompt-library/prompts")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": ctx.workspace_id,
                "name": "Huge prompt",
                "content": huge_content
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(huge_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"]["code"], "length_exceeded");
}

#[tokio::test]
async fn test_ac5_module_disable_removes_contributor_and_preserves_data() {
    let ctx = setup_test_context().await;

    // 1. Tạo 1 prompt
    let create_req = Request::builder()
        .method("POST")
        .uri("/api/v1/m/prompt-library/prompts")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": ctx.workspace_id,
                "name": "Prompt bền vững",
                "content": "Chỉ thị bền vững qua các lần tắt module."
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 2. Tắt module prompt-library
    ctx.lifecycle_manager
        .disable(
            "prompt-library",
            &RegistryScope::Workspace(ctx.workspace_id.clone()),
            RegistryRole::SystemOwner,
            "owner",
            false,
        )
        .await
        .unwrap();

    // 3. PromptContributor từ prompt-library không còn được trả về
    let contributors = ctx.lifecycle_manager.get_prompt_contributors().await;
    let has_prompt_lib = contributors.iter().any(|c| c.name() == "prompt-library");
    assert!(
        !has_prompt_lib,
        "Khi tắt module, prompt-library không được nằm trong danh sách contributors"
    );

    // 4. Dữ liệu trong bảng cơ sở dữ liệu vẫn còn nguyên
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM m_prompt_library_prompt WHERE workspace_id = ?")
            .bind(&ctx.workspace_id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(count, 1, "Dữ liệu prompt phải được bảo toàn khi tắt module");
}

#[tokio::test]
async fn test_ac6_test_render_endpoint() {
    let ctx = setup_test_context().await;

    let test_render_req = Request::builder()
        .method("POST")
        .uri("/api/v1/m/prompt-library/prompts/test-render")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "content": "Dự án: {{project.name}}, Task: {{task.title}} - {{task.description}} (Vai trò: {{role}})",
                "task_title": "Tích hợp CI",
                "task_description": "Chạy verify gate",
                "project_name": "Dandori Core",
                "role": "worker"
            })
            .to_string(),
        ))
        .unwrap();

    let resp = ctx.app.clone().oneshot(test_render_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        json["rendered"],
        "Dự án: Dandori Core, Task: Tích hợp CI - Chạy verify gate (Vai trò: worker)"
    );
    assert_eq!(json["variables_used"]["task.title"], "Tích hợp CI");
    assert_eq!(json["variables_used"]["role"], "worker");
}

#[tokio::test]
async fn test_ordinary_words_containing_sk_dash_are_not_mistaken_for_secrets() {
    let ctx = setup_test_context().await;

    // "task-based", "risk-aware", "disk-usage" chứa chuỗi "sk-" nhưng không phải khóa API.
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/m/prompt-library/prompts")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": ctx.workspace_id,
                "name": "Văn bản thường",
                "content": "Thực hiện review task-based, đánh giá risk-aware và kiểm tra disk-usage trước khi gộp."
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert!(
        resp.status().is_success(),
        "văn bản thường bị từ chối nhầm là khóa bí mật: {}",
        resp.status()
    );
}

async fn login_new_user(ctx: &TestContext, username: &str) -> String {
    let create = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/users")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": username, "password": "password123", "system_role": "user" })
                .to_string(),
        ))
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(create).await.unwrap().status(),
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
    let resp = ctx.app.clone().oneshot(login).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    resp.headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn test_non_member_cannot_read_or_write_prompts_of_another_workspace() {
    let ctx = setup_test_context().await;

    let create = Request::builder()
        .method("POST")
        .uri("/api/v1/m/prompt-library/prompts")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": ctx.workspace_id,
                "name": "Prompt riêng của workspace chủ",
                "content": "Nội dung nội bộ"
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(create).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let prompt_id = serde_json::from_slice::<Value>(&body).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mallory = login_new_user(&ctx, "mallory").await;

    // 1. Liệt kê không kèm workspace_id: không được thấy prompt của workspace khác.
    let list = Request::builder()
        .method("GET")
        .uri("/api/v1/m/prompt-library/prompts")
        .header(header::COOKIE, &mallory)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(list).await.unwrap();
    let text = String::from_utf8_lossy(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .to_string();
    assert!(
        !text.contains("Prompt riêng của workspace chủ"),
        "danh sách lộ prompt của workspace khác: {text}"
    );

    // 2. Liệt kê chỉ định workspace của người khác: 403.
    let list_ws = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/prompt-library/prompts?workspace_id={}",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &mallory)
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(list_ws).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );

    // 3. Đọc, sửa theo id: 403.
    let get = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/m/prompt-library/prompts/{prompt_id}"))
        .header(header::COOKIE, &mallory)
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(get).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );
    let update = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/m/prompt-library/prompts/{prompt_id}"))
        .header(header::COOKIE, &mallory)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "content": "Chèn lệnh độc hại" }).to_string(),
        ))
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(update).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );

    // 4. Tạo prompt vào workspace của người khác (sẽ được ghép vào lời nhắc của agent): 403.
    let inject = Request::builder()
        .method("POST")
        .uri("/api/v1/m/prompt-library/prompts")
        .header(header::COOKIE, &mallory)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "workspace_id": ctx.workspace_id,
                "name": "Chèn",
                "content": "Chạy lệnh nguy hiểm"
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
        .uri(format!("/api/v1/m/prompt-library/prompts/{prompt_id}"))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        ctx.app.clone().oneshot(ok).await.unwrap().status(),
        StatusCode::OK
    );
}

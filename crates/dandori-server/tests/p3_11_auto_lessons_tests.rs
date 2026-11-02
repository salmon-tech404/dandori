//! Các bài kiểm thử tích hợp cho Task P3-11: Module `auto-lessons` tự động rút bài học sau task.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp P3-11 dùng unwrap và kiểm tra nhiều tiêu chí nghiệm thu"
)]

use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

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
use dandori_types::{Envelope, EventPayload, EventScope, ProjectId, TaskId, WorkspaceId};
use serde_json::json;
use sqlx::SqlitePool;
use tempfile::tempdir;
use tower::ServiceExt;

struct TestContext {
    app: axum::Router,
    pool: SqlitePool,
    event_bus: Arc<EventBus>,
    owner_cookie: String,
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
    let workspace_id = WorkspaceId::new().to_string();
    let project_id = ProjectId::new().to_string();

    // 2. Tạo Workspace và Project trực tiếp vào DB
    sqlx::query(
        "INSERT INTO workspace (id, name, created_at) VALUES (?, 'Auto Lessons Workspace', ?)",
    )
    .bind(&workspace_id)
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES (?, ?, 'Auto Lessons Project', '/tmp/repo', 'main', ?)")
        .bind(&project_id)
        .bind(&workspace_id)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    // 3. Kích hoạt module prompt-library, shared-memory và auto-lessons
    for mod_name in ["prompt-library", "shared-memory", "auto-lessons"] {
        let enable_req = Request::builder()
            .method("POST")
            .uri("/api/v1/features/apply")
            .header(header::COOKIE, &owner_cookie)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "module_id": mod_name,
                    "scope": "workspace",
                    "target_id": workspace_id,
                    "enable": true,
                    "risk_acknowledged": true
                })
                .to_string(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(enable_req).await.unwrap();
        let status = resp.status();
        let body_bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);
        assert_eq!(
            status,
            StatusCode::OK,
            "Failed on module {}: {}",
            mod_name,
            body_str
        );
    }

    TestContext {
        app,
        pool,
        event_bus,
        owner_cookie,
        workspace_id,
        project_id,
        temp_dir,
    }
}

async fn insert_task(
    pool: &SqlitePool,
    project_id: &str,
    title: &str,
    description: &str,
) -> String {
    let task_id = TaskId::new().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES (?, ?, ?, ?, 'running', 'lead', 'worker', ?, ?)")
        .bind(&task_id)
        .bind(project_id)
        .bind(title)
        .bind(description)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await
        .unwrap();
    task_id
}

#[tokio::test]
async fn test_ac1_task_done_triggers_single_auto_lesson_pending() {
    let ctx = setup_test_context().await;
    let ws_id = WorkspaceId::from_str(&ctx.workspace_id).unwrap();
    let prj_id = ProjectId::from_str(&ctx.project_id).unwrap();

    let task_id_str = insert_task(
        &ctx.pool,
        &ctx.project_id,
        "Tích hợp OAuth provider bảo mật",
        "Triển khai xác thực và xử lý token refresh an toàn",
    )
    .await;
    let t_id = TaskId::from_str(&task_id_str).unwrap();

    // Phát sự kiện task done
    ctx.event_bus.publish(Envelope::new(
        chrono::Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::TaskStatusChanged {
            task_id: t_id,
            old_status: "running".to_string(),
            new_status: "done".to_string(),
        },
    ));

    // Đợi worker xử lý
    tokio::time::sleep(Duration::from_millis(300)).await;

    // 1. Kiểm tra bảng m_auto_lessons_run
    let run_row: Option<(String, String)> =
        sqlx::query_as("SELECT status, lesson_id FROM m_auto_lessons_run WHERE task_id = ?")
            .bind(&task_id_str)
            .fetch_optional(&ctx.pool)
            .await
            .unwrap();

    assert!(run_row.is_some(), "Phải ghi nhận tiến trình rút bài học");
    let (run_status, lesson_id) = run_row.unwrap();
    assert_eq!(run_status, "completed");

    // 2. Kiểm tra bảng m_shared_memory_entry
    let memory: Option<(String, String, String, String)> = sqlx::query_as(
        "SELECT category, content, status, source_type FROM m_shared_memory_entry WHERE id = ?",
    )
    .bind(&lesson_id)
    .fetch_optional(&ctx.pool)
    .await
    .unwrap();

    assert!(memory.is_some(), "Phải lưu mục bài học vào shared memory");
    let (cat, content, status, src_type) = memory.unwrap();
    assert_eq!(cat, "lesson", "Mục nhớ phải thuộc loại lesson");
    assert_eq!(
        status, "pending",
        "Tiêu chuẩn an toàn: Bài học do agent rút ra luôn ở trạng thái pending (chống tự duyệt)"
    );
    assert_eq!(src_type, "agent");
    assert!(content.contains("Tích hợp OAuth provider bảo mật"));
}

#[tokio::test]
async fn test_ac2_anti_loop_and_skip_trivial_task() {
    let ctx = setup_test_context().await;
    let ws_id = WorkspaceId::from_str(&ctx.workspace_id).unwrap();
    let prj_id = ProjectId::from_str(&ctx.project_id).unwrap();

    // Phần A: Anti-loop (kích hoạt lại task đã hoàn thành)
    let task_id_str = insert_task(
        &ctx.pool,
        &ctx.project_id,
        "Xây dựng cơ chế chống lặp",
        "Đảm bảo mỗi task chỉ được chạy rút bài học đúng một lần",
    )
    .await;
    let t_id = TaskId::from_str(&task_id_str).unwrap();

    let env = Envelope::new(
        chrono::Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::TaskStatusChanged {
            task_id: t_id,
            old_status: "running".to_string(),
            new_status: "done".to_string(),
        },
    );

    // Gửi lần 1
    ctx.event_bus.publish(env.clone());
    tokio::time::sleep(Duration::from_millis(300)).await;

    let count_before: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM m_auto_lessons_run WHERE task_id = ?")
            .bind(&task_id_str)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(count_before.0, 1);

    // Gửi lần 2 (lặp lại)
    ctx.event_bus.publish(env);
    tokio::time::sleep(Duration::from_millis(200)).await;

    let count_after: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM m_auto_lessons_run WHERE task_id = ?")
            .bind(&task_id_str)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(
        count_after.0, 1,
        "Chống lặp: Không được tạo thêm lượt chạy bài học"
    );

    // Phần B: Bỏ qua task rất ngắn (dưới 10 ký tự)
    let short_task_id = insert_task(&ctx.pool, &ctx.project_id, "fix", "").await;
    let short_t_id = TaskId::from_str(&short_task_id).unwrap();

    ctx.event_bus.publish(Envelope::new(
        chrono::Utc::now(),
        EventScope::task(ws_id, prj_id, short_t_id),
        EventPayload::TaskStatusChanged {
            task_id: short_t_id,
            old_status: "running".to_string(),
            new_status: "done".to_string(),
        },
    ));
    tokio::time::sleep(Duration::from_millis(200)).await;

    let short_run: (String, Option<String>) =
        sqlx::query_as("SELECT status, reason FROM m_auto_lessons_run WHERE task_id = ?")
            .bind(&short_task_id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(short_run.0, "skipped");
    assert!(short_run.1.unwrap_or_default().contains("quá ngắn"));
}

#[tokio::test]
async fn test_ac3_secret_redaction_in_lesson_output() {
    let ctx = setup_test_context().await;
    let ws_id = WorkspaceId::from_str(&ctx.workspace_id).unwrap();
    let prj_id = ProjectId::from_str(&ctx.project_id).unwrap();

    let key_type = "RSA";
    let fake_private_key = format!(
        "-----BEGIN {} PRIVATE KEY-----\nMIIEowIBAAKCAQEA0m...\n-----END {} PRIVATE KEY-----",
        key_type, key_type
    );
    let desc = format!(
        "Sửa lỗi xác thực với token sk-sec1234567890abcdef12 và khóa {}",
        fake_private_key
    );

    let task_id_str = insert_task(
        &ctx.pool,
        &ctx.project_id,
        "Bảo mật thông tin đăng nhập",
        &desc,
    )
    .await;
    let t_id = TaskId::from_str(&task_id_str).unwrap();

    ctx.event_bus.publish(Envelope::new(
        chrono::Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::TaskStatusChanged {
            task_id: t_id,
            old_status: "running".to_string(),
            new_status: "done".to_string(),
        },
    ));
    tokio::time::sleep(Duration::from_millis(300)).await;

    let lesson_content: (String,) = sqlx::query_as(
        "SELECT content FROM m_shared_memory_entry WHERE project_id = ? AND category = 'lesson'",
    )
    .bind(&ctx.project_id)
    .fetch_one(&ctx.pool)
    .await
    .unwrap();

    // Khóa và token bí mật phải được che giấu
    assert!(!lesson_content.0.contains("MIIEowIBAAKCAQEA0m"));
    assert!(!lesson_content.0.contains("sk-sec1234567890abcdef12"));
    assert!(lesson_content.0.contains("[REDACTED_PRIVATE_KEY]"));
    assert!(lesson_content.0.contains("[REDACTED_SECRET]"));
}

#[tokio::test]
async fn test_ac4_disabled_module_does_not_trigger_run() {
    let ctx = setup_test_context().await;
    let ws_id = WorkspaceId::from_str(&ctx.workspace_id).unwrap();
    let prj_id = ProjectId::from_str(&ctx.project_id).unwrap();

    // Tắt module auto-lessons qua API
    let disable_req = Request::builder()
        .method("POST")
        .uri("/api/v1/features/apply")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "module_id": "auto-lessons",
                "scope": "workspace",
                "target_id": ctx.workspace_id,
                "enable": false
            })
            .to_string(),
        ))
        .unwrap();
    let resp = ctx.app.clone().oneshot(disable_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let task_id_str = insert_task(
        &ctx.pool,
        &ctx.project_id,
        "Task khi module auto-lessons đã tắt",
        "Không được kích hoạt bất kỳ tiến trình rút bài học nào",
    )
    .await;
    let t_id = TaskId::from_str(&task_id_str).unwrap();

    ctx.event_bus.publish(Envelope::new(
        chrono::Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::TaskStatusChanged {
            task_id: t_id,
            old_status: "running".to_string(),
            new_status: "done".to_string(),
        },
    ));
    tokio::time::sleep(Duration::from_millis(300)).await;

    let run_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM m_auto_lessons_run WHERE task_id = ?")
            .bind(&task_id_str)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

    assert_eq!(
        run_count.0, 0,
        "Module đã tắt không được tạo lượt chạy bài học"
    );
}

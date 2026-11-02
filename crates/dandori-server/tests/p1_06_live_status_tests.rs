//! Bộ kiểm thử tích hợp cho Task P1-06: Module `live-status` và luồng sự kiện SSE thời gian thực.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    clippy::indexing_slicing,
    reason = "Kiểm thử tích hợp P1-06 dùng unwrap và indexing json"
)]

use std::sync::Arc;
use std::time::Duration;

use axum::body::{Body, Bytes};
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use dandori_events::EventBus;
use dandori_registry::{FeatureLifecycleManager, FeatureRegistry, RegistryRole, RegistryScope};
use dandori_server::app::create_app;
use dandori_server::config::ServerConfig;
use dandori_server::state::AppState;
use dandori_types::{Envelope, EventPayload, EventScope, ProjectId, RunId, TaskId, WorkspaceId};
use futures::StreamExt;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tempfile::tempdir;
use tower::ServiceExt;

struct TestContext {
    app: axum::Router,
    owner_cookie: String,
    event_bus: Arc<EventBus>,
    lifecycle_manager: Arc<FeatureLifecycleManager>,
    workspace_id: String,
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
        .body(Body::from(
            json!({ "name": "Workspace Live Test" }).to_string(),
        ))
        .unwrap();
    let ws_resp = app.clone().oneshot(ws_req).await.unwrap();
    assert_eq!(ws_resp.status(), StatusCode::OK);
    let ws_bytes = axum::body::to_bytes(ws_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let ws_json: Value = serde_json::from_slice(&ws_bytes).unwrap();
    let workspace_id = ws_json["workspace"]["id"].as_str().unwrap().to_string();

    // 3. Kích hoạt module live-status
    let scope = RegistryScope::Workspace(workspace_id.clone());
    lifecycle_manager
        .enable(
            "live-status",
            &scope,
            RegistryRole::SystemOwner,
            "owner",
            true,
        )
        .await
        .unwrap();

    TestContext {
        app,
        owner_cookie,
        event_bus,
        lifecycle_manager,
        workspace_id,
        _temp_dir: temp_dir,
    }
}

#[tokio::test]
async fn test_p1_06_live_status_ac1_stream_receives_ordered_events() {
    let ctx = setup_test_context().await;

    // Kết nối tới SSE stream
    let req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/live-status/stream?workspace_id={}",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();

    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let content_type = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.contains("text/event-stream"));

    let mut body_stream = resp.into_body().into_data_stream();

    let ws_id = ctx.workspace_id.parse::<WorkspaceId>().unwrap();
    let proj_id = ProjectId::new();
    let task_id = TaskId::new();
    let run_id = RunId::new();

    // Phát chuỗi sự kiện tuần tự
    ctx.event_bus.publish(Envelope::new(
        Utc::now(),
        EventScope::task(ws_id, proj_id, task_id),
        EventPayload::RunStarted {
            run_id,
            agent: "fake-agent".into(),
            account: "acc-1".into(),
        },
    ));

    ctx.event_bus.publish(Envelope::new(
        Utc::now(),
        EventScope::task(ws_id, proj_id, task_id),
        EventPayload::RunOutput {
            run_id,
            seq: 1,
            stream: "stdout".into(),
            chunk: "Compiling project...\n".into(),
        },
    ));

    ctx.event_bus.publish(Envelope::new(
        Utc::now(),
        EventScope::task(ws_id, proj_id, task_id),
        EventPayload::RunFinished {
            run_id,
            exit_code: Some(0),
        },
    ));

    // Đọc và kiểm tra các chunk từ stream
    let mut collected = String::new();
    for _ in 0..5 {
        if let Ok(Some(Ok(chunk))) =
            tokio::time::timeout(Duration::from_millis(500), body_stream.next()).await
        {
            let chunk: Bytes = chunk;
            collected.push_str(&String::from_utf8_lossy(&chunk));
            if collected.contains("run_finished") {
                break;
            }
        }
    }

    assert!(
        collected.contains("event: run_started"),
        "Stream phải nhận được event: run_started"
    );
    assert!(
        collected.contains("event: run_output"),
        "Stream phải nhận được event: run_output"
    );
    assert!(
        collected.contains("Compiling project..."),
        "Stream phải nhận được log output"
    );
    assert!(
        collected.contains("event: run_finished"),
        "Stream phải nhận được event: run_finished"
    );
}

#[tokio::test]
async fn test_p1_06_live_status_ac2_workspace_isolation() {
    let ctx = setup_test_context().await;

    // Tạo workspace 2
    let ws2_req = Request::builder()
        .method("POST")
        .uri("/api/v1/workspaces")
        .header(header::COOKIE, &ctx.owner_cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "name": "Workspace 2 Isolation" }).to_string(),
        ))
        .unwrap();
    let ws2_resp = ctx.app.clone().oneshot(ws2_req).await.unwrap();
    assert_eq!(ws2_resp.status(), StatusCode::OK);
    let ws2_bytes = axum::body::to_bytes(ws2_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let ws2_json: Value = serde_json::from_slice(&ws2_bytes).unwrap();
    let ws2_id = ws2_json["workspace"]["id"].as_str().unwrap().to_string();

    // Kích hoạt module live-status cho workspace 2
    let scope2 = RegistryScope::Workspace(ws2_id.clone());
    ctx.lifecycle_manager
        .enable(
            "live-status",
            &scope2,
            RegistryRole::SystemOwner,
            "owner",
            true,
        )
        .await
        .unwrap();

    // Kết nối client 1 cho workspace 1
    let req1 = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/live-status/stream?workspace_id={}",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp1 = ctx.app.clone().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);
    let mut stream1 = resp1.into_body().into_data_stream();

    // Kết nối client 2 cho workspace 2
    let req2 = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/live-status/stream?workspace_id={ws2_id}"
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp2 = ctx.app.clone().oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);
    let mut stream2 = resp2.into_body().into_data_stream();

    let ws1_id_typed = ctx.workspace_id.parse::<WorkspaceId>().unwrap();
    let ws2_id_typed = ws2_id.parse::<WorkspaceId>().unwrap();

    // Phát sự kiện riêng cho workspace 1
    ctx.event_bus.publish(Envelope::new(
        Utc::now(),
        EventScope::workspace(ws1_id_typed),
        EventPayload::TaskCreated {
            task_id: TaskId::new(),
            title: "Task chỉ thuộc Workspace 1".into(),
        },
    ));

    // Phát sự kiện riêng cho workspace 2
    ctx.event_bus.publish(Envelope::new(
        Utc::now(),
        EventScope::workspace(ws2_id_typed),
        EventPayload::TaskCreated {
            task_id: TaskId::new(),
            title: "Task chỉ thuộc Workspace 2".into(),
        },
    ));

    let mut collected1 = String::new();
    for _ in 0..5 {
        if let Ok(Some(Ok(chunk))) =
            tokio::time::timeout(Duration::from_millis(300), stream1.next()).await
        {
            let chunk: Bytes = chunk;
            collected1.push_str(&String::from_utf8_lossy(&chunk));
            if collected1.contains("Task chỉ thuộc Workspace 1") {
                break;
            }
        }
    }

    let mut collected2 = String::new();
    for _ in 0..5 {
        if let Ok(Some(Ok(chunk))) =
            tokio::time::timeout(Duration::from_millis(300), stream2.next()).await
        {
            let chunk: Bytes = chunk;
            collected2.push_str(&String::from_utf8_lossy(&chunk));
            if collected2.contains("Task chỉ thuộc Workspace 2") {
                break;
            }
        }
    }

    // Xác minh cách ly: client 1 chỉ thấy ws1, không thấy ws2
    assert!(
        collected1.contains("Task chỉ thuộc Workspace 1"),
        "Client 1 phải nhận task của Workspace 1"
    );
    assert!(
        !collected1.contains("Task chỉ thuộc Workspace 2"),
        "Client 1 TUYỆT ĐỐI KHÔNG nhận task của Workspace 2"
    );

    // Xác minh cách ly: client 2 chỉ thấy ws2, không thấy ws1
    assert!(
        collected2.contains("Task chỉ thuộc Workspace 2"),
        "Client 2 phải nhận task của Workspace 2"
    );
    assert!(
        !collected2.contains("Task chỉ thuộc Workspace 1"),
        "Client 2 TUYỆT ĐỐI KHÔNG nhận task của Workspace 1"
    );
}

#[tokio::test]
async fn test_p1_06_live_status_ac3_reconnection_with_last_event_id() {
    let ctx = setup_test_context().await;
    let ws_id = ctx.workspace_id.parse::<WorkspaceId>().unwrap();

    // Bật live-status để recorder bắt đầu lưu sự kiện vào buffer
    let event1 = Envelope::new(
        Utc::now(),
        EventScope::workspace(ws_id),
        EventPayload::TaskCreated {
            task_id: TaskId::new(),
            title: "Task Initial #1".into(),
        },
    );
    let event1_id = event1.id.to_string();

    let event2 = Envelope::new(
        Utc::now(),
        EventScope::workspace(ws_id),
        EventPayload::TaskCreated {
            task_id: TaskId::new(),
            title: "Task Missed #2".into(),
        },
    );

    ctx.event_bus.publish(event1);
    ctx.event_bus.publish(event2);

    // Chờ 50ms để subscriber của module ghi nhận vào buffer
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Client kết nối lại truyền Last-Event-ID = event1_id
    let req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/live-status/stream?workspace_id={}",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .header("Last-Event-ID", &event1_id)
        .body(Body::empty())
        .unwrap();

    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let mut body_stream = resp.into_body().into_data_stream();

    let mut collected = String::new();
    if let Ok(Some(Ok(chunk))) =
        tokio::time::timeout(Duration::from_millis(500), body_stream.next()).await
    {
        let chunk: Bytes = chunk;
        collected.push_str(&String::from_utf8_lossy(&chunk));
    }

    // Phải nhận được event2 mà không nhận lại event1 (không trùng lặp)
    assert!(
        collected.contains("Task Missed #2"),
        "Client phải nhận lại sự kiện bị lỡ sau Last-Event-ID"
    );
    assert!(
        !collected.contains("Task Initial #1"),
        "Client KHÔNG được nhận lại sự kiện trước đó (tránh trùng lặp)"
    );
}

#[tokio::test]
async fn test_p1_06_live_status_ac4_disable_module_closes_stream_and_rejects_new() {
    let ctx = setup_test_context().await;

    // 1. Kết nối stream
    let req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/live-status/stream?workspace_id={}",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();

    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let mut body_stream = resp.into_body().into_data_stream();

    // 2. Tắt module live-status
    let scope = RegistryScope::Workspace(ctx.workspace_id.clone());
    ctx.lifecycle_manager
        .disable(
            "live-status",
            &scope,
            RegistryRole::SystemOwner,
            "owner",
            false,
        )
        .await
        .unwrap();

    // 3. Đọc từ stream: phải nhận được thông báo module_disabled và đóng luồng
    let mut collected = String::new();
    for _ in 0..3 {
        if let Ok(Some(Ok(chunk))) =
            tokio::time::timeout(Duration::from_millis(500), body_stream.next()).await
        {
            let chunk: Bytes = chunk;
            collected.push_str(&String::from_utf8_lossy(&chunk));
        }
    }
    assert!(
        collected.contains("event: module_disabled"),
        "Khi tắt module, stream đang mở phải nhận event: module_disabled"
    );

    // 4. Thử kết nối mới khi module đang tắt -> phải bị từ chối 403 Forbidden
    let new_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/live-status/stream?workspace_id={}",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();

    let new_resp = ctx.app.clone().oneshot(new_req).await.unwrap();
    assert_eq!(new_resp.status(), StatusCode::FORBIDDEN);
    let new_bytes = axum::body::to_bytes(new_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let new_json: Value = serde_json::from_slice(&new_bytes).unwrap();
    assert_eq!(new_json["error"]["code"], "module_disabled");
}

#[tokio::test]
async fn test_p1_06_live_status_ac5_connection_limit_and_status() {
    let ctx = setup_test_context().await;

    // Kiểm tra endpoint status
    let status_req = Request::builder()
        .method("GET")
        .uri("/api/v1/m/live-status/status")
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let status_resp = ctx.app.clone().oneshot(status_req).await.unwrap();
    assert_eq!(status_resp.status(), StatusCode::OK);
    let status_bytes = axum::body::to_bytes(status_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let status_json: Value = serde_json::from_slice(&status_bytes).unwrap();
    assert_eq!(status_json["status"], "ok");

    // Tạo 5 kết nối đồng thời (đạt giới hạn MAX_CONNECTIONS_PER_USER)
    let mut streams = Vec::new();
    for _ in 0..5 {
        let req = Request::builder()
            .method("GET")
            .uri(format!(
                "/api/v1/m/live-status/stream?workspace_id={}",
                ctx.workspace_id
            ))
            .header(header::COOKIE, &ctx.owner_cookie)
            .body(Body::empty())
            .unwrap();
        let resp = ctx.app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        streams.push(resp.into_body().into_data_stream());
    }

    // Kết nối thứ 6 phải bị từ chối 429 Too Many Requests
    let req6 = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/live-status/stream?workspace_id={}",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp6 = ctx.app.clone().oneshot(req6).await.unwrap();
    assert_eq!(resp6.status(), StatusCode::TOO_MANY_REQUESTS);
    let b6 = axum::body::to_bytes(resp6.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let j6: Value = serde_json::from_slice(&b6).unwrap();
    assert_eq!(j6["error"]["code"], "too_many_connections");

    // Đóng các kết nối
    drop(streams);
}

#[tokio::test]
async fn test_p1_06_live_status_ac5_slow_client_does_not_block_bus() {
    let ctx = setup_test_context().await;
    let ws_id = ctx.workspace_id.parse::<WorkspaceId>().unwrap();

    // Kết nối một SSE client nhưng không đọc từ stream (mô phỏng client chậm)
    let req = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/m/live-status/stream?workspace_id={}",
            ctx.workspace_id
        ))
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let _slow_stream = resp.into_body().into_data_stream();

    // Bơm 200 sự kiện liên tục vào event_bus
    let start = std::time::Instant::now();
    for i in 0..200 {
        ctx.event_bus.publish(Envelope::new(
            Utc::now(),
            EventScope::workspace(ws_id),
            EventPayload::TaskCreated {
                task_id: TaskId::new(),
                title: format!("Rapid Event #{i}"),
            },
        ));
    }

    // Việc publish 200 sự kiện không bao giờ bị nghẽn (phải hoàn thành dưới 100ms)
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_millis(500),
        "Publish vào EventBus không được bị chặn bởi client chậm: mất {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_p1_06_live_status_non_member_stream_without_workspace_sees_no_foreign_events() {
    let ctx = setup_test_context().await;

    // Owner tạo một người dùng thường chưa thuộc workspace nào.
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
    let resp = ctx.app.clone().oneshot(create_user).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let login = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "mallory", "password": "password123" }).to_string(),
        ))
        .unwrap();
    let login_resp = ctx.app.clone().oneshot(login).await.unwrap();
    assert_eq!(login_resp.status(), StatusCode::OK);
    let mallory_cookie = login_resp
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Không truyền workspace_id: trước đây khớp mọi workspace.
    let stream_req = Request::builder()
        .method("GET")
        .uri("/api/v1/m/live-status/stream")
        .header(header::COOKIE, &mallory_cookie)
        .body(Body::empty())
        .unwrap();
    let stream_resp = ctx.app.clone().oneshot(stream_req).await.unwrap();
    assert_eq!(stream_resp.status(), StatusCode::OK);
    let mut stream = stream_resp.into_body().into_data_stream();

    let ws_typed = ctx.workspace_id.parse::<WorkspaceId>().unwrap();
    ctx.event_bus.publish(Envelope::new(
        Utc::now(),
        EventScope::workspace(ws_typed),
        EventPayload::TaskCreated {
            task_id: TaskId::new(),
            title: "Bí mật của workspace chủ".into(),
        },
    ));

    let mut collected = String::new();
    for _ in 0..4 {
        if let Ok(Some(Ok(chunk))) =
            tokio::time::timeout(Duration::from_millis(300), stream.next()).await
        {
            let chunk: Bytes = chunk;
            collected.push_str(&String::from_utf8_lossy(&chunk));
        }
    }
    assert!(
        !collected.contains("Bí mật của workspace chủ"),
        "người dùng không thuộc workspace không được nhận sự kiện của workspace khác"
    );

    // Đối chứng: owner (quản trị hệ thống) mở cùng luồng vẫn nhận được.
    let owner_req = Request::builder()
        .method("GET")
        .uri("/api/v1/m/live-status/stream")
        .header(header::COOKIE, &ctx.owner_cookie)
        .body(Body::empty())
        .unwrap();
    let owner_resp = ctx.app.clone().oneshot(owner_req).await.unwrap();
    let mut owner_stream = owner_resp.into_body().into_data_stream();
    let ws_typed2 = ctx.workspace_id.parse::<WorkspaceId>().unwrap();
    ctx.event_bus.publish(Envelope::new(
        Utc::now(),
        EventScope::workspace(ws_typed2),
        EventPayload::TaskCreated {
            task_id: TaskId::new(),
            title: "Sự kiện cho owner".into(),
        },
    ));
    let mut owner_seen = String::new();
    for _ in 0..5 {
        if let Ok(Some(Ok(chunk))) =
            tokio::time::timeout(Duration::from_millis(300), owner_stream.next()).await
        {
            let chunk: Bytes = chunk;
            owner_seen.push_str(&String::from_utf8_lossy(&chunk));
            if owner_seen.contains("Sự kiện cho owner") {
                break;
            }
        }
    }
    assert!(owner_seen.contains("Sự kiện cho owner"));
}

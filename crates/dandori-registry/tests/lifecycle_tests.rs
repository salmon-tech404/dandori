//! Kiểm thử vòng đời bật/tắt module thật (T07 & 01_KIEN_TRUC.md mục 4.3, 4.4).

use std::error::Error;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use dandori_db::DbPool;
use dandori_events::EventBus;
use dandori_module_api::{
    BoxFuture, Manifest, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup, ModuleRisk,
    ModuleScope, ModuleSubscription, WorkerSpec,
};
use dandori_registry::{FeatureLifecycleManager, FeatureRegistry, RegistryRole, RegistryScope};
use dandori_types::{EventKind, EventPayload};
use tokio_util::sync::CancellationToken;
use tower::ServiceExt;

static SAMPLE_HELLO_MANIFEST: LazyLock<Manifest> = LazyLock::new(|| Manifest {
    id: "sample-hello".into(),
    group: ModuleGroup::Safety,
    name: "Module mẫu kiểm thử".into(),
    description: "Module mẫu phục vụ kiểm thử và thử cổng P0".into(),
    version: "0.1.0".into(),
    requires: vec![],
    scopes: vec![ModuleScope::Workspace, ModuleScope::Project],
    effort: ModuleEffort::Light,
    risk: ModuleRisk::Low,
    risk_note: None,
    core: false,
    requires_worker_restart: true,
    settings: Default::default(),
    ui: Default::default(),
});

static SAMPLE_MIGRATIONS: [Migration; 1] = [Migration {
    id: "0001_initial",
    sql: "CREATE TABLE IF NOT EXISTS m_sample_hello_note (\
            id TEXT PRIMARY KEY,\
            workspace_id TEXT NOT NULL,\
            project_id TEXT NOT NULL,\
            note TEXT NOT NULL,\
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))\
          );",
}];

#[derive(Debug, Default, Clone)]
struct SampleHelloModule {
    tick_counter: Arc<AtomicU64>,
}

impl SampleHelloModule {
    fn new() -> Self {
        Self {
            tick_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    fn tick_count(&self) -> u64 {
        self.tick_counter.load(Ordering::Relaxed)
    }
}

impl Module for SampleHelloModule {
    fn manifest(&self) -> &Manifest {
        &SAMPLE_HELLO_MANIFEST
    }

    fn migrations(&self) -> &'static [Migration] {
        &SAMPLE_MIGRATIONS
    }

    fn routes(&self, _ctx: &ModuleContext) -> axum::Router {
        axum::Router::new().route(
            "/ping",
            axum::routing::get(|| async {
                axum::Json(serde_json::json!({
                    "status": "ok",
                    "module": "sample-hello"
                }))
            }),
        )
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        let counter = Arc::clone(&self.tick_counter);
        vec![WorkerSpec::new(
            "sample-ticker",
            move |cancel_token: CancellationToken| {
                let counter = Arc::clone(&counter);
                Box::pin(async move {
                    loop {
                        tokio::select! {
                            () = cancel_token.cancelled() => break,
                            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                                counter.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                }) as BoxFuture<'static, ()>
            },
        )]
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        vec![ModuleSubscription::new(
            "sample-listener",
            dandori_events::EventFilter::default(),
            |_envelope| Box::pin(async move {}) as BoxFuture<'static, ()>,
        )]
    }
}

#[derive(Debug)]
struct MockModule {
    manifest: Manifest,
}

impl Module for MockModule {
    fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    fn migrations(&self) -> &'static [Migration] {
        &[]
    }

    fn routes(&self, _ctx: &ModuleContext) -> axum::Router {
        axum::Router::new()
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        vec![]
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        vec![]
    }
}

#[derive(Debug)]
struct FaultyModule {
    manifest: Manifest,
}

static FAULTY_MIGRATIONS: [Migration; 1] = [Migration {
    id: "0001_broken",
    sql: "THIS IS INVALID SQL SYNTAX ERROR",
}];

impl Module for FaultyModule {
    fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    fn migrations(&self) -> &'static [Migration] {
        &FAULTY_MIGRATIONS
    }

    fn routes(&self, _ctx: &ModuleContext) -> axum::Router {
        axum::Router::new()
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        vec![]
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        vec![]
    }
}

async fn setup_lifecycle_env()
-> Result<(FeatureLifecycleManager, DbPool, Arc<EventBus>), Box<dyn Error>> {
    let pool = DbPool::in_memory().await?;
    let registry = FeatureRegistry::new(pool.inner().clone());
    registry.seed_modules_and_packs().await?;
    let event_bus = Arc::new(EventBus::new(64));
    let lifecycle = FeatureLifecycleManager::new(registry, pool.inner().clone(), event_bus.clone());
    Ok((lifecycle, pool, event_bus))
}

#[tokio::test]
async fn test_enable_lifecycle_success() -> Result<(), Box<dyn Error>> {
    let (lifecycle, pool, event_bus) = setup_lifecycle_env().await?;
    let sample = Arc::new(SampleHelloModule::new());
    lifecycle.register_module(sample.clone()).await;

    let mut rx = event_bus.subscribe(Default::default());

    let scope = RegistryScope::System;
    let enabled = lifecycle
        .enable(
            "sample-hello",
            &scope,
            RegistryRole::SystemOwner,
            "usr_1",
            true,
        )
        .await?;

    assert_eq!(enabled, vec!["sample-hello".to_string()]);

    let table_exists: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='m_sample_hello_note'",
    )
    .fetch_one(pool.inner())
    .await?;
    assert_eq!(table_exists.0, 1);

    tokio::time::sleep(Duration::from_millis(160)).await;
    assert!(sample.tick_count() > 0);

    let router = lifecycle.create_router();
    let req = Request::builder()
        .uri("/api/v1/m/sample-hello/ping")
        .body(Body::empty())?;
    let res = router.oneshot(req).await?;
    assert_eq!(res.status(), StatusCode::OK);

    let env = rx.recv().await.ok_or("kênh sự kiện bị đóng")?;
    assert_eq!(env.kind, EventKind::ModuleEnabled);
    assert!(matches!(env.payload, EventPayload::ModuleEnabled { .. }));

    Ok(())
}

#[tokio::test]
async fn test_disable_lifecycle_success() -> Result<(), Box<dyn Error>> {
    let (lifecycle, _pool, event_bus) = setup_lifecycle_env().await?;
    let sample = Arc::new(SampleHelloModule::new());
    lifecycle.register_module(sample.clone()).await;

    let scope = RegistryScope::System;
    lifecycle
        .enable(
            "sample-hello",
            &scope,
            RegistryRole::SystemOwner,
            "usr_1",
            true,
        )
        .await?;

    tokio::time::sleep(Duration::from_millis(120)).await;
    let ticks_before = sample.tick_count();
    assert!(ticks_before > 0);

    let mut rx = event_bus.subscribe(Default::default());

    let disabled = lifecycle
        .disable(
            "sample-hello",
            &scope,
            RegistryRole::SystemOwner,
            "usr_1",
            false,
        )
        .await?;
    assert_eq!(disabled, vec!["sample-hello".to_string()]);

    let ticks_at_stop = sample.tick_count();
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(sample.tick_count(), ticks_at_stop);

    let router = lifecycle.create_router();
    let req = Request::builder()
        .uri("/api/v1/m/sample-hello/ping")
        .body(Body::empty())?;
    let res = router.oneshot(req).await?;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    let body_bytes = axum::body::to_bytes(res.into_body(), 1024 * 16).await?;
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    assert_eq!(body_json["error"]["code"], "module_disabled");
    assert_eq!(body_json["error"]["message"], "module đang tắt");
    assert!(body_json["error"]["request_id"].is_string());

    let env = rx.recv().await.ok_or("kênh sự kiện bị đóng")?;
    assert_eq!(env.kind, EventKind::ModuleDisabled);
    assert!(matches!(env.payload, EventPayload::ModuleDisabled { .. }));

    Ok(())
}

async fn insert_test_note(pool: &DbPool) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        "INSERT INTO m_sample_hello_note (id, workspace_id, project_id, note) VALUES (?, ?, ?, ?)",
    )
    .bind("note_1")
    .bind("ws_test")
    .bind("prj_test")
    .bind("Ghi chú thử nghiệm Dandori")
    .execute(pool.inner())
    .await?;
    Ok(())
}

#[tokio::test]
async fn test_data_preserved_on_disable() -> Result<(), Box<dyn Error>> {
    let (lifecycle, pool, _event_bus) = setup_lifecycle_env().await?;
    let sample = Arc::new(SampleHelloModule::new());
    lifecycle.register_module(sample.clone()).await;

    let scope = RegistryScope::System;
    lifecycle
        .enable(
            "sample-hello",
            &scope,
            RegistryRole::SystemOwner,
            "usr_1",
            true,
        )
        .await?;

    insert_test_note(&pool).await?;

    lifecycle
        .disable(
            "sample-hello",
            &scope,
            RegistryRole::SystemOwner,
            "usr_1",
            false,
        )
        .await?;

    let row_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM m_sample_hello_note WHERE id = 'note_1'")
            .fetch_one(pool.inner())
            .await?;
    assert_eq!(row_count.0, 1);

    lifecycle
        .enable(
            "sample-hello",
            &scope,
            RegistryRole::SystemOwner,
            "usr_1",
            true,
        )
        .await?;

    let note: (String,) =
        sqlx::query_as("SELECT note FROM m_sample_hello_note WHERE id = 'note_1'")
            .fetch_one(pool.inner())
            .await?;
    assert_eq!(note.0, "Ghi chú thử nghiệm Dandori");
    Ok(())
}

#[tokio::test]
async fn test_migration_failure_rollback() -> Result<(), Box<dyn Error>> {
    let (lifecycle, _pool, _event_bus) = setup_lifecycle_env().await?;

    let faulty_manifest = Manifest {
        id: "mod-broken".into(),
        group: ModuleGroup::Safety,
        name: "Broken Module".into(),
        description: "Throws SQL syntax error".into(),
        version: "0.1.0".into(),
        requires: vec![],
        scopes: vec![ModuleScope::System],
        effort: ModuleEffort::Light,
        risk: ModuleRisk::Low,
        risk_note: None,
        core: false,
        requires_worker_restart: false,
        settings: Default::default(),
        ui: Default::default(),
    };

    let broken = Arc::new(FaultyModule {
        manifest: faulty_manifest,
    });
    lifecycle.register_module(broken).await;

    let res = lifecycle
        .enable(
            "mod-broken",
            &RegistryScope::System,
            RegistryRole::SystemOwner,
            "usr_1",
            true,
        )
        .await;

    assert!(res.is_err());
    assert!(!lifecycle.is_module_active("mod-broken").await);
    assert_eq!(lifecycle.active_task_count().await, 0);

    Ok(())
}

#[tokio::test]
async fn test_process_restart_restores_active_modules() -> Result<(), Box<dyn Error>> {
    let pool = DbPool::in_memory().await?;
    let registry = FeatureRegistry::new(pool.inner().clone());
    registry.seed_modules_and_packs().await?;
    let event_bus = Arc::new(EventBus::new(64));

    let sample = Arc::new(SampleHelloModule::new());

    {
        let lifecycle =
            FeatureLifecycleManager::new(registry.clone(), pool.inner().clone(), event_bus.clone());
        lifecycle.register_module(sample.clone()).await;

        lifecycle
            .enable(
                "sample-hello",
                &RegistryScope::System,
                RegistryRole::SystemOwner,
                "usr_1",
                true,
            )
            .await?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(sample.tick_count() > 0);
    }

    let fresh_sample = Arc::new(SampleHelloModule::new());
    let new_lifecycle = FeatureLifecycleManager::new(registry, pool.inner().clone(), event_bus);
    new_lifecycle.register_module(fresh_sample.clone()).await;

    new_lifecycle.restore_active_modules().await?;

    tokio::time::sleep(Duration::from_millis(150)).await;
    assert!(fresh_sample.tick_count() > 0);
    assert!(new_lifecycle.is_module_active("sample-hello").await);

    Ok(())
}

#[tokio::test]
async fn test_rs_async_004_no_leaking_tasks() -> Result<(), Box<dyn Error>> {
    let (lifecycle, _pool, _event_bus) = setup_lifecycle_env().await?;
    let sample = Arc::new(SampleHelloModule::new());
    lifecycle.register_module(sample).await;

    let scope = RegistryScope::System;
    for _ in 0..5 {
        lifecycle
            .enable(
                "sample-hello",
                &scope,
                RegistryRole::SystemOwner,
                "usr_1",
                true,
            )
            .await?;
        tokio::time::sleep(Duration::from_millis(60)).await;
        lifecycle
            .disable(
                "sample-hello",
                &scope,
                RegistryRole::SystemOwner,
                "usr_1",
                false,
            )
            .await?;
    }

    assert_eq!(lifecycle.active_task_count().await, 0);

    Ok(())
}

fn create_mock_manifest(id: &'static str, requires: Vec<String>) -> Manifest {
    Manifest {
        id: id.into(),
        group: ModuleGroup::Runtime,
        name: id.into(),
        description: id.into(),
        version: "0.1.0".into(),
        requires,
        scopes: vec![ModuleScope::System],
        effort: ModuleEffort::Light,
        risk: ModuleRisk::Low,
        risk_note: None,
        core: false,
        requires_worker_restart: false,
        settings: Default::default(),
        ui: Default::default(),
    }
}

async fn seed_parent_child_modules(pool: &DbPool) -> Result<(), Box<dyn Error>> {
    let repo = dandori_db::repo::FeatureRepo::new(pool.inner());
    repo.upsert_module(&dandori_types::FeatureModule {
        id: dandori_types::FeatureModuleId::new("mod-parent"),
        group_name: "runtime".into(),
        name: "Module Cha".into(),
        description: "Parent module".into(),
        version: "0.1.0".into(),
        requires: vec![],
        scopes: vec!["system".into()],
        risk: "low".into(),
        effort: "light".into(),
        core: false,
    })
    .await?;

    repo.upsert_module(&dandori_types::FeatureModule {
        id: dandori_types::FeatureModuleId::new("mod-child"),
        group_name: "runtime".into(),
        name: "Module Con".into(),
        description: "Child module".into(),
        version: "0.1.0".into(),
        requires: vec!["mod-parent".into()],
        scopes: vec!["system".into()],
        risk: "low".into(),
        effort: "light".into(),
        core: false,
    })
    .await?;
    Ok(())
}

#[tokio::test]
async fn test_disable_with_dependents_reverse_topo() -> Result<(), Box<dyn Error>> {
    let (lifecycle, pool, _event_bus) = setup_lifecycle_env().await?;
    seed_parent_child_modules(&pool).await?;

    let parent_mod = Arc::new(MockModule {
        manifest: create_mock_manifest("mod-parent", vec![]),
    });
    let child_mod = Arc::new(MockModule {
        manifest: create_mock_manifest("mod-child", vec!["mod-parent".into()]),
    });

    lifecycle.register_module(parent_mod).await;
    lifecycle.register_module(child_mod).await;

    let scope = RegistryScope::System;
    let enabled = lifecycle
        .enable(
            "mod-child",
            &scope,
            RegistryRole::SystemOwner,
            "usr_1",
            true,
        )
        .await?;
    assert_eq!(
        enabled,
        vec!["mod-parent".to_string(), "mod-child".to_string()]
    );

    let res = lifecycle
        .disable(
            "mod-parent",
            &scope,
            RegistryRole::SystemOwner,
            "usr_1",
            false,
        )
        .await;
    assert!(res.is_err());

    let disabled = lifecycle
        .disable(
            "mod-parent",
            &scope,
            RegistryRole::SystemOwner,
            "usr_1",
            true,
        )
        .await?;
    assert_eq!(
        disabled,
        vec!["mod-child".to_string(), "mod-parent".to_string()]
    );

    assert!(!lifecycle.is_module_active("mod-parent").await);
    assert!(!lifecycle.is_module_active("mod-child").await);

    Ok(())
}

#[tokio::test]
async fn test_requires_worker_restart() -> Result<(), Box<dyn Error>> {
    let (lifecycle, _pool, _event_bus) = setup_lifecycle_env().await?;
    let sample = Arc::new(SampleHelloModule::new());
    lifecycle.register_module(sample.clone()).await;

    let scope = RegistryScope::System;
    lifecycle
        .enable(
            "sample-hello",
            &scope,
            RegistryRole::SystemOwner,
            "usr_1",
            true,
        )
        .await?;

    tokio::time::sleep(Duration::from_millis(80)).await;
    let count_before = sample.tick_count();
    assert!(count_before > 0);

    lifecycle.restart_module_workers("sample-hello").await?;
    tokio::time::sleep(Duration::from_millis(120)).await;
    assert!(sample.tick_count() > count_before);

    Ok(())
}

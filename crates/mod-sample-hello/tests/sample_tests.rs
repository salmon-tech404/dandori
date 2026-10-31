use dandori_module_api::{Module, ModuleContext};
use mod_sample_hello::SampleHelloModule;
use std::time::Duration;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_sample_hello_module_manifest_and_migrations() {
    let module = SampleHelloModule::new();
    let manifest = module.manifest();
    assert_eq!(manifest.id, "sample-hello");
    assert!(manifest.validate().is_ok());

    let migrations = module.migrations();
    assert_eq!(migrations.len(), 1);
    assert!(migrations[0].sql.contains("m_sample_hello_note"));
}

#[tokio::test]
async fn test_sample_hello_worker_cancellation() {
    let module = SampleHelloModule::new();
    let ctx = ModuleContext::for_test("sample-hello");

    let workers = module.workers(&ctx);
    assert_eq!(workers.len(), 1);

    let cancel_token = CancellationToken::new();
    let runner = workers[0].run.clone();
    let token_clone = cancel_token.clone();

    let task_handle = tokio::spawn(async move {
        (runner)(token_clone).await;
    });

    sleep(Duration::from_millis(150)).await;
    assert!(module.tick_count() >= 1, "worker must have ticked");

    cancel_token.cancel();
    tokio::time::timeout(Duration::from_secs(1), task_handle)
        .await
        .expect("task must finish within timeout")
        .expect("task must not panic");

    let final_ticks = module.tick_count();
    sleep(Duration::from_millis(100)).await;
    assert_eq!(module.tick_count(), final_ticks, "worker must be stopped");
}

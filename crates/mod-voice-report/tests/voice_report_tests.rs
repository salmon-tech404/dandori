//! Kiểm thử cho module `voice-report`.

use dandori_module_api::{EventBus, Module, ModuleContext};
use mod_voice_report::VoiceReportModule;
use std::sync::Arc;

#[test]
fn test_ac1_voice_report_manifest_and_requires() {
    let module = VoiceReportModule::new();
    let manifest = module.manifest();

    assert_eq!(manifest.id, "voice-report");
    assert_eq!(manifest.requires, vec!["reports"]);
}

#[test]
fn test_ac2_routes_and_migrations_are_safe_and_minimal() {
    let module = VoiceReportModule::new();
    let event_bus = Arc::new(EventBus::new(10));
    let ctx = ModuleContext::new("voice-report", event_bus, serde_json::Value::Null);

    let router = module.routes(&ctx);
    let _ = router;

    let migrations = module.migrations();
    assert!(migrations.is_empty(), "voice-report không cần bảng dữ liệu");

    let subscriptions = module.subscriptions();
    assert!(subscriptions.is_empty());

    let workers = module.workers(&ctx);
    assert!(workers.is_empty());
}

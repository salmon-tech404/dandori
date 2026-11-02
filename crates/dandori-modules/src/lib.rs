// TỆP SINH TỰ ĐỘNG bởi scripts/gen-modules.mjs (T05); không sửa tay
//! Crate cầu nối gom mọi module tự động (sinh bởi gen-modules).

#![warn(missing_docs)]

use dandori_module_api::Module;
use std::sync::Arc;

/// Phiên bản của crate dandori-modules.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Khởi tạo và trả về danh sách tất cả các module có sẵn trong hệ thống.
pub fn registered_modules() -> Vec<Arc<dyn Module>> {
    let modules: Vec<Arc<dyn Module>> = vec![
        Arc::new(mod_api_key_fallback::ApiKeyFallbackModule::new()),
        Arc::new(mod_approval_gate::ApprovalGateModule::new()),
        Arc::new(mod_auth_login::AuthLoginModule::new()),
        Arc::new(mod_auto_switch::AutoSwitchModule::new()),
        Arc::new(mod_cost_tracking::CostTrackingModule::new()),
        Arc::new(mod_diff_preview::DiffPreviewModule::new()),
        Arc::new(mod_doc_agent::DocAgentModule::new()),
        Arc::new(mod_integrations::IntegrationsModule::new()),
        Arc::new(mod_kanban::KanbanModule::new()),
        Arc::new(mod_lead_worker::LeadWorkerModule::new()),
        Arc::new(mod_live_status::LiveStatusModule::new()),
        Arc::new(mod_multi_account::MultiAccountModule::new()),
        Arc::new(mod_prompt_library::PromptLibraryModule::new()),
        Arc::new(mod_quota_tracking::QuotaTrackingModule::new()),
        Arc::new(mod_run_history::RunHistoryModule::new()),
        Arc::new(mod_subscription_connect::SubscriptionConnectModule::new()),
        Arc::new(mod_tech_profile::TechProfileModule::new()),
        Arc::new(mod_test_agent::TestAgentModule::new()),
        Arc::new(mod_worktree::WorktreeModule::new()),
    ];
    #[cfg(feature = "sample-modules")]
    let modules = {
        let mut modules = modules;
        modules.push(Arc::new(mod_sample_hello::SampleHelloModule::new()));
        modules
    };
    modules
}

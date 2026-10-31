// TỆP SINH TỰ ĐỘNG bởi scripts/gen-modules.mjs (T05); không sửa tay
//! Crate cầu nối gom mọi module tự động (sinh bởi gen-modules).

#![warn(missing_docs)]

use dandori_module_api::Module;
use std::sync::Arc;

/// Phiên bản của crate dandori-modules.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Khởi tạo và trả về danh sách tất cả các module có sẵn trong hệ thống.
pub fn registered_modules() -> Vec<Arc<dyn Module>> {
    let modules: Vec<Arc<dyn Module>> = vec![Arc::new(mod_auth_login::AuthLoginModule::new())];
    #[cfg(feature = "sample-modules")]
    let modules = {
        let mut modules = modules;
        modules.push(Arc::new(mod_sample_hello::SampleHelloModule::new()));
        modules
    };
    modules
}

//! Giao diện hợp đồng module của Dandori (trait Module, Manifest, ModuleContext).

#![warn(missing_docs)]

/// Phiên bản API của module contract.
pub const MODULE_API_VERSION: &str = env!("CARGO_PKG_VERSION");

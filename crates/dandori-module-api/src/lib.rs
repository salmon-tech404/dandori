//! Giao diện hợp đồng module của Dandori (trait Module, Manifest, ModuleContext).
//!
//! Xem chi tiết tại:
//! - `01_KIEN_TRUC.md` mục 4: Hợp đồng module
//! - `02_QUY_UOC_RUST.md`: Quy chuẩn mã nguồn Rust

#![warn(missing_docs)]

pub mod context;
pub mod manifest;
pub mod module;
pub mod table;

pub use context::ModuleContext;
pub use dandori_events::EventFilter;
pub use manifest::{
    Manifest, ManifestError, ModuleEffort, ModuleGroup, ModuleRisk, ModuleScope, SettingSchema,
    UiContribution, UiDeclaration,
};
pub use module::{BoxFuture, Migration, Module, ModuleSubscription, WorkerSpec};
pub use table::{TableNameError, table_name};

/// Phiên bản API của module contract.
pub const MODULE_API_VERSION: &str = env!("CARGO_PKG_VERSION");

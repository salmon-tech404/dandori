//! Feature Registry quản lý trạng thái, vòng đời và tính hợp lệ của các module (01_KIEN_TRUC.md mục 5).

#![warn(missing_docs)]

pub mod error;
pub mod lifecycle;
pub mod registry;
pub mod types;

pub use error::RegistryError;
pub use lifecycle::FeatureLifecycleManager;
pub use registry::FeatureRegistry;
pub use types::{
    ChangeRequest, DisablePlan, EffectiveState, EnablePlan, RegistryRole, RegistryScope,
};

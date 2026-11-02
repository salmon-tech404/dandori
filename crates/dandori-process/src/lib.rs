//! Quản lý tiến trình con an toàn của Dandori.

pub mod error;
pub mod event;
pub mod executor;
pub mod resolve;
pub mod spec;

pub use error::ProcessError;
pub use event::ProcessEvent;
pub use executor::spawn;
pub use resolve::{resolve_executable, resolve_executable_in};
pub use spec::ProcessSpec;

//! Bus sự kiện trong tiến trình của Dandori (01_KIEN_TRUC.md mục 7).

#![warn(missing_docs)]

pub mod bus;
pub mod filter;
pub mod subscription;
pub mod time;

pub use bus::EventBus;
pub use filter::EventFilter;
pub use subscription::EventSubscription;
pub use time::{SystemTimeSource, TimeSource};

/// Phiên bản hiện tại của crate `dandori-events`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

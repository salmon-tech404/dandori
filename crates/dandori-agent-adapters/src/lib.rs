//! Các adapter cụ thể cho CLI agent (Claude, Gemini, Codex, fake-agent).

pub mod fake;

pub use fake::FakeAdapter;

/// Phiên bản của crate dandori-agent-adapters.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

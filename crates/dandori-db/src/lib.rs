//! Cơ sở dữ liệu SQLite và migrations lõi của Dandori.

#![warn(missing_docs)]

pub mod error;
pub mod pool;
pub mod repo;
pub mod token;

pub use error::DbError;
pub use pool::DbPool;
pub use repo::{
    AgentAccountRepo, FeatureRepo, ProjectRepo, RunEventRepo, RunRepo, SecretRepo, SessionRepo,
    TaskRepo, UserRepo, WorkspaceRepo,
};
pub use token::hash_session_token;

/// Phiên bản của crate `dandori-db`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

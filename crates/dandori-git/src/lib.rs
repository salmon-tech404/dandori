//! Thao tác git (worktree, diff, branch) an toàn của Dandori.

pub mod client;
pub mod diff;
pub mod error;
pub mod info;
pub mod sanitize;

pub use client::GitClient;
pub use diff::{
    DiffFile, DiffFileStat, DiffResult, DiffStats, parse_numstat_line,
    validate_path_inside_worktree,
};
pub use error::GitError;
pub use info::WorktreeInfo;
pub use sanitize::{sanitize_branch_name, slugify, validate_ref_name};

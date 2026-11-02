//! Giao diện hợp đồng module của Dandori (trait Module, Manifest, ModuleContext).
//!
//! Xem chi tiết tại:
//! - `01_KIEN_TRUC.md` mục 4: Hợp đồng module
//! - `02_QUY_UOC_RUST.md`: Quy chuẩn mã nguồn Rust

#![warn(missing_docs)]

pub mod access;
pub mod context;
pub mod extension;
pub mod login_probe;
pub mod manifest;
pub mod module;
pub mod net_policy;
pub mod safe_http;
pub mod secrets;
pub mod table;

pub use access::{
    SessionUser, accessible_workspace_ids, can_access_workspace, can_administer_workspace,
    project_workspace, session_user, task_workspace,
};
pub use context::ModuleContext;
pub use dandori_agent_port::AgentRole;
pub use dandori_db::hash_session_token;
pub use dandori_events::{EventBus, EventFilter, EventSubscription};
pub use dandori_git::{
    DiffFile, DiffFileStat, DiffResult, DiffStats, GitClient, GitError, WorktreeInfo,
    parse_numstat_line, sanitize_branch_name, slugify, validate_path_inside_worktree,
};
pub use dandori_process::{ProcessError, ProcessEvent, ProcessSpec, spawn};
pub use dandori_vault::{
    MasterKeyStore, MemoryKeyStore, MemorySecretStorage, OsKeyStore, SecretBox, SecretStorage,
    SqliteSecretStorage, Vault, VaultError, default_key_store,
};
pub use extension::{
    CiState, CiStatusSummary, CreatePullRequestPayload, DecoratorContext, DecoratorError,
    DecoratorResult, GitHostError, GitHostProvider, GuardDecision, GuardError, MergeMethod,
    MergeResult, PromptContributor, PromptPart, PullRequestState, ReleaseReason, RemotePullRequest,
    SystemSource, TaskRunner, TransitionActor, TransitionGuard, Workdir, WorkdirDecorator,
    WorkdirError, WorkdirProvider, safe_write_worktree_file,
};
pub use login_probe::{
    LoginProbe, probe_claude_login, probe_login_status, supports_home_isolation,
};
pub use manifest::{
    Manifest, ManifestError, ModuleEffort, ModuleGroup, ModuleRisk, ModuleScope, SettingSchema,
    UiContribution, UiDeclaration,
};
pub use module::{BoxFuture, Migration, Module, ModuleSubscription, WorkerSpec};
pub use net_policy::{
    allow_local_endpoints, is_valid_allowed_origin_entry, origin_host, origin_host_matches,
    set_allow_local_endpoints_for_tests,
};
pub use safe_http::{
    SsrfError, build_safe_client, build_safe_client_with_timeout, is_ip_blocked, read_limited_text,
    resolve_and_check, sanitize_error_message, validate_domain_in_allowlist, validate_host_safety,
    validate_redirect_target,
};
pub use secrets::{
    MIN_SECRET_TOKEN_LEN, SECRET_PREFIXES, contains_secret_like, matched_secret_prefix,
    redact_secrets, secret_token_end,
};
pub use table::{TableNameError, table_name};

/// Phiên bản API của module contract.
pub const MODULE_API_VERSION: &str = env!("CARGO_PKG_VERSION");

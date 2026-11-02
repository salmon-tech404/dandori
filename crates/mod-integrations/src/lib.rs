//! Module `integrations`: Tích hợp nền tảng Git từ xa và Linear (P2-06).

#![warn(missing_docs)]

use axum::Router;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use dandori_module_api::{
    BoxFuture, CiStatusSummary, CreatePullRequestPayload, GitHostError, GitHostProvider, Manifest,
    MergeMethod, MergeResult, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup,
    ModuleRisk, ModuleScope, ModuleSubscription, RemotePullRequest, UiContribution, UiDeclaration,
    WorkerSpec,
};
use sqlx::SqlitePool;

pub mod github;
pub mod gitlab;
pub mod handlers;
pub mod http_client;
pub mod linear;
pub mod model;
pub mod routes;
pub mod ssrf;

pub use github::GitHubProvider;
pub use gitlab::GitLabProvider;
pub use handlers::IntegrationsState;
pub use linear::LinearClient;
pub use model::{
    CreateConnectionRequest, IntegrationConnection, LinearImportRequest, LinearIssue,
    PublicIntegrationConnection,
};
pub use ssrf::{SsrfError, validate_connection_url};

/// Provider tích hợp điều phối các kết nối Git từ xa đã lưu trong cơ sở dữ liệu.
#[derive(Debug, Clone)]
pub struct DynamicGitHostProvider {
    pool: SqlitePool,
}

impl DynamicGitHostProvider {
    /// Khởi tạo `DynamicGitHostProvider` mới.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Chọn kết nối đang hoạt động. Giao diện `GitHostProvider` chưa mang theo workspace, nên chỉ chấp nhận
    /// khi **mọi** kết nối hoạt động thuộc đúng một workspace: nếu không, dùng token của workspace này cho
    /// workspace khác sẽ là rò rỉ chéo, nên từ chối (P2-07 phải truyền workspace trước khi dùng).
    async fn fetch_active_connection(
        &self,
        target_provider: Option<&str>,
    ) -> Result<Option<IntegrationConnection>, GitHostError> {
        let workspaces: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT workspace_id) FROM m_integration_connection WHERE status = 'active'",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| GitHostError::Network("Lỗi đọc kết nối tích hợp".to_string()))?;
        if workspaces > 1 {
            return Err(GitHostError::InvalidParameter(
                "Nhiều workspace có kết nối hoạt động: cần chỉ định workspace, từ chối để không dùng nhầm token"
                    .to_string(),
            ));
        }
        let conn = if let Some(p) = target_provider {
            sqlx::query_as(
                "SELECT id, workspace_id, provider, name, base_url, token_encrypted, \
                 allow_private_network, status, created_at, updated_at \
                 FROM m_integration_connection WHERE provider = ? AND status = 'active' LIMIT 1",
            )
            .bind(p)
            .fetch_optional(&self.pool)
            .await
        } else {
            sqlx::query_as(
                "SELECT id, workspace_id, provider, name, base_url, token_encrypted, \
                 allow_private_network, status, created_at, updated_at \
                 FROM m_integration_connection WHERE status = 'active' LIMIT 1",
            )
            .fetch_optional(&self.pool)
            .await
        };
        conn.map_err(|_| GitHostError::Network("Lỗi đọc kết nối tích hợp".to_string()))
    }

    async fn resolve_provider(
        &self,
        target_provider: Option<&str>,
    ) -> Result<Arc<dyn GitHostProvider>, GitHostError> {
        let conn = self.fetch_active_connection(target_provider).await?;

        let Some(c) = conn else {
            return Err(GitHostError::NotFound(
                "Chưa cấu hình kết nối Git host nào trong hệ thống".to_string(),
            ));
        };

        let Ok(url) = url::Url::parse(&c.base_url) else {
            return Err(GitHostError::InvalidParameter(
                "URL cơ sở không hợp lệ".to_string(),
            ));
        };

        let Ok(client) = http_client::build_safe_client(&url, c.allow_private_network).await else {
            return Err(GitHostError::Network(
                "Lỗi tạo HTTP client an toàn".to_string(),
            ));
        };

        let token = handlers::reveal_token(&self.pool, &c.token_encrypted)
            .await
            .map_err(GitHostError::Network)?;

        if c.provider == "gitlab" {
            Ok(Arc::new(GitLabProvider::new(client, url, token)))
        } else {
            Ok(Arc::new(GitHubProvider::new(client, url, token)))
        }
    }
}

impl GitHostProvider for DynamicGitHostProvider {
    fn name(&self) -> &'static str {
        "integrations"
    }

    fn test_connection(&self) -> BoxFuture<'static, Result<(), GitHostError>> {
        let this = self.clone();
        Box::pin(async move {
            let provider = this.resolve_provider(None).await?;
            provider.test_connection().await
        })
    }

    fn create_pull_request(
        &self,
        repo: &str,
        payload: &CreatePullRequestPayload,
    ) -> BoxFuture<'static, Result<RemotePullRequest, GitHostError>> {
        let this = self.clone();
        let repo = repo.to_string();
        let payload = payload.clone();
        Box::pin(async move {
            let provider = this.resolve_provider(None).await?;
            provider.create_pull_request(&repo, &payload).await
        })
    }

    fn get_pull_request(
        &self,
        repo: &str,
        number: u64,
    ) -> BoxFuture<'static, Result<RemotePullRequest, GitHostError>> {
        let this = self.clone();
        let repo = repo.to_string();
        Box::pin(async move {
            let provider = this.resolve_provider(None).await?;
            provider.get_pull_request(&repo, number).await
        })
    }

    fn get_ci_status(
        &self,
        repo: &str,
        git_ref: &str,
    ) -> BoxFuture<'static, Result<CiStatusSummary, GitHostError>> {
        let this = self.clone();
        let repo = repo.to_string();
        let git_ref = git_ref.to_string();
        Box::pin(async move {
            let provider = this.resolve_provider(None).await?;
            provider.get_ci_status(&repo, &git_ref).await
        })
    }

    fn merge_pull_request(
        &self,
        repo: &str,
        number: u64,
        method: MergeMethod,
    ) -> BoxFuture<'static, Result<MergeResult, GitHostError>> {
        let this = self.clone();
        let repo = repo.to_string();
        Box::pin(async move {
            let provider = this.resolve_provider(None).await?;
            provider.merge_pull_request(&repo, number, method).await
        })
    }
}

fn get_manifest() -> Manifest {
    Manifest {
        id: "integrations".to_string(),
        group: ModuleGroup::Integrations,
        name: "Tích hợp Git & Quản lý dự án".to_string(),
        description:
            "Tích hợp nền tảng Git từ xa GitHub, GitLab và hệ thống quản lý công việc Linear"
                .to_string(),
        version: "0.1.0".to_string(),
        requires: vec!["auth-login".to_string()],
        scopes: vec![ModuleScope::Workspace],
        effort: ModuleEffort::Heavy,
        risk: ModuleRisk::Medium,
        risk_note: None,
        core: false,
        requires_worker_restart: false,
        settings: HashMap::new(),
        ui: UiDeclaration {
            contributes: vec![
                UiContribution {
                    id: "integrations-settings".to_string(),
                    slot: "settings.section".to_string(),
                    label: "Tích hợp Git & Quản lý dự án".to_string(),
                    order: 50,
                },
                UiContribution {
                    id: "integrations-card".to_string(),
                    slot: "integrations.card".to_string(),
                    label: "Kết nối tích hợp".to_string(),
                    order: 10,
                },
            ],
        },
    }
}

const MIGRATIONS: &[Migration] = &[Migration {
    id: "0001_integrations",
    sql: include_str!("../migrations/0001_integrations.sql"),
}];

/// Thực thi giao diện module cho `integrations`.
#[derive(Debug, Default)]
pub struct IntegrationsModule;

impl IntegrationsModule {
    /// Khởi tạo instance mới của `IntegrationsModule`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for IntegrationsModule {
    fn manifest(&self) -> &Manifest {
        static MANIFEST: OnceLock<Manifest> = OnceLock::new();
        MANIFEST.get_or_init(get_manifest)
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        let Some(pool) = ctx.pool().cloned() else {
            return Router::new();
        };
        routes::create_routes(IntegrationsState { pool })
    }

    fn workers(&self, _: &ModuleContext) -> Vec<WorkerSpec> {
        Vec::new()
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }

    fn git_host_provider(&self, ctx: &ModuleContext) -> Option<Arc<dyn GitHostProvider>> {
        let pool = ctx.pool()?.clone();
        Some(Arc::new(DynamicGitHostProvider::new(pool)))
    }
}

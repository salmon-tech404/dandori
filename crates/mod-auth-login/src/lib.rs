//! Module xác thực và đăng nhập hệ thống của Dandori (01_KIEN_TRUC.md mục 4 & T10).

use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use dandori_module_api::{
    Manifest, Migration, Module, ModuleContext, ModuleSubscription, WorkerSpec,
};
use std::sync::LazyLock;

#[expect(
    clippy::expect_used,
    reason = "embedded static module.json validated at build/test time"
)]
static MANIFEST: LazyLock<Manifest> = LazyLock::new(|| {
    let json = include_str!("../module.json");
    serde_json::from_str(json).expect("module.json của mod-auth-login phải hợp lệ")
});

static MIGRATIONS: [Migration; 1] = [Migration {
    id: "0001_auth_login",
    sql: "CREATE TABLE IF NOT EXISTS m_auth_login_attempt (\
            key TEXT PRIMARY KEY,\
            failed_count INTEGER NOT NULL DEFAULT 0,\
            locked_until TEXT,\
            updated_at TEXT NOT NULL\
          );",
}];

/// Struct triển khai module auth-login.
#[derive(Debug, Default, Clone, Copy)]
pub struct AuthLoginModule;

impl AuthLoginModule {
    /// Khởi tạo instance mới của module auth-login.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for AuthLoginModule {
    fn manifest(&self) -> &Manifest {
        &MANIFEST
    }

    fn migrations(&self) -> &'static [Migration] {
        &MIGRATIONS
    }

    fn routes(&self, _ctx: &ModuleContext) -> Router {
        Router::new().route("/status", get(status_handler))
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        Vec::new()
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }
}

async fn status_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ready",
        "module": "auth-login",
        "core": true
    }))
}

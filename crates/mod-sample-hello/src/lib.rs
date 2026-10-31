//! Module mẫu phục vụ kiểm thử và thử nghiệm cổng P0 (01_KIEN_TRUC.md mục 4 & T05).

#![warn(missing_docs)]

use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use dandori_module_api::{
    BoxFuture, EventFilter, Manifest, Migration, Module, ModuleContext, ModuleSubscription,
    WorkerSpec,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

/// Manifest tĩnh của module sample-hello.
#[expect(
    clippy::expect_used,
    reason = "embedded static module.json validated at build/test time"
)]
static MANIFEST: LazyLock<Manifest> = LazyLock::new(|| {
    let json = include_str!("../module.json");
    serde_json::from_str(json).expect("module.json của mod-sample-hello phải hợp lệ")
});

/// Danh sách migration SQL tạo bảng m_sample_hello_note.
static MIGRATIONS: [Migration; 1] = [Migration {
    id: "0001_initial",
    sql: "CREATE TABLE IF NOT EXISTS m_sample_hello_note (\
            id TEXT PRIMARY KEY,\
            workspace_id TEXT NOT NULL,\
            project_id TEXT NOT NULL,\
            note TEXT NOT NULL,\
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))\
          );",
}];

/// Cấu trúc chính đại diện cho module sample-hello.
#[derive(Debug, Default, Clone)]
pub struct SampleHelloModule {
    tick_counter: Arc<AtomicU64>,
}

impl SampleHelloModule {
    /// Khởi tạo instance mới của module.
    pub fn new() -> Self {
        Self {
            tick_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Lấy giá trị số nhịp đếm hiện tại của worker.
    pub fn tick_count(&self) -> u64 {
        self.tick_counter.load(Ordering::Relaxed)
    }
}

async fn handle_ping() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "module": "sample-hello"
    }))
}

impl Module for SampleHelloModule {
    fn manifest(&self) -> &Manifest {
        &MANIFEST
    }

    fn migrations(&self) -> &'static [Migration] {
        &MIGRATIONS
    }

    fn routes(&self, _ctx: &ModuleContext) -> Router {
        Router::new().route("/ping", get(handle_ping))
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        let counter = Arc::clone(&self.tick_counter);
        vec![WorkerSpec::new(
            "sample-ticker",
            move |cancel_token: CancellationToken| {
                let counter = Arc::clone(&counter);
                Box::pin(async move {
                    loop {
                        tokio::select! {
                            _ = cancel_token.cancelled() => {
                                break;
                            }
                            _ = sleep(Duration::from_millis(50)) => {
                                counter.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                }) as BoxFuture<'static, ()>
            },
        )]
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        vec![ModuleSubscription::new(
            "sample-listener",
            EventFilter::default(),
            |_envelope| {
                Box::pin(async move {
                    // Xử lý sự kiện mẫu
                }) as BoxFuture<'static, ()>
            },
        )]
    }
}

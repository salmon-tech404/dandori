//! Điểm khởi động máy chủ và composition root của ứng dụng Dandori (01_KIEN_TRUC.md & T10).

pub mod app;
pub mod auth;
pub mod config;
pub mod error;
pub mod middleware;
pub mod routes;
pub mod run_service;
pub mod state;

use std::future::IntoFuture;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub use app::create_app;
pub use config::ServerConfig;
pub use error::ApiError;
pub use run_service::RunService;
pub use state::AppState;

/// Chọn kho khóa chủ của vault cho cả tiến trình: mặc định là kho của hệ điều hành (Windows Credential
/// Manager, Keychain, Secret Service) để bí mật còn giải mã được sau khi khởi động lại. Chỉ dùng bộ nhớ
/// khi cấu hình `DANDORI_VAULT_BACKEND=memory` (kiểm thử), khi đó bí mật mất sau mỗi lần khởi động.
fn configure_vault_key_store(config: &ServerConfig) {
    use std::sync::Arc;
    let (store, kind): (Arc<dyn dandori_vault::MasterKeyStore>, &'static str) =
        if config.vault_in_memory {
            tracing::warn!("Vault dùng khóa trong bộ nhớ: bí mật sẽ mất khi khởi động lại");
            (Arc::new(dandori_vault::MemoryKeyStore::shared()), "memory")
        } else {
            (Arc::new(dandori_vault::OsKeyStore::new()), "os-keyring")
        };
    if !dandori_vault::set_default_key_store(store, kind) {
        tracing::debug!("Kho khóa chủ đã được cấu hình trước đó");
    }
}

/// Khởi tạo trạng thái và các thành phần cốt lõi của máy chủ Dandori:
/// database SQLite, migrations lõi và module, event bus, registry, lifecycle manager, run service.
///
/// # Errors
/// Trả về lỗi nếu khởi tạo DB hoặc chạy migration thất bại.
#[expect(
    clippy::too_many_lines,
    reason = "Khởi tạo server state đầy đủ các adapter, bus và module"
)]
pub async fn init_server_state(
    config: &ServerConfig,
) -> anyhow::Result<(
    AppState,
    Arc<dandori_registry::lifecycle::FeatureLifecycleManager>,
)> {
    configure_vault_key_store(config);
    tokio::fs::create_dir_all(&config.data_dir).await?;
    let db_path = config.data_dir.join("dandori.db");
    let db_pool = dandori_db::DbPool::from_path(&db_path).await?;
    let pool = db_pool.inner().clone();

    // Chạy module migrations
    let modules = dandori_modules::registered_modules();
    for m in &modules {
        for mig in m.migrations() {
            sqlx::query(mig.sql).execute(&pool).await?;
        }
    }

    let event_bus = Arc::new(dandori_events::EventBus::new(100));
    let registry = dandori_registry::FeatureRegistry::new(pool.clone());
    registry.seed_modules_and_packs().await?;

    let lifecycle_manager = Arc::new(dandori_registry::lifecycle::FeatureLifecycleManager::new(
        registry,
        pool.clone(),
        Arc::clone(&event_bus),
    ));

    for m in modules {
        lifecycle_manager.register_module(m).await;
    }
    lifecycle_manager.restore_active_modules().await?;

    let run_service = Arc::new(RunService::new(
        pool.clone(),
        Arc::clone(&event_bus),
        Arc::clone(&lifecycle_manager),
        crate::run_service::DEFAULT_MAX_CONCURRENT_RUNS,
    ));

    // Nhà cung cấp `fake` chỉ đăng ký khi cờ kiểm thử bật (P1-02, mục 5).
    if config.enable_fake_provider {
        let fake_bin = std::env::current_exe()
            .ok()
            .and_then(|p| {
                p.parent().map(|d| {
                    d.join(if cfg!(windows) {
                        "fake-agent.exe"
                    } else {
                        "fake-agent"
                    })
                })
            })
            .unwrap_or_else(|| {
                PathBuf::from(if cfg!(windows) {
                    "target/debug/fake-agent.exe"
                } else {
                    "target/debug/fake-agent"
                })
            });
        let fake_adapter = Arc::new(dandori_agent_adapters::FakeAdapter::new(fake_bin));
        run_service
            .register_adapter("fake", fake_adapter.clone())
            .await;
        run_service
            .register_adapter("fake-agent", fake_adapter)
            .await;
    }

    // Đăng ký các adapter CLI chính thức (P1-02)
    let claude_adapter = Arc::new(dandori_agent_adapters::ClaudeAdapter::new("claude"));
    run_service.register_adapter("claude", claude_adapter).await;

    let agy_adapter = Arc::new(dandori_agent_adapters::AgyAdapter::new("agy"));
    run_service
        .register_adapter("agy", agy_adapter.clone())
        .await;
    run_service.register_adapter("gemini", agy_adapter).await;

    let codex_adapter = Arc::new(dandori_agent_adapters::CodexAdapter::new("codex"));
    run_service.register_adapter("codex", codex_adapter).await;

    let state = AppState::new_with_run_service(
        pool,
        event_bus,
        Arc::clone(&lifecycle_manager),
        config.clone(),
        run_service,
    );

    Ok((state, lifecycle_manager))
}

/// Khởi chạy máy chủ HTTP thật trên socket TCP nền với port được cấp phát (hỗ trợ port 0).
/// Trả về địa chỉ cục bộ (SocketAddr), handle tác vụ máy chủ và lifecycle manager.
///
/// # Errors
/// Trả về lỗi nếu không thể bind socket TCP hoặc khởi tạo tài nguyên.
pub async fn start_background_server(
    config: ServerConfig,
    cancel_token: CancellationToken,
) -> anyhow::Result<(
    SocketAddr,
    tokio::task::JoinHandle<anyhow::Result<()>>,
    Arc<dandori_registry::lifecycle::FeatureLifecycleManager>,
)> {
    let host = config.host.clone();
    let port = config.port;
    let (state, lifecycle_manager) = init_server_state(&config).await?;
    let app = create_app(state);

    let bind_addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    let local_addr = listener.local_addr()?;
    tracing::info!(%local_addr, "Máy chủ Dandori đang lắng nghe trên TCP socket");

    let lm_clone = Arc::clone(&lifecycle_manager);
    let handle =
        tokio::spawn(async move { serve_app(listener, app, lm_clone, Some(cancel_token)).await });

    Ok((local_addr, handle, lifecycle_manager))
}

/// Thời hạn tối đa chờ các yêu cầu đang chạy hoàn tất sau tín hiệu tắt; quá hạn thì đóng cưỡng bức.
pub const SHUTDOWN_DEADLINE: Duration = Duration::from_secs(10);

/// Lắng nghe và phục vụ ứng dụng axum với graceful shutdown và dọn dẹp tài nguyên.
///
/// # Errors
/// Trả về lỗi nếu axum serve thất bại.
pub async fn serve_app(
    listener: tokio::net::TcpListener,
    app: axum::Router,
    lifecycle_manager: Arc<dandori_registry::lifecycle::FeatureLifecycleManager>,
    cancel_token: Option<CancellationToken>,
) -> anyhow::Result<()> {
    serve_app_with_deadline(
        listener,
        app,
        lifecycle_manager,
        cancel_token,
        SHUTDOWN_DEADLINE,
    )
    .await
}

/// Như [`serve_app`] nhưng cho phép chỉ định thời hạn chờ yêu cầu đang chạy khi tắt.
///
/// # Errors
/// Trả về lỗi nếu axum serve thất bại.
pub async fn serve_app_with_deadline(
    listener: tokio::net::TcpListener,
    app: axum::Router,
    lifecycle_manager: Arc<dandori_registry::lifecycle::FeatureLifecycleManager>,
    cancel_token: Option<CancellationToken>,
    deadline: Duration,
) -> anyhow::Result<()> {
    let make_service = app.into_make_service_with_connect_info::<SocketAddr>();

    let shutdown = CancellationToken::new();
    let trigger = shutdown.clone();
    let signal_task = tokio::spawn(async move {
        shutdown_signal(cancel_token).await;
        trigger.cancel();
    });

    let server = axum::serve(listener, make_service)
        .with_graceful_shutdown(shutdown.clone().cancelled_owned());

    // Chờ yêu cầu đang chạy xong; quá thời hạn kể từ lúc có tín hiệu tắt thì bỏ qua (kết nối treo
    // không được giữ máy chủ sống mãi).
    let result = tokio::select! {
        res = server.into_future() => res.map_err(anyhow::Error::from),
        () = async {
            shutdown.cancelled().await;
            tokio::time::sleep(deadline).await;
        } => {
            tracing::warn!(?deadline, "Hết thời hạn chờ yêu cầu đang chạy, đóng cưỡng bức");
            Ok(())
        }
    };
    signal_task.abort();

    tracing::info!("Đang tắt êm: giải phóng các tác vụ nền của module...");
    lifecycle_manager.shutdown().await;
    tracing::info!("Máy chủ Dandori đã dừng");

    result
}

async fn shutdown_signal(cancel_token: Option<CancellationToken>) {
    // Nếu không cài được bộ bắt tín hiệu thì KHÔNG coi là đã có tín hiệu (nếu không máy chủ tắt ngay).
    let ctrl_c = async {
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut sig) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            sig.recv().await;
        } else {
            std::future::pending::<()>().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    let token_fut = async {
        if let Some(token) = cancel_token {
            token.cancelled().await;
        } else {
            std::future::pending::<()>().await;
        }
    };

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Nhận tín hiệu Ctrl+C, bắt đầu quy trình tắt êm...");
        },
        _ = terminate => {
            tracing::info!("Nhận tín hiệu SIGTERM, bắt đầu quy trình tắt êm...");
        },
        _ = token_fut => {
            tracing::info!("Nhận tín hiệu hủy nội bộ, bắt đầu quy trình tắt êm...");
        },
    }
}

/// Chạy máy chủ HTTP Dandori với cấu hình tùy biến và hỗ trợ tín hiệu tắt êm (graceful shutdown).
///
/// # Errors
/// Trả về lỗi nếu bind socket thất bại hoặc lỗi hệ thống trong quá trình chạy.
pub async fn run_server(
    config: ServerConfig,
    cancel_token: Option<CancellationToken>,
) -> anyhow::Result<()> {
    let host = config.host.clone();
    let port = config.port;
    let (state, lifecycle_manager) = init_server_state(&config).await?;
    let app = create_app(state);

    let bind_addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    let local_addr = listener.local_addr()?;
    tracing::info!(%local_addr, "Máy chủ Dandori đang lắng nghe");

    serve_app(listener, app, lifecycle_manager, cancel_token).await
}

/// Khởi chạy máy chủ HTTP Dandori.
///
/// # Errors
/// Trả về lỗi nếu khởi tạo máy chủ thất bại.
pub async fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = ServerConfig::from_env().map_err(|e| anyhow::anyhow!(e))?;
    tracing::info!(host = %config.host, port = config.port, "Khởi động Dandori Server");
    run_server(config, None).await
}

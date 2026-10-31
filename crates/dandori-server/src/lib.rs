//! Điểm khởi động máy chủ và composition root của ứng dụng Dandori (01_KIEN_TRUC.md & T10).

pub mod app;
pub mod auth;
pub mod config;
pub mod error;
pub mod middleware;
pub mod routes;
pub mod state;

pub use app::create_app;
pub use config::ServerConfig;
pub use error::ApiError;
pub use state::AppState;

/// Khởi chạy máy chủ HTTP Dandori.
///
/// # Errors
/// Trả về lỗi nếu khởi tạo máy chủ thất bại.
pub async fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = ServerConfig::default();
    tracing::info!(host = %config.host, port = config.port, "Khởi động Dandori Server");
    Ok(())
}

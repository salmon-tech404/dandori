//! Điểm khởi động máy chủ và composition root của ứng dụng Dandori.

#![warn(missing_docs)]

/// Khởi chạy máy chủ Dandori.
///
/// # Errors
/// Trả về lỗi nếu khởi tạo tracing hoặc cấu hình hệ thống thất bại.
pub fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init()
        .map_err(|e| anyhow::anyhow!("cannot install tracing subscriber: {e}"))?;
    tracing::info!("dandori: chưa có gì (khung)");
    Ok(())
}

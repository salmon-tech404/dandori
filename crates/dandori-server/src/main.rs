//! Điểm khởi động chính của binary Dandori.

use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("dandori: chưa có gì (khung khởi tạo T01)");
    Ok(())
}

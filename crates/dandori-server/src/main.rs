//! Điểm khởi động chính của binary Dandori.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dandori_server::run().await
}

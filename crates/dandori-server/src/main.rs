//! Điểm khởi động chính của binary Dandori.

fn main() -> anyhow::Result<()> {
    dandori_server::run()
}

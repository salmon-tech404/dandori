//! Điểm khởi động chính của binary Dandori.

use std::io::Write;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        let mut stdout = std::io::stdout().lock();
        let _ = writeln!(
            stdout,
            "Dandori - Trung tâm điều phối agent code chạy trên máy người dùng"
        );
        let _ = writeln!(stdout);
        let _ = writeln!(stdout, "SỬ DỤNG:");
        let _ = writeln!(stdout, "    dandori [LỰA_CHỌN]");
        let _ = writeln!(stdout);
        let _ = writeln!(stdout, "LỰA_CHỌN:");
        let _ = writeln!(
            stdout,
            "    -h, --help       Hiển thị thông tin trợ giúp này"
        );
        let _ = writeln!(
            stdout,
            "    -v, --version    Hiển thị phiên bản của Dandori"
        );
        return Ok(());
    }

    if args.iter().any(|a| a == "--version" || a == "-v") {
        let mut stdout = std::io::stdout().lock();
        let _ = writeln!(stdout, "dandori {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    dandori_server::run().await
}

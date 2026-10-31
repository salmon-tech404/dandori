//! Binary sinh kiểu TypeScript từ các kiểu dữ liệu Rust chia sẻ (RS-TS-001).

use std::fs;
use std::path::Path;

#[expect(
    clippy::print_stdout,
    reason = "CLI generate_types thông báo trạng thái sinh và kiểm tra ra stdout"
)]
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let check_mode = args.iter().any(|arg| arg == "--check");

    let out_dir = Path::new("shared");
    let out_path = out_dir.join("types.ts");

    let generated = dandori_types::generate_typescript_types();

    if check_mode {
        if !out_path.exists() {
            anyhow::bail!(
                "shared/types.ts chưa tồn tại. Chạy `cargo run --bin generate_types` để sinh tệp."
            );
        }
        let current = fs::read_to_string(&out_path)?;
        let current_normalized = current.replace("\r\n", "\n");
        let generated_normalized = generated.replace("\r\n", "\n");

        if current_normalized.trim() != generated_normalized.trim() {
            anyhow::bail!(
                "shared/types.ts bị lệch so với định nghĩa trong Rust. Chạy `cargo run --bin generate_types` để cập nhật."
            );
        }
        println!("generate_types --check: shared/types.ts khớp định nghĩa Rust.");
    } else {
        if !out_dir.exists() {
            fs::create_dir_all(out_dir)?;
        }
        fs::write(&out_path, generated)?;
        println!("generate_types: Đã sinh shared/types.ts thành công.");
    }

    Ok(())
}

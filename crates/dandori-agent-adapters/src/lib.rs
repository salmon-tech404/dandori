//! Các adapter cụ thể cho CLI agent (Claude, Gemini, Codex, fake-agent).

pub mod agy;
pub mod claude;
pub mod codex;
pub mod discovery;
pub mod fake;

pub use agy::AgyAdapter;
pub use claude::ClaudeAdapter;
pub use codex::CodexAdapter;
pub use discovery::{check_binary_exists, check_binary_exists_with_path, resolve_binary_path};
pub use fake::FakeAdapter;

/// Lấy sự kiện kế tiếp từ tiến trình, nhưng dừng ngay khi bên nhận phía trên đã bị hủy.
///
/// Khi bên nhận (`tx`) đóng, hàm trả về `None`; vòng chuyển tiếp thoát và `proc_rx` bị hủy theo,
/// khiến `dandori-process` diệt tiến trình con. Thiếu bước này, hủy một lần chạy đang im lặng (không
/// có đầu ra) sẽ để agent chạy ngầm tới khi hết thời hạn.
pub(crate) async fn next_event(
    proc_rx: &mut tokio::sync::mpsc::Receiver<dandori_process::ProcessEvent>,
    tx: &tokio::sync::mpsc::Sender<dandori_agent_port::AgentEvent>,
) -> Option<dandori_process::ProcessEvent> {
    tokio::select! {
        () = tx.closed() => None,
        ev = proc_rx.recv() => ev,
    }
}

/// Phiên bản của crate dandori-agent-adapters.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

use dandori_module_api::{LoginProbe, probe_claude_login};
use std::path::PathBuf;
use tempfile::tempdir;

fn fake_agent_bin() -> PathBuf {
    let name = if cfg!(windows) {
        "fake-agent.exe"
    } else {
        "fake-agent"
    };
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidate = manifest.join("../../target/debug").join(name);
    if !candidate.exists() {
        #[expect(
            clippy::disallowed_methods,
            reason = "test tự build fake-agent nếu chưa có"
        )]
        let _ = std::process::Command::new("cargo")
            .args(["build", "--package", "fake-agent"])
            .status();
    }
    candidate
}

/// `fake-agent` đọc kịch bản từ biến môi trường; thăm dò không truyền biến này nên dùng cờ `--scenario`
/// không có tác dụng. Ở đây kiểm tra phần phân tích bằng cách chạy chương trình thật không tồn tại.
#[tokio::test]
async fn test_missing_cli_is_reported_as_not_installed() {
    let home = tempdir().expect("home");
    let result =
        probe_claude_login(std::path::Path::new("dandori-no-such-cli-xyz"), home.path()).await;
    assert_eq!(result, LoginProbe::NotInstalled);
}

#[tokio::test]
async fn test_cli_without_json_status_is_unknown() {
    let home = tempdir().expect("home");
    // fake-agent không in JSON `loggedIn` khi chạy `auth status` nên phải ra Unknown (không đoán).
    let result = probe_claude_login(&fake_agent_bin(), home.path()).await;
    assert_eq!(result, LoginProbe::Unknown);
}

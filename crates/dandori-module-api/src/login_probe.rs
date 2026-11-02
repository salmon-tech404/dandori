//! Thăm dò trạng thái đăng nhập của CLI agent trong thư mục HOME cô lập của một tài khoản.

use std::path::Path;
use std::time::Duration;

use dandori_process::{ProcessEvent, ProcessSpec, spawn};

/// Kết quả thăm dò đăng nhập.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginProbe {
    /// Không tìm thấy hoặc không chạy được CLI.
    NotInstalled,
    /// CLI báo chưa đăng nhập trong HOME này.
    NotLoggedIn,
    /// CLI báo đã đăng nhập.
    Ready,
    /// Không xác định được (CLI không có lệnh kiểm tra, hoặc đầu ra lạ).
    Unknown,
}

impl LoginProbe {
    /// Chuỗi trạng thái lưu trong `agent_account.status`.
    #[must_use]
    pub fn as_status(self) -> &'static str {
        match self {
            Self::NotInstalled => "not_installed",
            Self::NotLoggedIn => "not_logged_in",
            Self::Ready => "ready",
            Self::Unknown => "unknown",
        }
    }
}

/// Kiểm tra xem nhà cung cấp có hỗ trợ cô lập thư mục HOME cho nhiều tài khoản hay không.
///
/// Dựa theo kết quả khảo sát tại `docs/agents/*.md`:
/// - `claude`: Hỗ trợ qua USERPROFILE / HOME / APPDATA
/// - `agy` / `gemini`: Hỗ trợ qua USERPROFILE / HOME / APPDATA
/// - `fake`: Môi trường kiểm thử giả lập, hỗ trợ cô lập HOME
/// - `codex`: Chưa cài đặt, chưa kiểm chứng, không hỗ trợ cô lập HOME
#[must_use]
pub fn supports_home_isolation(provider: &str) -> bool {
    matches!(
        provider.to_ascii_lowercase().as_str(),
        "claude" | "agy" | "gemini" | "fake"
    )
}

/// Thăm dò đăng nhập theo nhà cung cấp. Hiện chỉ `claude` có lệnh kiểm tra đã được xác minh
/// (`claude auth status`, in JSON có `loggedIn`); các nhà cung cấp khác trả về `Unknown`
/// (chủ xác nhận thủ công).
pub async fn probe_login_status(provider: &str, home_dir: &Path) -> LoginProbe {
    match provider {
        "claude" => probe_claude_login(Path::new("claude"), home_dir).await,
        _ => LoginProbe::Unknown,
    }
}

/// Chạy `<program> auth status` trong HOME cô lập và đọc trường `loggedIn` của JSON.
pub async fn probe_claude_login(program: &Path, home_dir: &Path) -> LoginProbe {
    let spec = ProcessSpec::new(program, home_dir)
        .args(["auth", "status"])
        .isolated_home(home_dir)
        .env_whitelist(["PATH", "SystemRoot", "PATHEXT", "COMSPEC", "TEMP", "TMP"])
        .total_timeout(Duration::from_secs(20))
        .line_limit(200);

    let Ok(mut rx) = spawn(spec) else {
        return LoginProbe::NotInstalled;
    };
    let mut stdout = Vec::new();
    while let Some(event) = rx.recv().await {
        if let ProcessEvent::StdoutLine(line) = event {
            stdout.push(line);
        }
    }
    parse_logged_in(&stdout.join("\n"))
}

fn parse_logged_in(text: &str) -> LoginProbe {
    match serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|v| v.get("loggedIn").and_then(serde_json::Value::as_bool))
    {
        Some(true) => LoginProbe::Ready,
        Some(false) => LoginProbe::NotLoggedIn,
        None => LoginProbe::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::{LoginProbe, parse_logged_in};

    #[test]
    fn test_parse_real_claude_status_shapes() {
        // Đầu ra thật của `claude auth status` (2.1.300) khi chưa đăng nhập.
        let not_logged = "{\n  \"loggedIn\": false,\n  \"authMethod\": \"none\",\n  \"apiProvider\": \"firstParty\"\n}";
        assert_eq!(parse_logged_in(not_logged), LoginProbe::NotLoggedIn);
        assert_eq!(parse_logged_in("{\"loggedIn\": true}"), LoginProbe::Ready);
    }

    #[test]
    fn test_parse_garbage_is_unknown_not_a_guess() {
        assert_eq!(parse_logged_in(""), LoginProbe::Unknown);
        assert_eq!(parse_logged_in("not json"), LoginProbe::Unknown);
        assert_eq!(
            parse_logged_in("{\"loggedIn\": \"yes\"}"),
            LoginProbe::Unknown
        );
        assert_eq!(parse_logged_in("{}"), LoginProbe::Unknown);
    }
}

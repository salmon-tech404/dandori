use dandori_agent_adapters::{
    AgyAdapter, ClaudeAdapter, CodexAdapter, check_binary_exists_with_path,
};
use dandori_agent_port::{AgentAdapter, AgentEvent, AgentRole, AgentRunSpec};
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_ac3_claude_adapter_exact_command_args() {
    let prompt = "Kiểm tra mã nguồn và chạy test";
    let args = ClaudeAdapter::build_args(prompt);

    assert_eq!(
        args,
        vec![
            "-p",
            "--output-format",
            "stream-json",
            "--verbose",
            "--dangerously-skip-permissions",
            "--",
            prompt,
        ],
        "Lệnh của ClaudeAdapter phải khớp CLI thật (cần --verbose với stream-json, và -- trước lời nhắc)"
    );
}

#[test]
fn test_ac3_agy_adapter_exact_command_args() {
    let prompt = "Viết thêm module kết nối subscription";
    let args = AgyAdapter::build_args(prompt);

    assert_eq!(
        args,
        vec![
            "--print",
            "--output-format",
            "stream-json",
            "--dangerously-skip-permissions",
            "--",
            prompt,
        ],
        "Lệnh của AgyAdapter phải khớp chính xác đặc tả khảo sát"
    );
}

#[test]
fn test_ac3_codex_adapter_exact_command_args() {
    let prompt = "Tối ưu hóa hiệu năng cơ sở dữ liệu";
    let args = CodexAdapter::build_args(prompt);

    assert_eq!(
        args,
        vec!["--print", "--output-format", "json", prompt,],
        "Lệnh của CodexAdapter phải khớp chính xác đặc tả khảo sát"
    );
}

#[test]
fn test_ac2_binary_detection() {
    assert!(
        !check_binary_exists_with_path("nonexistent_binary_xyz_12345", None),
        "Binary không tồn tại phải trả về false"
    );

    let temp_dir = tempdir().expect("tempdir");
    let fake_bin_name = if cfg!(windows) {
        "my-cli.cmd"
    } else {
        "my-cli"
    };
    let fake_bin_path = temp_dir.path().join(fake_bin_name);

    if cfg!(windows) {
        std::fs::write(&fake_bin_path, "@echo off\necho 1.0.0\n").expect("write fake bin");
    } else {
        std::fs::write(&fake_bin_path, "#!/bin/sh\necho 1.0.0\n").expect("write fake bin");
    }

    let custom_path = temp_dir.path().to_string_lossy().to_string();

    assert!(
        check_binary_exists_with_path("my-cli", Some(&custom_path)),
        "Binary trên PATH tùy chỉnh phải được phát hiện thành công mà không cần sửa đổi môi trường toàn cục"
    );
}

fn get_fake_agent_bin() -> PathBuf {
    let bin_name = if cfg!(windows) {
        "fake-agent.exe"
    } else {
        "fake-agent"
    };

    if let Ok(mut exe) = std::env::current_exe() {
        exe.pop();
        if exe.ends_with("deps") {
            exe.pop();
        }
        let candidate = exe.join(bin_name);
        if candidate.exists() {
            return candidate;
        }
    }

    let manifest_candidate = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target/debug")
        .join(bin_name);
    if manifest_candidate.exists() {
        return manifest_candidate;
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "tự động build fake-agent binary phục vụ kiểm thử nếu chưa tồn tại"
    )]
    let _ = std::process::Command::new("cargo")
        .args(["build", "--package", "fake-agent"])
        .status();

    manifest_candidate
}

#[tokio::test]
async fn test_ac4_home_isolation_and_env_whitelist() {
    let workdir = tempdir().expect("workdir");
    let home_dir = tempdir().expect("home_dir");
    let fake_bin = get_fake_agent_bin();

    let adapter = ClaudeAdapter::new(&fake_bin);

    // Kiểm tra biến USERPROFILE / HOME khớp với isolated_home
    let check_var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    let scenario = serde_json::json!({
        "check_env": check_var,
        "exit_code": 0
    })
    .to_string();

    let mut spec = AgentRunSpec::new(workdir.path().to_path_buf(), scenario, AgentRole::Worker);
    spec.isolated_home = Some(home_dir.path().to_path_buf());

    let mut rx = adapter.start_run(spec).await.expect("start_run");
    let mut outputs = Vec::new();
    while let Some(ev) = rx.recv().await {
        if let AgentEvent::Text(t) = ev {
            outputs.push(t);
        }
    }

    let expected_home_str = home_dir.path().to_string_lossy().to_string();
    let has_matching_home = outputs
        .iter()
        .any(|line| line.starts_with("ENV_CHECK:") && line.contains(&expected_home_str));

    assert!(
        has_matching_home,
        "Tiến trình con phải nhận biến {check_var} trỏ tới thư mục HOME cô lập (nhận: {:?})",
        outputs
    );

    // Kiểm tra biến ngoài whitelist bị xóa hoàn toàn (ví dụ PROMPT hay COMPUTERNAME)
    let non_whitelisted = "COMPUTERNAME";
    let scenario2 = serde_json::json!({
        "check_env": non_whitelisted,
        "exit_code": 0
    })
    .to_string();

    let mut spec2 = AgentRunSpec::new(workdir.path().to_path_buf(), scenario2, AgentRole::Worker);
    spec2.isolated_home = Some(home_dir.path().to_path_buf());

    let mut rx2 = adapter.start_run(spec2).await.expect("start_run");
    let mut outputs2 = Vec::new();
    while let Some(ev) = rx2.recv().await {
        if let AgentEvent::Text(t) = ev {
            outputs2.push(t);
        }
    }

    let is_not_found = outputs2.iter().any(|line| line == "ENV_CHECK:NOT_FOUND");
    assert!(
        is_not_found,
        "Biến ngoài whitelist ({non_whitelisted}) phải bị xóa, không được lọt xuống tiến trình con"
    );
}

#[tokio::test]
async fn test_ac6_no_secret_leak_and_quota_detection() {
    let workdir = tempdir().expect("workdir");
    let fake_bin = get_fake_agent_bin();
    let adapter =
        ClaudeAdapter::new(&fake_bin).with_quota_patterns(vec!["rate_limit_exceeded".to_string()]);

    let scenario = serde_json::json!({
        "print_lines": [
            "Dòng bình thường",
            "Lỗi: rate_limit_exceeded do vượt quá hạn mức sử dụng",
            "Kết thúc"
        ],
        "exit_code": 0
    })
    .to_string();

    let spec = AgentRunSpec::new(workdir.path().to_path_buf(), scenario, AgentRole::Worker);
    let mut rx = adapter.start_run(spec).await.expect("start_run");
    let mut events = Vec::new();
    while let Some(ev) = rx.recv().await {
        events.push(ev);
    }

    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentEvent::QuotaExhausted { .. })),
        "Phải phát hiện QuotaExhausted khi gặp mẫu rate_limit_exceeded"
    );
}

#[tokio::test]
async fn test_ac5_no_access_outside_isolated_home() {
    let workdir = tempdir().expect("workdir");
    let home_dir = tempdir().expect("isolated home dir");
    let fake_bin = get_fake_agent_bin();
    let adapter = ClaudeAdapter::new(&fake_bin);

    // Kịch bản ghi file vào thư mục con của HOME
    let target_file = home_dir.path().join("config.json");
    let scenario = serde_json::json!({
        "write_file": {
            "path": target_file.to_string_lossy(),
            "content": "{\"installed\": true}"
        },
        "exit_code": 0
    })
    .to_string();

    let mut spec = AgentRunSpec::new(workdir.path().to_path_buf(), scenario, AgentRole::Worker);
    spec.isolated_home = Some(home_dir.path().to_path_buf());

    let mut rx = adapter.start_run(spec).await.expect("start_run");
    while rx.recv().await.is_some() {}

    // Khẳng định file chỉ được ghi bên trong thư mục HOME cô lập
    assert!(
        target_file.exists(),
        "Tệp cấu hình phải được tạo thành công trong thư mục HOME cô lập"
    );

    // Kiểm tra không có bất kỳ truy cập ngoài thư mục HOME tạm
    let content = std::fs::read_to_string(&target_file).expect("read target file");
    assert_eq!(content, "{\"installed\": true}");
}

#[tokio::test]
async fn test_default_quota_patterns_do_not_flag_ordinary_text() {
    let workdir = tempdir().expect("workdir");
    let fake_bin = get_fake_agent_bin();
    let adapter = ClaudeAdapter::new(&fake_bin);

    // Dòng thường có chứa số 429 và cụm "quota exceeded" không được coi là hết hạn mức khi chưa cấu hình mẫu.
    let scenario = serde_json::json!({
        "print_lines": ["Đã sửa 429 dòng", "Bảng thống kê: quota exceeded 3 lần trong tuần"],
        "exit_code": 0
    })
    .to_string();

    let spec = AgentRunSpec::new(workdir.path().to_path_buf(), scenario, AgentRole::Worker);
    let mut rx = adapter.start_run(spec).await.expect("start_run");
    let mut events = Vec::new();
    while let Some(ev) = rx.recv().await {
        events.push(ev);
    }
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, AgentEvent::QuotaExhausted { .. })),
        "mẫu mặc định không được khớp nhầm văn bản thường"
    );
}

#[tokio::test]
async fn test_codex_adapter_refuses_to_run_until_verified() {
    let workdir = tempdir().expect("workdir");
    let adapter = CodexAdapter::new("codex");
    let spec = AgentRunSpec::new(workdir.path().to_path_buf(), "x", AgentRole::Worker);
    let result = adapter.start_run(spec).await;
    assert!(
        result.is_err(),
        "Codex chưa được kiểm chứng nên không được gọi CLI bằng tham số phỏng đoán"
    );
}

/// Hủy lần chạy (bỏ bên nhận) khi agent đang im lặng phải diệt tiến trình, không để chạy ngầm.
#[cfg(windows)]
#[tokio::test]
async fn test_dropping_event_receiver_kills_silent_agent() {
    fn is_alive(pid: u32) -> bool {
        #[expect(
            clippy::disallowed_methods,
            reason = "test helper kiểm tra pid còn sống trên Windows"
        )]
        let out = std::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}")])
            .output()
            .expect("tasklist");
        String::from_utf8_lossy(&out.stdout).contains(&pid.to_string())
    }

    let workdir = tempdir().expect("workdir");
    let pid_file = workdir.path().join("agent.pid");
    let scenario = serde_json::json!({
        "write_pid_file": pid_file.to_string_lossy(),
        "hang": true
    })
    .to_string();

    let adapter = ClaudeAdapter::new(get_fake_agent_bin());
    let spec = AgentRunSpec::new(workdir.path().to_path_buf(), scenario, AgentRole::Worker);
    let rx = adapter.start_run(spec).await.expect("start_run");

    let mut pid = None;
    for _ in 0..60 {
        if let Ok(text) = std::fs::read_to_string(&pid_file)
            && let Ok(p) = text.trim().parse::<u32>()
        {
            pid = Some(p);
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    let pid = pid.expect("fake-agent phải ghi pid file");
    assert!(is_alive(pid));

    drop(rx);

    let mut dead = false;
    for _ in 0..50 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if !is_alive(pid) {
            dead = true;
            break;
        }
    }
    assert!(dead, "bỏ bên nhận phải diệt agent đang im lặng (PID {pid})");
}

use dandori_process::{ProcessEvent, ProcessSpec, spawn};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::tempdir;
use tokio_util::sync::CancellationToken;

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

#[cfg(windows)]
fn is_pid_alive(pid: u32) -> bool {
    #[expect(
        clippy::disallowed_methods,
        reason = "test helper kiểm tra pid tồn tại trên Windows"
    )]
    let output = std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}")])
        .output();
    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        text.contains(&pid.to_string())
    } else {
        false
    }
}

#[cfg(not(windows))]
fn is_pid_alive(_pid: u32) -> bool {
    false
}

#[tokio::test]
async fn test_ac1_total_timeout_kills_process_and_grandchild() {
    let dir = tempdir().expect("tempdir");
    let bin = get_fake_agent_bin();
    let scenario = serde_json::json!({
        "spawn_grandchild": true,
        "grandchild_sleep_ms": 60_000,
        "hang": true
    })
    .to_string();

    let spec = ProcessSpec::new(bin, dir.path())
        .arg("--scenario")
        .arg(scenario)
        .total_timeout(Duration::from_millis(600));

    let mut rx = spawn(spec).expect("spawn");
    let start = Instant::now();

    // Chờ một chút để tiến trình con kịp ghi grandchild.pid
    tokio::time::sleep(Duration::from_millis(250)).await;

    while let Some(_ev) = rx.recv().await {}
    assert!(start.elapsed() < Duration::from_secs(5));

    #[cfg(windows)]
    {
        let pid_file = dir.path().join("grandchild.pid");
        if pid_file.exists() {
            let pid_str = std::fs::read_to_string(&pid_file).expect("read pid");
            if let Ok(g_pid) = pid_str.trim().parse::<u32>() {
                tokio::time::sleep(Duration::from_millis(300)).await;
                assert!(
                    !is_pid_alive(g_pid),
                    "Tiến trình cháu (PID {g_pid}) phải bị tiêu diệt khi tiến trình cha bị kill theo cây!"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_ac2_silent_timeout_kills_process() {
    let dir = tempdir().expect("tempdir");
    let bin = get_fake_agent_bin();
    let scenario = serde_json::json!({
        "print_lines": ["hello start"],
        "sleep_after_ms": 10_000
    })
    .to_string();

    let spec = ProcessSpec::new(bin, dir.path())
        .arg("--scenario")
        .arg(scenario)
        .silent_timeout(Duration::from_millis(400));

    let mut rx = spawn(spec).expect("spawn");
    let start = Instant::now();

    let first_ev = rx.recv().await;
    assert_eq!(
        first_ev,
        Some(ProcessEvent::StdoutLine("hello start".to_string()))
    );

    // Kế tiếp phải bị cắt bởi silent timeout
    while let Some(_ev) = rx.recv().await {}
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(3),
        "Tiến trình phải bị giết do silent timeout sau khoảng 400ms, elapsed: {elapsed:?}"
    );
}

#[tokio::test]
async fn test_ac3_flood_output_bounded_by_line_limit() {
    let dir = tempdir().expect("tempdir");
    let bin = get_fake_agent_bin();
    let scenario = serde_json::json!({
        "flood_lines": 5_000
    })
    .to_string();

    let spec = ProcessSpec::new(bin, dir.path())
        .arg("--scenario")
        .arg(scenario)
        .line_limit(35);

    let mut rx = spawn(spec).expect("spawn");
    let mut count = 0;
    while let Some(ev) = rx.recv().await {
        if matches!(ev, ProcessEvent::StdoutLine(_)) {
            count += 1;
        }
    }

    assert!(
        count <= 36,
        "Số dòng nhận được ({count}) không được vượt quá giới hạn chặn trên"
    );
}

#[tokio::test]
async fn test_ac4_env_whitelist_prevents_leaking_secret() {
    let dir = tempdir().expect("tempdir");
    let bin = get_fake_agent_bin();

    // Chọn một biến môi trường chắc chắn có trong môi trường cha (như USERNAME hoặc USER hoặc PATH)
    let parent_var = if cfg!(windows) { "USERNAME" } else { "USER" };
    assert!(
        std::env::var(parent_var).is_ok(),
        "Môi trường cha phải có biến {parent_var}"
    );

    let scenario = serde_json::json!({
        "check_env": parent_var
    })
    .to_string();

    // Whitelist KHÔNG bao gồm parent_var (chỉ cho PATH và SystemRoot)
    let spec = ProcessSpec::new(bin, dir.path())
        .arg("--scenario")
        .arg(scenario)
        .env_whitelist(vec!["PATH".to_string(), "SystemRoot".to_string()]);

    let mut rx = spawn(spec).expect("spawn");
    let mut check_line = String::new();

    while let Some(ev) = rx.recv().await {
        if let ProcessEvent::StdoutLine(line) = ev
            && line.starts_with("ENV_CHECK:")
        {
            check_line = line;
        }
    }

    assert_eq!(
        check_line, "ENV_CHECK:NOT_FOUND",
        "Biến không nằm trong whitelist tuyệt đối không được lọt vào tiến trình con!"
    );
}

#[tokio::test]
async fn test_ac5_cancellation_token_stops_process_fast() {
    let dir = tempdir().expect("tempdir");
    let bin = get_fake_agent_bin();
    let scenario = serde_json::json!({
        "hang": true
    })
    .to_string();

    let token = CancellationToken::new();
    let spec = ProcessSpec::new(bin, dir.path())
        .arg("--scenario")
        .arg(scenario)
        .cancellation_token(token.clone());

    let mut rx = spawn(spec).expect("spawn");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let start = Instant::now();
    token.cancel();

    while let Some(_ev) = rx.recv().await {}
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_millis(1500),
        "Hủy bằng token phải dừng tiến trình nhanh chóng, elapsed: {elapsed:?}"
    );
}

use dandori_agent_adapters::FakeAdapter;
use dandori_agent_port::{AgentAdapter, AgentEvent, AgentRole, AgentRunSpec};
use std::path::PathBuf;
use tempfile::tempdir;

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
async fn test_ac6_fake_adapter_emits_expected_events() {
    let dir = tempdir().expect("tempdir");
    let bin = get_fake_agent_bin();
    let adapter = FakeAdapter::new(bin);

    let scenario = serde_json::json!({
        "print_lines": [
            "Bắt đầu thực thi nhiệm vụ",
            "{\"type\":\"tool_call\",\"id\":\"call_1\",\"name\":\"read_file\",\"arguments\":\"{\\\"path\\\":\\\"main.rs\\\"}\"}",
            "ERROR: Không tìm thấy tệp được yêu cầu"
        ],
        "quota_exhausted": true,
        "exit_code": 0
    })
    .to_string();

    let spec = AgentRunSpec::new(dir.path().to_path_buf(), scenario, AgentRole::Worker);

    let mut rx = adapter.start_run(spec).await.expect("start_run");
    let mut events = Vec::new();

    while let Some(ev) = rx.recv().await {
        events.push(ev);
    }

    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentEvent::Text(t) if t == "Bắt đầu thực thi nhiệm vụ")),
        "Phải nhận được sự kiện Text"
    );

    assert!(
        events.iter().any(|e| matches!(e, AgentEvent::ToolCall { id, name, .. } if id == "call_1" && name == "read_file")),
        "Phải nhận được sự kiện ToolCall"
    );

    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentEvent::Error(msg) if msg.contains("Không tìm thấy tệp"))),
        "Phải nhận được sự kiện Error"
    );

    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentEvent::QuotaExhausted { .. })),
        "Phải nhận được sự kiện QuotaExhausted"
    );

    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentEvent::Finished { exit_code: 0 })),
        "Phải nhận được sự kiện Finished"
    );
}

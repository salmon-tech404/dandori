//! Fake adapter sử dụng fake-agent binary để giả lập CLI agent trong môi trường kiểm thử.

use dandori_agent_port::{AgentAdapter, AgentEvent, AgentPortError, AgentRunSpec, BoxFuture};
use dandori_process::{ProcessEvent, ProcessSpec};
use serde_json::Value;
use std::path::PathBuf;
use tokio::sync::mpsc::{Receiver, Sender, channel};

const FORWARD_BUFFER: usize = 64;

/// Adapter giả lập tương tác với CLI agent thông qua binary `fake-agent`.
#[derive(Debug, Clone)]
pub struct FakeAdapter {
    fake_agent_bin: PathBuf,
}

impl FakeAdapter {
    /// Khởi tạo một thể hiện FakeAdapter mới trỏ tới đường dẫn binary `fake-agent`.
    #[must_use]
    pub fn new(fake_agent_bin: impl Into<PathBuf>) -> Self {
        Self {
            fake_agent_bin: fake_agent_bin.into(),
        }
    }
}

impl AgentAdapter for FakeAdapter {
    fn name(&self) -> &'static str {
        "fake-agent"
    }

    fn start_run(
        &self,
        spec: AgentRunSpec,
    ) -> BoxFuture<'_, Result<Receiver<AgentEvent>, AgentPortError>> {
        Box::pin(async move {
            let mut proc_spec = ProcessSpec::new(&self.fake_agent_bin, &spec.working_dir);

            proc_spec = proc_spec.arg("--scenario").arg(&spec.prompt);
            proc_spec.custom_env = spec.env;

            proc_spec.env_whitelist = vec![
                "PATH".to_string(),
                "SystemRoot".to_string(),
                "TEMP".to_string(),
                "TMP".to_string(),
            ];

            if let Some(home) = spec.isolated_home {
                proc_spec = proc_spec.isolated_home(home);
            }
            if let Some(t_timeout) = spec.total_timeout {
                proc_spec = proc_spec.total_timeout(t_timeout);
            }
            if let Some(s_timeout) = spec.silent_timeout {
                proc_spec = proc_spec.silent_timeout(s_timeout);
            }

            let proc_rx = dandori_process::spawn(proc_spec)
                .map_err(|e| AgentPortError::ProcessError(e.to_string()))?;

            let (tx, rx) = channel::<AgentEvent>(FORWARD_BUFFER);
            tokio::spawn(forward_events(proc_rx, tx));

            Ok(rx)
        })
    }
}

async fn forward_events(mut proc_rx: Receiver<ProcessEvent>, tx: Sender<AgentEvent>) {
    while let Some(ev) = crate::next_event(&mut proc_rx, &tx).await {
        match ev {
            ProcessEvent::StdoutLine(line) => {
                let mapped = parse_line_event(&line);
                if tx.send(mapped).await.is_err() {
                    break;
                }
            }
            ProcessEvent::StderrLine(line) => {
                let err_event = AgentEvent::Error(line);
                if tx.send(err_event).await.is_err() {
                    break;
                }
            }
            ProcessEvent::Exit(code) => {
                let _ = tx.send(AgentEvent::Finished { exit_code: code }).await;
                break;
            }
        }
    }
}

fn parse_line_event(line: &str) -> AgentEvent {
    if let Ok(val) = serde_json::from_str::<Value>(line) {
        if val.get("type").and_then(Value::as_str) == Some("quota_exhausted") {
            let msg = val
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Hết hạn mức")
                .to_string();
            return AgentEvent::QuotaExhausted { message: msg };
        }
        if val.get("type").and_then(Value::as_str) == Some("tool_call") {
            let id = val
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let name = val
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let arguments = val
                .get("arguments")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            return AgentEvent::ToolCall {
                id,
                name,
                arguments,
            };
        }
        if val.get("type").and_then(Value::as_str) == Some("error") {
            let msg = val
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Lỗi thực thi")
                .to_string();
            return AgentEvent::Error(msg);
        }
    }

    if let Some(err_msg) = line.strip_prefix("ERROR:") {
        return AgentEvent::Error(err_msg.trim().to_string());
    }

    AgentEvent::Text(line.to_string())
}

//! Adapter tương tác với Claude Code CLI.

use dandori_agent_port::{AgentAdapter, AgentEvent, AgentPortError, AgentRunSpec, BoxFuture};
use dandori_process::{ProcessEvent, ProcessSpec};
use serde_json::Value;
use std::path::PathBuf;
use tokio::sync::mpsc::{Receiver, Sender, channel};

const FORWARD_BUFFER: usize = 64;

/// Adapter cho Claude Code CLI.
#[derive(Debug, Clone)]
pub struct ClaudeAdapter {
    program: PathBuf,
    quota_patterns: Vec<String>,
}

impl ClaudeAdapter {
    /// Khởi tạo adapter Claude với đường dẫn chương trình.
    #[must_use]
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            // Mặc định rỗng: chưa có thông điệp hết hạn mức thật của CLI để dựa vào; cấu hình qua
            // `with_quota_patterns`. (Mẫu đoán như "429" từng khớp nhầm mọi dòng chứa số 429.)
            quota_patterns: Vec::new(),
        }
    }

    /// Tùy biến danh sách các mẫu phát hiện hết hạn mức.
    #[must_use]
    pub fn with_quota_patterns(mut self, patterns: Vec<String>) -> Self {
        self.quota_patterns = patterns;
        self
    }

    /// Lấy danh sách tham số dòng lệnh khớp chính xác đặc tả khảo sát.
    #[must_use]
    pub fn build_args(prompt: &str) -> Vec<String> {
        vec![
            "-p".to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            // CLI thật (2.1.300) từ chối `-p --output-format stream-json` nếu thiếu --verbose.
            "--verbose".to_string(),
            "--dangerously-skip-permissions".to_string(),
            // `--` để lời nhắc bắt đầu bằng "-" (ví dụ danh sách markdown) không bị hiểu là tùy chọn.
            "--".to_string(),
            prompt.to_string(),
        ]
    }
}

impl AgentAdapter for ClaudeAdapter {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn start_run(
        &self,
        spec: AgentRunSpec,
    ) -> BoxFuture<'_, Result<Receiver<AgentEvent>, AgentPortError>> {
        Box::pin(async move {
            let mut proc_spec = ProcessSpec::new(&self.program, &spec.working_dir);
            proc_spec.args = Self::build_args(&spec.prompt);
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
            let quota_patterns = self.quota_patterns.clone();
            tokio::spawn(forward_claude_events(proc_rx, tx, quota_patterns));

            Ok(rx)
        })
    }
}

async fn forward_claude_events(
    mut proc_rx: Receiver<ProcessEvent>,
    tx: Sender<AgentEvent>,
    quota_patterns: Vec<String>,
) {
    while let Some(ev) = crate::next_event(&mut proc_rx, &tx).await {
        match ev {
            ProcessEvent::StdoutLine(line) => {
                if quota_patterns.iter().any(|p| line.contains(p)) {
                    let _ = tx
                        .send(AgentEvent::QuotaExhausted {
                            message: line.clone(),
                        })
                        .await;
                }
                let mut closed = false;
                for mapped in parse_claude_stdout_line(&line) {
                    if tx.send(mapped).await.is_err() {
                        closed = true;
                        break;
                    }
                }
                if closed {
                    break;
                }
            }
            ProcessEvent::StderrLine(line) => {
                if quota_patterns.iter().any(|p| line.contains(p)) {
                    let _ = tx
                        .send(AgentEvent::QuotaExhausted {
                            message: line.clone(),
                        })
                        .await;
                }
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

/// Số ký tự tối đa giữ lại của một kết quả công cụ (đầu ra có thể rất dài).
const TOOL_RESULT_PREVIEW_CHARS: usize = 300;

/// Chuyển một dòng `stream-json` của `claude -p` thành các sự kiện của Dandori.
///
/// Định dạng đã đối chiếu với đầu ra thật (Claude Code 2.1.300): mỗi dòng là một JSON có `type`:
/// `system` (khởi tạo, danh sách lệnh), `assistant` (khối `text` và `tool_use` trong
/// `message.content`), `user` (khối `tool_result`), `rate_limit_event`, `result` (kết thúc, có
/// `is_error` và `result`). Dòng không phải JSON hoặc `type` lạ được giữ nguyên dạng văn bản.
fn parse_claude_stdout_line(line: &str) -> Vec<AgentEvent> {
    let Ok(val) = serde_json::from_str::<Value>(line) else {
        return vec![AgentEvent::Text(line.to_string())];
    };
    match val.get("type").and_then(Value::as_str) {
        Some("assistant") => assistant_events(&val),
        Some("user") => tool_result_events(&val),
        Some("result") => {
            if val.get("is_error").and_then(Value::as_bool) == Some(true) {
                let msg = val
                    .get("result")
                    .and_then(Value::as_str)
                    .unwrap_or("Claude báo lỗi khi kết thúc");
                vec![AgentEvent::Error(msg.to_string())]
            } else {
                // Nội dung kết quả đã được phát qua khối `text` của `assistant`.
                Vec::new()
            }
        }
        Some("system" | "rate_limit_event") => Vec::new(),
        _ => vec![legacy_event(&val, line)],
    }
}

fn assistant_events(val: &Value) -> Vec<AgentEvent> {
    let blocks = val
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(Value::as_array);
    let Some(blocks) = blocks else {
        return Vec::new();
    };
    blocks
        .iter()
        .filter_map(|block| match block.get("type").and_then(Value::as_str) {
            Some("text") => block
                .get("text")
                .and_then(Value::as_str)
                .map(|t| AgentEvent::Text(t.to_string())),
            Some("tool_use") => Some(AgentEvent::ToolCall {
                id: block
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                name: block
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                arguments: block
                    .get("input")
                    .map_or_else(|| "{}".to_string(), Value::to_string),
            }),
            _ => None,
        })
        .collect()
}

fn tool_result_events(val: &Value) -> Vec<AgentEvent> {
    let blocks = val
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(Value::as_array);
    let Some(blocks) = blocks else {
        return Vec::new();
    };
    blocks
        .iter()
        .filter(|b| b.get("type").and_then(Value::as_str) == Some("tool_result"))
        .map(|b| {
            let content = match b.get("content") {
                Some(Value::String(s)) => s.clone(),
                Some(other) => other.to_string(),
                None => String::new(),
            };
            let preview: String = content.chars().take(TOOL_RESULT_PREVIEW_CHARS).collect();
            AgentEvent::Text(format!("[kết quả công cụ] {preview}"))
        })
        .collect()
}

/// Các dạng đơn giản cũ (`text`, `delta.text`, `content`, `tool_use` ở cấp ngoài) và dự phòng.
fn legacy_event(val: &Value, line: &str) -> AgentEvent {
    if let Some(delta_text) = val
        .get("delta")
        .and_then(|d| d.get("text"))
        .and_then(Value::as_str)
    {
        return AgentEvent::Text(delta_text.to_string());
    }
    if let Some(text) = val.get("text").and_then(Value::as_str) {
        return AgentEvent::Text(text.to_string());
    }
    if let Some(content) = val.get("content").and_then(Value::as_str) {
        return AgentEvent::Text(content.to_string());
    }
    AgentEvent::Text(line.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_real_assistant_text_and_tool_use_blocks() {
        let text_line = r#"{"type":"assistant","message":{"model":"m","content":[{"type":"text","text":"Đã tạo HELLO.txt"}]},"session_id":"s"}"#;
        let events = parse_claude_stdout_line(text_line);
        assert!(matches!(&events[..], [AgentEvent::Text(t)] if t == "Đã tạo HELLO.txt"));

        let tool_line = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"toolu_1","name":"Write","input":{"file_path":"a.txt","content":"x"}}]}}"#;
        let events = parse_claude_stdout_line(tool_line);
        match &events[..] {
            [
                AgentEvent::ToolCall {
                    id,
                    name,
                    arguments,
                },
            ] => {
                assert_eq!(id, "toolu_1");
                assert_eq!(name, "Write");
                assert!(arguments.contains("a.txt"));
            }
            other => panic!("kỳ vọng một ToolCall, nhận {other:?}"),
        }
    }

    #[test]
    fn test_one_message_with_text_and_tool_yields_both() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Tôi sẽ ghi tệp"},{"type":"tool_use","id":"t","name":"Write","input":{}}]}}"#;
        let events = parse_claude_stdout_line(line);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_noise_events_are_dropped() {
        for line in [
            r#"{"type":"system","subtype":"init","cwd":"x","tools":["Bash"]}"#,
            r#"{"type":"system","subtype":"commands_changed","commands":[]}"#,
            r#"{"type":"rate_limit_event","rate_limit_info":{"status":"allowed"}}"#,
            r#"{"type":"result","subtype":"success","is_error":false,"result":"xong"}"#,
        ] {
            assert!(parse_claude_stdout_line(line).is_empty(), "{line}");
        }
    }

    #[test]
    fn test_error_result_becomes_error_event() {
        let line = r#"{"type":"result","subtype":"error_during_execution","is_error":true,"result":"Hết phiên"}"#;
        assert!(matches!(
            &parse_claude_stdout_line(line)[..],
            [AgentEvent::Error(m)] if m == "Hết phiên"
        ));
    }

    #[test]
    fn test_tool_result_is_truncated_preview() {
        let long = "x".repeat(1000);
        let line = format!(
            r#"{{"type":"user","message":{{"content":[{{"type":"tool_result","tool_use_id":"t","content":"{long}"}}]}}}}"#
        );
        match &parse_claude_stdout_line(&line)[..] {
            [AgentEvent::Text(t)] => {
                assert!(t.starts_with("[kết quả công cụ] "));
                assert!(t.chars().count() < 400);
            }
            other => panic!("kỳ vọng một Text, nhận {other:?}"),
        }
    }

    /// Dòng thật ghi lại từ một lần chạy `claude -p --output-format stream-json --verbose`
    /// (Claude Code 2.1.300, đã bỏ trường dài và đường dẫn riêng).
    #[test]
    fn test_real_recorded_run_yields_tool_call_and_text_without_noise() {
        let raw = include_str!("../tests/fixtures/claude_stream_real.jsonl");
        let events: Vec<AgentEvent> = raw.lines().flat_map(parse_claude_stdout_line).collect();

        let tool_calls: Vec<&AgentEvent> = events
            .iter()
            .filter(|e| matches!(e, AgentEvent::ToolCall { name, .. } if name == "Write"))
            .collect();
        assert_eq!(
            tool_calls.len(),
            1,
            "phải có đúng một lệnh Write: {events:?}"
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, AgentEvent::Text(t) if t.contains("HELLO.txt"))),
            "phải có lời Claude nhắc tới HELLO.txt"
        );
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, AgentEvent::Text(t) if t.starts_with('{'))),
            "không được để JSON thô lọt ra thành văn bản: {events:?}"
        );
    }

    #[test]
    fn test_plain_and_legacy_lines_are_kept() {
        assert!(matches!(
            &parse_claude_stdout_line("dòng thường")[..],
            [AgentEvent::Text(t)] if t == "dòng thường"
        ));
        assert!(matches!(
            &parse_claude_stdout_line(r#"{"text":"cũ"}"#)[..],
            [AgentEvent::Text(t)] if t == "cũ"
        ));
    }
}

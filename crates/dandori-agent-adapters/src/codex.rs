//! Adapter tương tác với OpenAI Codex CLI (khung mẫu tương thích).

use dandori_agent_port::{AgentAdapter, AgentEvent, AgentPortError, AgentRunSpec, BoxFuture};
use dandori_process::{ProcessEvent, ProcessSpec};
use std::path::PathBuf;
use tokio::sync::mpsc::{Receiver, Sender, channel};

const FORWARD_BUFFER: usize = 64;

/// Adapter khung cho Codex CLI.
#[derive(Debug, Clone)]
pub struct CodexAdapter {
    program: PathBuf,
    quota_patterns: Vec<String>,
}

impl CodexAdapter {
    /// Tham số dòng lệnh đã được đối chiếu với Codex CLI thật hay chưa (hiện: chưa).
    pub const ARGS_VERIFIED: bool = false;

    /// Khởi tạo adapter Codex với đường dẫn chương trình.
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
            "--print".to_string(),
            "--output-format".to_string(),
            "json".to_string(),
            prompt.to_string(),
        ]
    }
}

impl AgentAdapter for CodexAdapter {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn start_run(
        &self,
        spec: AgentRunSpec,
    ) -> BoxFuture<'_, Result<Receiver<AgentEvent>, AgentPortError>> {
        Box::pin(async move {
            // CHƯA KIỂM CHỨNG: Codex CLI chưa được cài trên máy kiểm nên `build_args` bên dưới là
            // phỏng đoán theo khuôn của Claude/Agy, không phải tham số thật của Codex. Từ chối chạy
            // thay vì gọi CLI thật bằng tham số bịa.
            if !Self::ARGS_VERIFIED {
                return Err(AgentPortError::ProcessError(
                    "Adapter Codex chưa được kiểm chứng với Codex CLI thật nên bị tắt".to_string(),
                ));
            }
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
            tokio::spawn(forward_codex_events(proc_rx, tx, quota_patterns));

            Ok(rx)
        })
    }
}

async fn forward_codex_events(
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
                if tx.send(AgentEvent::Text(line)).await.is_err() {
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

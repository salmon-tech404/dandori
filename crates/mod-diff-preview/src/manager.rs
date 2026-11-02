//! Trình quản lý các phiên xem trước ứng dụng của task.

use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

use crate::model::PreviewStatus;
use dandori_module_api::{ProcessEvent, ProcessSpec, spawn};

const MAX_LOG_LINES: usize = 500;

#[derive(Clone)]
struct ActivePreview {
    port: u16,
    host: String,
    command: Vec<String>,
    logs: Arc<Mutex<VecDeque<String>>>,
    token: CancellationToken,
}

/// Trình quản lý tiến trình xem trước cục bộ.
#[derive(Debug, Clone, Default)]
pub struct PreviewManager {
    processes: Arc<RwLock<HashMap<String, ActivePreview>>>,
}

impl std::fmt::Debug for ActivePreview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActivePreview")
            .field("port", &self.port)
            .field("host", &self.host)
            .field("command", &self.command)
            .finish()
    }
}

impl PreviewManager {
    /// Khởi tạo `PreviewManager` mới.
    #[must_use]
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Khởi chạy tiến trình xem trước cho task trong thư mục worktree.
    ///
    /// # Errors
    /// Trả về lỗi nếu lệnh rỗng, tiến trình đang chạy hoặc không thể spawn.
    pub async fn start_preview(
        &self,
        task_id: &str,
        workdir: &Path,
        command: Vec<String>,
        port: u16,
    ) -> Result<PreviewStatus, &'static str> {
        let program = command.first().ok_or("Lệnh xem trước rỗng")?;
        {
            let lock = self.processes.read().await;
            if lock.contains_key(task_id) {
                return Err("Đang có bản xem trước chạy cho task này");
            }
        }

        let mut spec = ProcessSpec::new(program, workdir);
        if command.len() > 1 {
            spec.args = command.get(1..).unwrap_or(&[]).to_vec();
        }

        let rx = spawn(spec).map_err(|_| "Không thể khởi chạy tiến trình xem trước")?;
        let token = CancellationToken::new();
        let logs = Arc::new(Mutex::new(VecDeque::with_capacity(MAX_LOG_LINES)));

        let active = ActivePreview {
            port,
            host: "127.0.0.1".to_string(),
            command: command.clone(),
            logs: Arc::clone(&logs),
            token: token.clone(),
        };

        {
            let mut lock = self.processes.write().await;
            lock.insert(task_id.to_string(), active);
        }

        self.spawn_log_collector(task_id.to_string(), rx, logs, token);

        let url = format!("http://127.0.0.1:{port}");
        Ok(PreviewStatus {
            running: true,
            pid: None,
            port: Some(port),
            host: "127.0.0.1".to_string(),
            url: Some(url),
            command,
            logs: Vec::new(),
        })
    }

    fn spawn_log_collector(
        &self,
        task_id: String,
        mut rx: tokio::sync::mpsc::Receiver<ProcessEvent>,
        logs: Arc<Mutex<VecDeque<String>>>,
        token: CancellationToken,
    ) {
        let processes = Arc::clone(&self.processes);
        tokio::spawn(async move {
            tokio::select! {
                () = token.cancelled() => {}
                () = async {
                    while let Some(event) = rx.recv().await {
                        let line = match event {
                            ProcessEvent::StdoutLine(l) | ProcessEvent::StderrLine(l) => l,
                            ProcessEvent::Exit(_) => break,
                        };
                        let mut lock = logs.lock().await;
                        if lock.len() >= MAX_LOG_LINES {
                            lock.pop_front();
                        }
                        lock.push_back(line);
                    }
                } => {}
            }
            let mut lock = processes.write().await;
            lock.remove(&task_id);
        });
    }

    /// Lấy trạng thái hiện tại của bản xem trước cho task.
    pub async fn get_status(&self, task_id: &str) -> PreviewStatus {
        let lock = self.processes.read().await;
        if let Some(active) = lock.get(task_id) {
            let logs_guard = active.logs.lock().await;
            let logs: Vec<String> = logs_guard.iter().cloned().collect();
            let url = format!("http://127.0.0.1:{}", active.port);
            PreviewStatus {
                running: true,
                pid: None,
                port: Some(active.port),
                host: active.host.clone(),
                url: Some(url),
                command: active.command.clone(),
                logs,
            }
        } else {
            PreviewStatus {
                running: false,
                pid: None,
                port: None,
                host: "127.0.0.1".to_string(),
                url: None,
                command: Vec::new(),
                logs: Vec::new(),
            }
        }
    }

    /// Dừng phiên xem trước của task.
    pub async fn stop_preview(&self, task_id: &str) -> bool {
        let mut lock = self.processes.write().await;
        if let Some(active) = lock.remove(task_id) {
            active.token.cancel();
            true
        } else {
            false
        }
    }

    /// Dừng toàn bộ các phiên xem trước đang chạy (khi tắt module).
    pub async fn stop_all(&self) {
        let mut lock = self.processes.write().await;
        for (_, active) in lock.drain() {
            active.token.cancel();
        }
    }
}

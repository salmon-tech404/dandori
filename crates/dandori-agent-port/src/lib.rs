//! Cổng giao tiếp agent (trait AgentAdapter và kiểu liên quan).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;

/// Kiểu Future trả về cho các hàm bất đồng bộ của adapter.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Thông tin tham chiếu tài khoản agent kết nối.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentAccountRef {
    /// Định danh tài khoản.
    pub id: String,
    /// Nhà cung cấp (Claude, Gemini, Codex, v.v.).
    pub provider: String,
    /// Nhãn hiển thị của tài khoản.
    pub label: String,
}

/// Vai trò của agent trong task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentRole {
    /// Agent điều phối / quản lý dự án (PM).
    Lead,
    /// Agent thực thi công việc cụ thể.
    Worker,
    /// Vai trò tùy chỉnh khác.
    Custom(String),
}

/// Sự kiện chuẩn hóa phát ra từ luồng chạy của agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentEvent {
    /// Đoạn văn bản đầu ra từ mô hình.
    Text(String),
    /// Yêu cầu gọi công cụ.
    ToolCall {
        /// ID lệnh gọi công cụ.
        id: String,
        /// Tên công cụ.
        name: String,
        /// Chuỗi tham số JSON.
        arguments: String,
    },
    /// Cảnh báo hết hạn mức tài khoản.
    QuotaExhausted {
        /// Chi tiết thông điệp hạn mức.
        message: String,
    },
    /// Phiên chạy kết thúc với mã thoát.
    Finished {
        /// Mã thoát tiến trình con.
        exit_code: i32,
    },
    /// Lỗi phát sinh trong quá trình chạy.
    Error(String),
}

/// Cấu hình yêu cầu chạy một tác vụ agent.
#[derive(Debug, Clone)]
pub struct AgentRunSpec {
    /// Thư mục làm việc cho tiến trình.
    pub working_dir: PathBuf,
    /// Chỉ thị / prompt gửi tới agent.
    pub prompt: String,
    /// Tài khoản chỉ định (nếu có).
    pub account: Option<AgentAccountRef>,
    /// Vai trò của agent.
    pub role: AgentRole,
    /// Các biến môi trường bổ sung.
    pub env: HashMap<String, String>,
    /// Thư mục HOME cô lập cho tài khoản.
    pub isolated_home: Option<PathBuf>,
    /// Thời gian chờ tối đa cho toàn bộ phiên chạy.
    pub total_timeout: Option<Duration>,
    /// Thời gian chờ im lặng tối đa (không có output).
    pub silent_timeout: Option<Duration>,
}

impl AgentRunSpec {
    /// Khởi tạo cấu hình chạy agent mới với các tham số cơ bản.
    #[must_use]
    pub fn new(working_dir: PathBuf, prompt: impl Into<String>, role: AgentRole) -> Self {
        Self {
            working_dir,
            prompt: prompt.into(),
            account: None,
            role,
            env: HashMap::new(),
            isolated_home: None,
            total_timeout: None,
            silent_timeout: None,
        }
    }
}

/// Lỗi của cổng giao tiếp agent.
#[derive(Debug, thiserror::Error)]
pub enum AgentPortError {
    /// Lỗi tiến trình con bên dưới.
    #[error("lỗi tiến trình con: {0}")]
    ProcessError(String),
    /// Hết hạn mức.
    #[error("hết hạn mức tài khoản: {0}")]
    QuotaExhausted(String),
    /// Lỗi thực thi.
    #[error("lỗi thực thi: {0}")]
    ExecutionError(String),
    /// Tiến trình bị hủy chủ động.
    #[error("tiến trình bị hủy")]
    Cancelled,
    /// Quá thời gian chờ.
    #[error("quá thời gian chờ: {0}")]
    Timeout(String),
}

/// Trait cổng giao tiếp chuẩn cho các adapter agent (Claude, Gemini, Codex, fake-agent).
pub trait AgentAdapter: Send + Sync {
    /// Tên định danh của adapter.
    fn name(&self) -> &'static str;

    /// Khởi động một phiên chạy agent và trả về luồng nhận sự kiện chuẩn hóa.
    fn start_run(
        &self,
        spec: AgentRunSpec,
    ) -> BoxFuture<'_, Result<Receiver<AgentEvent>, AgentPortError>>;
}

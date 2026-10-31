//! Cấu hình thông số khởi chạy tiến trình con an toàn.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

/// Thông số chi tiết để khởi chạy một tiến trình con an toàn.
#[derive(Debug, Clone)]
pub struct ProcessSpec {
    /// Đường dẫn hoặc tên chương trình thực thi.
    pub program: PathBuf,
    /// Danh sách các tham số dòng lệnh (truyền dạng mảng, cấm nối chuỗi shell).
    pub args: Vec<String>,
    /// Thư mục làm việc của tiến trình.
    pub working_dir: PathBuf,
    /// Danh sách các tên biến môi trường cha được phép truyền xuống (whitelist).
    pub env_whitelist: Vec<String>,
    /// Các biến môi trường bổ sung tường minh.
    pub custom_env: HashMap<String, String>,
    /// Thư mục HOME/USERPROFILE riêng biệt cho tài khoản agent.
    pub isolated_home: Option<PathBuf>,
    /// Thời hạn chạy tối đa trước khi bị tiêu diệt toàn bộ cây tiến trình.
    pub total_timeout: Option<Duration>,
    /// Thời hạn tối đa không phát sinh đầu ra (stdout/stderr) trước khi bị tiêu diệt.
    pub silent_timeout: Option<Duration>,
    /// Giới hạn số dòng tối đa cho phép đọc từ tiến trình.
    pub line_limit: Option<usize>,
    /// Giới hạn tổng số byte tối đa cho phép đọc từ tiến trình.
    pub byte_limit: Option<usize>,
    /// Token hủy chủ động tiến trình từ bên ngoài.
    pub cancellation_token: Option<CancellationToken>,
}

impl ProcessSpec {
    /// Tạo mới một cấu hình tiến trình với chương trình và thư mục làm việc mặc định.
    #[must_use]
    pub fn new(program: impl Into<PathBuf>, working_dir: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            working_dir: working_dir.into(),
            env_whitelist: Vec::new(),
            custom_env: HashMap::new(),
            isolated_home: None,
            total_timeout: None,
            silent_timeout: None,
            line_limit: None,
            byte_limit: None,
            cancellation_token: None,
        }
    }

    /// Thêm một tham số dòng lệnh.
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Thêm danh sách tham số dòng lệnh.
    #[must_use]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for arg in args {
            self.args.push(arg.into());
        }
        self
    }

    /// Thiết lập danh sách whitelist các biến môi trường được kế thừa.
    #[must_use]
    pub fn env_whitelist<I, S>(mut self, vars: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.env_whitelist = vars.into_iter().map(Into::into).collect();
        self
    }

    /// Thiết lập một biến môi trường cụ thể.
    #[must_use]
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_env.insert(key.into(), value.into());
        self
    }

    /// Thiết lập thư mục HOME cô lập cho tiến trình.
    #[must_use]
    pub fn isolated_home(mut self, home: impl Into<PathBuf>) -> Self {
        self.isolated_home = Some(home.into());
        self
    }

    /// Thiết lập thời gian chạy tối đa.
    #[must_use]
    pub fn total_timeout(mut self, duration: Duration) -> Self {
        self.total_timeout = Some(duration);
        self
    }

    /// Thiết lập thời gian im lặng tối đa.
    #[must_use]
    pub fn silent_timeout(mut self, duration: Duration) -> Self {
        self.silent_timeout = Some(duration);
        self
    }

    /// Thiết lập giới hạn số dòng.
    #[must_use]
    pub fn line_limit(mut self, limit: usize) -> Self {
        self.line_limit = Some(limit);
        self
    }

    /// Thiết lập giới hạn tổng dung lượng byte.
    #[must_use]
    pub fn byte_limit(mut self, limit: usize) -> Self {
        self.byte_limit = Some(limit);
        self
    }

    /// Thiết lập token hủy tiến trình.
    #[must_use]
    pub fn cancellation_token(mut self, token: CancellationToken) -> Self {
        self.cancellation_token = Some(token);
        self
    }
}

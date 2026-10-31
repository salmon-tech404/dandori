//! Định nghĩa các loại lỗi khi thực thi tiến trình con.

use std::time::Duration;

/// Lỗi phát sinh trong quá trình chuẩn bị hoặc thực thi tiến trình con.
#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    /// Lỗi I/O từ hệ điều hành.
    #[error("lỗi I/O hệ thống: {0}")]
    Io(#[from] std::io::Error),

    /// Tiến trình vượt quá thời gian chạy tổng.
    #[error("tiến trình chạy quá thời gian tối đa ({0:?})")]
    TotalTimeout(Duration),

    /// Tiến trình im lặng không có bất kỳ đầu ra nào quá thời gian cho phép.
    #[error("tiến trình im lặng quá thời gian tối đa ({0:?})")]
    SilentTimeout(Duration),

    /// Vượt quá giới hạn số dòng hoặc dung lượng bộ đệm.
    #[error("tiến trình vượt quá giới hạn tài nguyên: {0}")]
    LimitExceeded(String),

    /// Tiến trình bị hủy bởi token hủy.
    #[error("tiến trình bị hủy theo yêu cầu")]
    Cancelled,

    /// Không thể lấy PID của tiến trình con.
    #[error("không thể xác định PID của tiến trình con")]
    MissingPid,
}

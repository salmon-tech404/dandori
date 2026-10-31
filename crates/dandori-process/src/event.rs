//! Định nghĩa sự kiện sinh ra từ tiến trình con.

/// Sự kiện từ tiến trình con truyền về qua kênh nhận bất đồng bộ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessEvent {
    /// Một dòng dữ liệu đọc được từ stdout.
    StdoutLine(String),
    /// Một dòng dữ liệu đọc được từ stderr.
    StderrLine(String),
    /// Tiến trình đã kết thúc với mã thoát tương ứng.
    Exit(i32),
}

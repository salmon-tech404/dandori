//! Nguồn cấp thời gian trừu tượng cho kiểm thử và vận hành (RS-TEST-004).

use chrono::{DateTime, Utc};
use std::fmt::Debug;

/// Trait cung cấp thời gian thực tế hoặc giả lập cho hệ thống Dandori.
pub trait TimeSource: Debug + Send + Sync + 'static {
    /// Lấy thời điểm hiện tại dưới định dạng UTC.
    fn now(&self) -> DateTime<Utc>;
}

/// Nguồn thời gian hệ thống thật sử dụng `Utc::now()`.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemTimeSource;

impl TimeSource for SystemTimeSource {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

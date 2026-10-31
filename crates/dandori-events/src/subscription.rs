//! Tay cầm đăng ký lắng nghe sự kiện của một subscriber.

use dandori_types::Envelope;
use tokio::sync::mpsc::Receiver;

use crate::filter::EventFilter;

/// Tay cầm đăng ký nhận sự kiện từ EventBus. Khi drop tay cầm, subscriber tự động hủy đăng ký.
#[derive(Debug)]
pub struct EventSubscription {
    receiver: Receiver<Envelope>,
    filter: EventFilter,
}

impl EventSubscription {
    /// Tạo mới một `EventSubscription`.
    #[must_use]
    pub fn new(receiver: Receiver<Envelope>, filter: EventFilter) -> Self {
        Self { receiver, filter }
    }

    /// Lấy bộ lọc của subscriber này.
    #[must_use]
    pub fn filter(&self) -> &EventFilter {
        &self.filter
    }

    /// Đọc sự kiện tiếp theo từ hàng đợi. Trả về `None` khi kênh đã đóng.
    pub async fn recv(&mut self) -> Option<Envelope> {
        self.receiver.recv().await
    }
}

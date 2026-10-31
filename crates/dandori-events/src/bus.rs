//! Bus sự kiện trong tiến trình của Dandori (RS-ASYNC-004, 01_KIEN_TRUC.md mục 7).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use dandori_types::{Envelope, EventPayload, EventScope};
use tokio::sync::mpsc::{self, Sender, error::TrySendError};

use crate::filter::EventFilter;
use crate::subscription::EventSubscription;
use crate::time::{SystemTimeSource, TimeSource};

#[derive(Debug, Clone)]
struct SubscriberEntry {
    sender: Sender<Envelope>,
    filter: EventFilter,
}

/// Bus sự kiện trong tiến trình (in-process event bus).
/// Người nghe chậm không bao giờ làm nghẽn người phát; hàng đợi tràn sẽ đếm số sự kiện bị mất.
#[derive(Debug, Clone)]
pub struct EventBus {
    subscribers: Arc<RwLock<Vec<SubscriberEntry>>>,
    capacity_per_subscriber: usize,
    time_source: Arc<dyn TimeSource>,
    dropped_events: Arc<AtomicU64>,
    published_events: Arc<AtomicU64>,
}

impl EventBus {
    /// Khởi tạo một `EventBus` mới với dung lượng hàng đợi cho mỗi người nghe.
    #[must_use]
    pub fn new(capacity_per_subscriber: usize) -> Self {
        Self::with_time_source(capacity_per_subscriber, Arc::new(SystemTimeSource))
    }

    /// Khởi tạo `EventBus` với nguồn cấp thời gian trừu tượng (dùng cho test RS-TEST-004).
    #[must_use]
    pub fn with_time_source(
        capacity_per_subscriber: usize,
        time_source: Arc<dyn TimeSource>,
    ) -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(Vec::new())),
            capacity_per_subscriber,
            time_source,
            dropped_events: Arc::new(AtomicU64::new(0)),
            published_events: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Đăng ký nhận sự kiện với một bộ lọc điều kiện.
    /// Trả về tay cầm `EventSubscription`. Khi drop tay cầm, subscriber tự động hủy nhận sự kiện.
    #[must_use]
    pub fn subscribe(&self, filter: EventFilter) -> EventSubscription {
        let (tx, rx) = mpsc::channel(self.capacity_per_subscriber);
        let entry = SubscriberEntry {
            sender: tx,
            filter: filter.clone(),
        };

        if let Ok(mut subs) = self.subscribers.write() {
            subs.push(entry);
        }

        EventSubscription::new(rx, filter)
    }

    /// Tạo một bao thư sự kiện chuẩn hóa dựa trên nguồn thời gian hiện tại của bus.
    #[must_use]
    pub fn create_envelope(&self, scope: EventScope, payload: EventPayload) -> Envelope {
        Envelope::new(self.time_source.now(), scope, payload)
    }

    /// Phát một sự kiện tới toàn bộ subscriber phù hợp với bộ lọc.
    ///
    /// Thao tác phát sự kiện là **không chặn** (non-blocking). Nếu một subscriber có hàng đợi bị đầy,
    /// sự kiện sẽ bị bỏ qua đối với subscriber đó và bộ đếm mất sự kiện sẽ tăng lên.
    pub fn publish(&self, envelope: Envelope) {
        self.published_events.fetch_add(1, Ordering::Relaxed);

        let mut has_closed_subscribers = false;

        if let Ok(subs) = self.subscribers.read() {
            for sub in subs.iter() {
                if sub.filter.matches(&envelope) {
                    match sub.sender.try_send(envelope.clone()) {
                        Ok(()) => {}
                        Err(TrySendError::Full(_)) => {
                            self.dropped_events.fetch_add(1, Ordering::Relaxed);
                            tracing::debug!(
                                "EventBus: Hàng đợi tràn, bỏ qua sự kiện cho subscriber chậm"
                            );
                        }
                        Err(TrySendError::Closed(_)) => {
                            has_closed_subscribers = true;
                        }
                    }
                }
            }
        }

        // Tự động dọn dẹp các subscriber đã drop tay cầm
        if has_closed_subscribers && let Ok(mut subs) = self.subscribers.write() {
            subs.retain(|sub| !sub.sender.is_closed());
        }
    }

    /// Lấy tổng số sự kiện bị mất do người nghe chậm làm tràn hàng đợi.
    #[must_use]
    pub fn dropped_events_count(&self) -> u64 {
        self.dropped_events.load(Ordering::Relaxed)
    }

    /// Lấy tổng số sự kiện đã được phát ra qua bus.
    #[must_use]
    pub fn published_events_count(&self) -> u64 {
        self.published_events.load(Ordering::Relaxed)
    }

    /// Lấy số lượng subscriber hiện đang hoạt động.
    #[must_use]
    pub fn active_subscribers_count(&self) -> usize {
        if let Ok(subs) = self.subscribers.read() {
            subs.iter().filter(|sub| !sub.sender.is_closed()).count()
        } else {
            0
        }
    }
}

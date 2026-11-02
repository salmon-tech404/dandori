//! Quản lý trạng thái kết nối và vùng đệm sự kiện gần đây cho module live-status.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};

use dandori_types::Envelope;

/// Số lượng kết nối tối đa cho mỗi người dùng.
pub const MAX_CONNECTIONS_PER_USER: usize = 5;

/// Dung lượng tối đa của vùng đệm sự kiện gần đây để phục vụ Last-Event-ID.
pub const EVENT_BUFFER_CAPACITY: usize = 200;

/// Guard RAII tự động giảm số lượng kết nối của người dùng khi kết thúc luồng.
#[derive(Debug)]
pub struct ConnectionGuard {
    user_id: String,
    tracker: Arc<RwLock<HashMap<String, usize>>>,
}

impl ConnectionGuard {
    fn new(user_id: String, tracker: Arc<RwLock<HashMap<String, usize>>>) -> Self {
        Self { user_id, tracker }
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        if let Ok(mut lock) = self.tracker.write()
            && let Some(count) = lock.get_mut(&self.user_id)
        {
            if *count > 1 {
                *count -= 1;
            } else {
                lock.remove(&self.user_id);
            }
        }
    }
}

/// Vùng đệm sự kiện gần đây theo cơ chế ring buffer.
#[derive(Debug, Default)]
pub struct RecentEventBuffer {
    buffer: RwLock<VecDeque<Envelope>>,
}

impl RecentEventBuffer {
    /// Khởi tạo buffer mới.
    #[must_use]
    pub fn new() -> Self {
        Self {
            buffer: RwLock::new(VecDeque::with_capacity(EVENT_BUFFER_CAPACITY)),
        }
    }

    /// Ghi nhận một sự kiện vào buffer (loại bỏ sự kiện cũ nhất khi tràn).
    pub fn record(&self, envelope: Envelope) {
        if let Ok(mut lock) = self.buffer.write() {
            if lock.len() >= EVENT_BUFFER_CAPACITY {
                lock.pop_front();
            }
            lock.push_back(envelope);
        }
    }

    /// Trả về số lượng sự kiện hiện tại trong buffer.
    #[must_use]
    pub fn len(&self) -> usize {
        self.buffer.read().map(|l| l.len()).unwrap_or(0)
    }

    /// Kiểm tra buffer có rỗng không.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Lấy danh sách sự kiện phát sinh sau `last_event_id` phù hợp với workspace.
    #[must_use]
    pub fn events_after(
        &self,
        last_event_id: &str,
        scope: &crate::filter::WorkspaceScope,
    ) -> Vec<Envelope> {
        let lock = match self.buffer.read() {
            Ok(l) => l,
            Err(_) => return Vec::new(),
        };

        let mut found = false;
        let mut results = Vec::new();

        for env in lock.iter() {
            if found {
                if crate::filter::matches_scope(env, scope)
                    && crate::filter::is_allowed_kind(&env.kind)
                {
                    results.push(env.clone());
                }
            } else if env.id.to_string() == last_event_id {
                found = true;
            }
        }

        results
    }
}

/// Trạng thái dùng chung của module live-status.
#[derive(Debug, Clone)]
pub struct LiveStatusState {
    connections: Arc<RwLock<HashMap<String, usize>>>,
    recent_events: Arc<RecentEventBuffer>,
}

impl Default for LiveStatusState {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveStatusState {
    /// Khởi tạo trạng thái mới.
    #[must_use]
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            recent_events: Arc::new(RecentEventBuffer::new()),
        }
    }

    /// Vùng đệm sự kiện gần đây.
    #[must_use]
    pub fn recent_events(&self) -> &Arc<RecentEventBuffer> {
        &self.recent_events
    }

    /// Đếm tổng số kết nối đang hoạt động trên toàn hệ thống.
    #[must_use]
    pub fn total_connections(&self) -> usize {
        self.connections
            .read()
            .map(|l| l.values().sum())
            .unwrap_or(0)
    }

    /// Cố gắng cấp phát một kết nối cho người dùng.
    ///
    /// # Errors
    /// Trả về lỗi nếu người dùng đã đạt tới giới hạn tối đa kết nối đồng thời.
    pub fn acquire_connection(&self, user_id: &str) -> Result<ConnectionGuard, &'static str> {
        let mut lock = self.connections.write().map_err(|_| "Lỗi khóa kết nối")?;
        let count = lock.entry(user_id.to_string()).or_insert(0);
        if *count >= MAX_CONNECTIONS_PER_USER {
            return Err("Đã vượt quá số lượng kết nối tối đa cho mỗi người dùng");
        }
        *count += 1;
        Ok(ConnectionGuard::new(
            user_id.to_string(),
            self.connections.clone(),
        ))
    }
}

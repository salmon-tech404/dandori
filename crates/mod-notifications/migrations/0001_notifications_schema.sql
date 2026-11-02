-- Migration tạo các bảng lưu cấu hình kênh thông báo và lịch sử gửi (P3-12)

CREATE TABLE IF NOT EXISTS m_notifications_channel (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    name TEXT NOT NULL,
    channel_type TEXT NOT NULL,
    vault_secret_id TEXT NOT NULL,
    chat_id TEXT,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    notify_on_needs_human INTEGER NOT NULL DEFAULT 1,
    notify_on_failed INTEGER NOT NULL DEFAULT 1,
    notify_on_approval_requested INTEGER NOT NULL DEFAULT 1,
    notify_on_limit_warning INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_m_notifications_channel_ws
    ON m_notifications_channel(workspace_id);

CREATE TABLE IF NOT EXISTS m_notifications_history (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    task_id TEXT,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    error_message TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_m_notifications_history_ws
    ON m_notifications_history(workspace_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_m_notifications_history_dedup
    ON m_notifications_history(channel_id, task_id, event_type, created_at DESC);

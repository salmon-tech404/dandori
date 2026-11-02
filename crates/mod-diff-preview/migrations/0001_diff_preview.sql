-- Bảng lưu lịch sử các phiên xem trước ứng dụng của từng task
CREATE TABLE IF NOT EXISTS m_diff_preview_session (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    command TEXT NOT NULL,
    port INTEGER NOT NULL,
    host TEXT NOT NULL DEFAULT '127.0.0.1',
    status TEXT NOT NULL,
    started_at TEXT NOT NULL,
    stopped_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_diff_preview_task ON m_diff_preview_session (task_id);

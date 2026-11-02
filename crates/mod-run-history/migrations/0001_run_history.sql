-- Migration khởi tạo bảng m_run_history_note (P2-01)
CREATE TABLE IF NOT EXISTS m_run_history_note (
    run_id TEXT PRIMARY KEY NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',
    updated_at TEXT NOT NULL
);

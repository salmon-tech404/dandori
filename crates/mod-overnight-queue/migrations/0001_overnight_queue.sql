-- Bảng cấu hình hàng đợi qua đêm cho từng workspace
CREATE TABLE IF NOT EXISTS m_overnight_queue_config (
    workspace_id TEXT PRIMARY KEY,
    start_time TEXT NOT NULL DEFAULT '23:00',
    timezone TEXT NOT NULL DEFAULT 'UTC',
    max_concurrency INTEGER NOT NULL DEFAULT 1,
    consecutive_failure_limit INTEGER NOT NULL DEFAULT 3,
    status TEXT NOT NULL DEFAULT 'idle', -- 'idle', 'running', 'paused', 'stopped'
    consecutive_failures INTEGER NOT NULL DEFAULT 0,
    last_run_summary TEXT, -- JSON summary of morning review chain
    updated_at TEXT NOT NULL,
    FOREIGN KEY (workspace_id) REFERENCES workspace(id) ON DELETE CASCADE
);

-- Bảng lưu trữ danh sách các task trong hàng đợi
CREATE TABLE IF NOT EXISTS m_overnight_queue_item (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    order_index INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'queued', -- 'queued', 'running', 'completed', 'failed', 'paused'
    run_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(workspace_id, task_id),
    FOREIGN KEY (workspace_id) REFERENCES workspace(id) ON DELETE CASCADE,
    FOREIGN KEY (task_id) REFERENCES task(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_m_overnight_queue_order
ON m_overnight_queue_item(workspace_id, status, order_index);

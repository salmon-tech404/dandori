-- Migration khởi tạo bảng cấu hình và lịch sử tự chuyển tài khoản (P2-10)
CREATE TABLE IF NOT EXISTS m_auto_switch_config (
    project_id TEXT PRIMARY KEY NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    max_switches INTEGER NOT NULL DEFAULT 2,
    allow_cross_provider INTEGER NOT NULL DEFAULT 0,
    allowed_providers TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS m_auto_switch_history (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL REFERENCES task(id) ON DELETE CASCADE,
    from_account_id TEXT NOT NULL,
    to_account_id TEXT NOT NULL,
    from_provider TEXT NOT NULL,
    to_provider TEXT NOT NULL,
    reason TEXT NOT NULL,
    switch_count INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_m_auto_switch_history_task_id ON m_auto_switch_history(task_id);

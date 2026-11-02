-- Bảng cấu hình báo cáo tự động cho không gian làm việc
CREATE TABLE IF NOT EXISTS m_report_config (
    workspace_id TEXT PRIMARY KEY REFERENCES workspace(id) ON DELETE CASCADE,
    auto_generate INTEGER NOT NULL DEFAULT 0,
    timezone TEXT NOT NULL DEFAULT 'UTC',
    daily_time TEXT NOT NULL DEFAULT '23:59',
    weekly_day INTEGER NOT NULL DEFAULT 7,
    weekly_time TEXT NOT NULL DEFAULT '23:59',
    updated_at TEXT NOT NULL
);

-- Bảng lưu trữ bản chụp báo cáo đã tạo (ngày/tuần)
CREATE TABLE IF NOT EXISTS m_report_snapshot (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    period_type TEXT NOT NULL,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    timezone TEXT NOT NULL,
    data_json TEXT NOT NULL,
    markdown TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_m_report_snapshot_ws_period
ON m_report_snapshot (workspace_id, period_type, period_start DESC);

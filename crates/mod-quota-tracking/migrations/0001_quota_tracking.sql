-- Migration khởi tạo bảng cấu hình và báo cáo hạn mức (P2-09)
CREATE TABLE IF NOT EXISTS m_quota_tracking_config (
    account_id TEXT PRIMARY KEY NOT NULL REFERENCES agent_account(id) ON DELETE CASCADE,
    limit_5h INTEGER,
    limit_weekly INTEGER,
    alert_threshold_percent INTEGER NOT NULL DEFAULT 80,
    alerted_above_threshold INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS m_quota_tracking_reported (
    account_id TEXT PRIMARY KEY NOT NULL REFERENCES agent_account(id) ON DELETE CASCADE,
    used_5h INTEGER NOT NULL DEFAULT 0,
    limit_5h INTEGER,
    used_weekly INTEGER NOT NULL DEFAULT 0,
    limit_weekly INTEGER,
    reported_at TEXT NOT NULL
);

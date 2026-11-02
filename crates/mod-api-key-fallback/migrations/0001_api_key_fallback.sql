-- Migration: 0001_api_key_fallback.sql
-- Trần chi tiêu bắt buộc và cấu hình dự phòng API key cho dự án

CREATE TABLE IF NOT EXISTS m_api_key_fallback_spending_limit (
    account_id TEXT PRIMARY KEY NOT NULL REFERENCES agent_account(id) ON DELETE CASCADE,
    period TEXT NOT NULL CHECK (period IN ('daily', 'monthly')),
    limit_type TEXT NOT NULL CHECK (limit_type IN ('tokens', 'currency')),
    limit_value REAL NOT NULL CHECK (limit_value > 0),
    used_value REAL NOT NULL DEFAULT 0.0,
    period_start TEXT NOT NULL,
    alert_threshold REAL NOT NULL DEFAULT 80.0,
    alerted_above_threshold INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS m_api_key_fallback_project_config (
    project_id TEXT PRIMARY KEY NOT NULL,
    fallback_enabled INTEGER NOT NULL DEFAULT 0 CHECK (fallback_enabled IN (0, 1)),
    allowed_accounts TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

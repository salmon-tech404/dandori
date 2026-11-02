-- Schema cho module mobile (P3-13)

CREATE TABLE IF NOT EXISTS m_mobile_device (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL,
    name TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    created_by TEXT NOT NULL,
    read_only BOOLEAN NOT NULL DEFAULT 0,
    is_revoked BOOLEAN NOT NULL DEFAULT 0,
    expires_at TEXT NOT NULL,
    last_used_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_m_mobile_device_ws ON m_mobile_device(workspace_id);
CREATE INDEX IF NOT EXISTS idx_m_mobile_device_token_hash ON m_mobile_device(token_hash);

CREATE TABLE IF NOT EXISTS m_mobile_action_log (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL,
    device_id TEXT NOT NULL,
    action TEXT NOT NULL,
    target_id TEXT,
    detail TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_m_mobile_action_log_ws ON m_mobile_action_log(workspace_id);

CREATE TABLE IF NOT EXISTS m_mobile_config (
    workspace_id TEXT PRIMARY KEY NOT NULL,
    allowed_origins TEXT NOT NULL DEFAULT '[]',
    read_only BOOLEAN NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);

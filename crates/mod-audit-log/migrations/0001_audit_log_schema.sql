-- Bảng m_audit_log: nhật ký kiểm toán bất biến chỉ thêm với chuỗi băm mật mã (P3-14)
CREATE TABLE IF NOT EXISTS m_audit_log (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    sequence INTEGER NOT NULL,
    actor_id TEXT NOT NULL,
    actor_name TEXT NOT NULL,
    action TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    details TEXT NOT NULL,
    source TEXT NOT NULL,
    prev_hash TEXT NOT NULL,
    hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(workspace_id, sequence)
);

CREATE INDEX IF NOT EXISTS idx_m_audit_log_ws_seq ON m_audit_log(workspace_id, sequence);
CREATE INDEX IF NOT EXISTS idx_m_audit_log_ws_act ON m_audit_log(workspace_id, action);
CREATE INDEX IF NOT EXISTS idx_m_audit_log_ws_time ON m_audit_log(workspace_id, created_at);

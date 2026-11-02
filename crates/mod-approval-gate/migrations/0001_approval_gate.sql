-- Migration 0001: Bảng dữ liệu chính sách và yêu cầu phê duyệt chuyển trạng thái nhiệm vụ (P2-03)

CREATE TABLE IF NOT EXISTS m_approval_gate_policy (
    workspace_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    require_in_review_to_merge INTEGER NOT NULL DEFAULT 1,
    disallow_self_approval INTEGER NOT NULL DEFAULT 1,
    expiry_hours INTEGER NOT NULL DEFAULT 24,
    PRIMARY KEY (workspace_id, project_id)
);

CREATE TABLE IF NOT EXISTS m_approval_gate_request (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'transition',
    from_status TEXT NOT NULL,
    target_status TEXT NOT NULL,
    requester_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected', 'expired')),
    reason TEXT,
    decider_id TEXT,
    created_at TEXT NOT NULL,
    decided_at TEXT,
    expires_at TEXT NOT NULL,
    rejection_reason TEXT
);

CREATE INDEX IF NOT EXISTS idx_m_approval_gate_request_workspace
    ON m_approval_gate_request (workspace_id);

CREATE INDEX IF NOT EXISTS idx_m_approval_gate_request_task
    ON m_approval_gate_request (task_id);

CREATE INDEX IF NOT EXISTS idx_m_approval_gate_request_status
    ON m_approval_gate_request (status);

CREATE INDEX IF NOT EXISTS idx_m_approval_gate_request_expires_at
    ON m_approval_gate_request (expires_at);

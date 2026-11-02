-- Migration khởi tạo cho module integrations (P2-06)

CREATE TABLE IF NOT EXISTS m_integration_connection (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    token_encrypted TEXT NOT NULL,
    allow_private_network INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (workspace_id) REFERENCES workspace(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_m_integration_connection_ws ON m_integration_connection(workspace_id);

CREATE TABLE IF NOT EXISTS m_integration_issue_link (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    issue_id TEXT NOT NULL,
    issue_key TEXT NOT NULL,
    issue_title TEXT NOT NULL,
    issue_url TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (workspace_id) REFERENCES workspace(id) ON DELETE CASCADE,
    FOREIGN KEY (task_id) REFERENCES task(id) ON DELETE CASCADE,
    UNIQUE (workspace_id, project_id, issue_id)
);

CREATE INDEX IF NOT EXISTS idx_m_integration_issue_link_task ON m_integration_issue_link(task_id);

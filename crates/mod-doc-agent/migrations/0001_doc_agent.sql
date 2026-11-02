-- Migration cho module doc-agent (P2-05)

CREATE TABLE IF NOT EXISTS m_doc_agent_config (
    project_id TEXT PRIMARY KEY NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    allowed_path_patterns TEXT NOT NULL DEFAULT '["docs/**","README*","CHANGELOG*"]',
    prompt_template TEXT,
    timeout_seconds INTEGER NOT NULL DEFAULT 600,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_m_doc_agent_config_ws ON m_doc_agent_config(workspace_id);

CREATE TABLE IF NOT EXISTS m_doc_agent_run (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL REFERENCES task(id) ON DELETE CASCADE,
    project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    run_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('running', 'completed', 'failed')),
    summary TEXT,
    changed_doc_files TEXT NOT NULL DEFAULT '[]',
    flagged_files TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    finished_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_m_doc_agent_run_task ON m_doc_agent_run(task_id);
CREATE INDEX IF NOT EXISTS idx_m_doc_agent_run_run ON m_doc_agent_run(run_id);

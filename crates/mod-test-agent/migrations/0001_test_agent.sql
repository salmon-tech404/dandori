-- 1. Bảng cấu hình Test Agent theo dự án (P2-04)
CREATE TABLE IF NOT EXISTS m_test_agent_config (
    project_id TEXT PRIMARY KEY NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    workspace_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT 1,
    suggested_command TEXT,
    allowed_path_patterns TEXT NOT NULL DEFAULT '["tests/**","**/*.test.*","**/*_test.*","**/*test*.rs"]',
    max_test_runs_per_worker INTEGER NOT NULL DEFAULT 1,
    timeout_seconds INTEGER NOT NULL DEFAULT 600,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 2. Bảng theo dõi các lần chạy của Test Agent (P2-04)
CREATE TABLE IF NOT EXISTS m_test_agent_run (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL REFERENCES task(id) ON DELETE CASCADE,
    project_id TEXT NOT NULL,
    workspace_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    trigger_run_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'running', 'passed', 'failed', 'cancelled')),
    exit_code INTEGER,
    summary TEXT,
    flagged_files TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    finished_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_m_test_agent_run_task ON m_test_agent_run(task_id);
CREATE INDEX IF NOT EXISTS idx_m_test_agent_run_run_id ON m_test_agent_run(run_id);
CREATE INDEX IF NOT EXISTS idx_m_test_agent_run_trigger ON m_test_agent_run(trigger_run_id);

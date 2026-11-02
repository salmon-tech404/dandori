-- Migration 0001: Lược đồ bảng lưu trữ công việc Markdown (md-jobs)
CREATE TABLE IF NOT EXISTS m_md_job (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    job_name TEXT NOT NULL,
    schedule_cron TEXT NOT NULL,
    agent_role TEXT NOT NULL,
    account_id TEXT,
    prompt TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    approved_hash TEXT,
    status TEXT NOT NULL CHECK (status IN ('pending_approval', 'approved', 'rejected', 'syntax_error')),
    next_fire TEXT,
    last_fire TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(project_id, file_path)
);

CREATE INDEX IF NOT EXISTS idx_md_job_project_id ON m_md_job(project_id);
CREATE INDEX IF NOT EXISTS idx_md_job_status_next_fire ON m_md_job(status, next_fire);

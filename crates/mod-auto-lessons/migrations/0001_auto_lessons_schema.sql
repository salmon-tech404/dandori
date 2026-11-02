-- Migration cho module auto-lessons (P3-11)

CREATE TABLE IF NOT EXISTS m_auto_lessons_config (
    project_id TEXT PRIMARY KEY REFERENCES project(id) ON DELETE CASCADE,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL DEFAULT 1,
    trigger_on_done BOOLEAN NOT NULL DEFAULT 1,
    trigger_on_failed BOOLEAN NOT NULL DEFAULT 1,
    max_token_budget INTEGER NOT NULL DEFAULT 1000,
    min_content_length INTEGER NOT NULL DEFAULT 10,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS m_auto_lessons_run (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    task_id TEXT NOT NULL UNIQUE,
    run_id TEXT,
    status TEXT NOT NULL,
    lesson_id TEXT,
    reason TEXT,
    created_at TEXT NOT NULL,
    finished_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_auto_lessons_run_ws ON m_auto_lessons_run(workspace_id);
CREATE INDEX IF NOT EXISTS idx_auto_lessons_run_proj ON m_auto_lessons_run(project_id);
CREATE INDEX IF NOT EXISTS idx_auto_lessons_run_task ON m_auto_lessons_run(task_id);

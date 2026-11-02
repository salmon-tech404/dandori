-- Migration cho module instruction-sync (P3-09)

CREATE TABLE IF NOT EXISTS m_instruction_sync_config (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    target_filename TEXT NOT NULL,
    strategy TEXT NOT NULL DEFAULT 'skip',
    is_enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(workspace_id, project_id, provider)
);

CREATE INDEX IF NOT EXISTS idx_instruction_sync_ws ON m_instruction_sync_config(workspace_id);
CREATE INDEX IF NOT EXISTS idx_instruction_sync_proj ON m_instruction_sync_config(project_id);

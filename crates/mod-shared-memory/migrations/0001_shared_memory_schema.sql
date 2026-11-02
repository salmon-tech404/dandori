-- Migration cho module shared-memory (P3-10)

CREATE TABLE IF NOT EXISTS m_shared_memory_entry (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    project_id TEXT REFERENCES project(id) ON DELETE CASCADE,
    category TEXT NOT NULL, -- fact, decision, convention, lesson
    content TEXT NOT NULL,
    source_type TEXT NOT NULL, -- human, agent
    source_ref TEXT NOT NULL, -- user:<id>, agent:<role>#run:<id>#task:<id>
    status TEXT NOT NULL DEFAULT 'pending', -- pending, approved, rejected
    reviewed_by TEXT,
    reviewed_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_shared_memory_ws ON m_shared_memory_entry(workspace_id);
CREATE INDEX IF NOT EXISTS idx_shared_memory_proj ON m_shared_memory_entry(project_id);
CREATE INDEX IF NOT EXISTS idx_shared_memory_status ON m_shared_memory_entry(status);
CREATE INDEX IF NOT EXISTS idx_shared_memory_cat ON m_shared_memory_entry(category);

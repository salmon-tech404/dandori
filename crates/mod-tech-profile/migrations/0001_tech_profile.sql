-- Migration 0001: Bảng hồ sơ kỹ thuật theo dự án (P1-08)

CREATE TABLE IF NOT EXISTS m_tech_profile_entry (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    category TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    version INTEGER NOT NULL DEFAULT 1,
    updated_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(project_id, key)
);

CREATE INDEX IF NOT EXISTS idx_tech_profile_project_cat
    ON m_tech_profile_entry(project_id, category, sort_order, key);

CREATE INDEX IF NOT EXISTS idx_tech_profile_workspace
    ON m_tech_profile_entry(workspace_id);

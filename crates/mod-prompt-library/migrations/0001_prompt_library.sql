-- Migration 0001: Bảng dữ liệu cho module prompt-library (P1-07, RS-DB-001)

CREATE TABLE IF NOT EXISTS m_prompt_library_prompt (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL,
    project_id TEXT,
    name TEXT NOT NULL,
    role TEXT,
    skill_tag TEXT,
    current_version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_prompt_library_scope
ON m_prompt_library_prompt(workspace_id, project_id, role);

CREATE TABLE IF NOT EXISTS m_prompt_library_version (
    id TEXT PRIMARY KEY NOT NULL,
    prompt_id TEXT NOT NULL REFERENCES m_prompt_library_prompt(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(prompt_id, version)
);

CREATE INDEX IF NOT EXISTS idx_prompt_library_version_lookup
ON m_prompt_library_version(prompt_id, version);

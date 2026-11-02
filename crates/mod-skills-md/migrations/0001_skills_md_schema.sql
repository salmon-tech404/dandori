-- Migration cho module skills-md (P3-08)

CREATE TABLE IF NOT EXISTS m_skills_md_skill (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    version TEXT,
    author TEXT,
    tags TEXT,
    tools TEXT,
    content TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    approved_hash TEXT,
    has_scripts INTEGER NOT NULL DEFAULT 0,
    script_files TEXT,
    status TEXT NOT NULL DEFAULT 'pending_approval',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(workspace_id, name)
);

CREATE INDEX IF NOT EXISTS idx_skills_md_workspace ON m_skills_md_skill(workspace_id);
CREATE INDEX IF NOT EXISTS idx_skills_md_status ON m_skills_md_skill(status);

CREATE TABLE IF NOT EXISTS m_skills_md_binding (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    skill_id TEXT NOT NULL REFERENCES m_skills_md_skill(id) ON DELETE CASCADE,
    project_id TEXT,
    role TEXT,
    embed_full INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    UNIQUE(skill_id, project_id, role)
);

CREATE INDEX IF NOT EXISTS idx_skills_md_binding_ws ON m_skills_md_binding(workspace_id);
CREATE INDEX IF NOT EXISTS idx_skills_md_binding_skill ON m_skills_md_binding(skill_id);
CREATE INDEX IF NOT EXISTS idx_skills_md_binding_match ON m_skills_md_binding(project_id, role);

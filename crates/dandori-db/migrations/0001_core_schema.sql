-- Migration 0001: Lược đồ cơ sở dữ liệu lõi của Dandori

-- 1. Bảng user
CREATE TABLE IF NOT EXISTS user (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    system_role TEXT NOT NULL CHECK (system_role IN ('admin', 'user')),
    created_at TEXT NOT NULL
);

-- 2. Bảng session
CREATE TABLE IF NOT EXISTS session (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES user(id) ON DELETE CASCADE,
    expires_at TEXT NOT NULL,
    token_hash TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_session_user_id ON session(user_id);

-- 3. Bảng workspace
CREATE TABLE IF NOT EXISTS workspace (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- 4. Bảng workspace_member
CREATE TABLE IF NOT EXISTS workspace_member (
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES user(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('admin', 'member')),
    PRIMARY KEY (workspace_id, user_id)
);
CREATE INDEX IF NOT EXISTS idx_workspace_member_user_id ON workspace_member(user_id);

-- 5. Bảng project
CREATE TABLE IF NOT EXISTS project (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    repo_path TEXT NOT NULL,
    default_branch TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_project_workspace_id ON project(workspace_id);

-- 6. Bảng task
CREATE TABLE IF NOT EXISTS task (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('backlog', 'running', 'needs_human', 'in_review', 'ready_to_merge', 'done', 'failed', 'cancelled', 'todo', 'in_progress')),
    lead_agent TEXT NOT NULL,
    worker_agent TEXT NOT NULL,
    workdir_path TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_task_project_id ON task(project_id);

-- 7. Bảng run
CREATE TABLE IF NOT EXISTS run (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL REFERENCES task(id) ON DELETE CASCADE,
    agent TEXT NOT NULL,
    account TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('queued', 'running', 'success', 'failed', 'cancelled')),
    started_at TEXT NOT NULL,
    finished_at TEXT,
    exit_code INTEGER
);
CREATE INDEX IF NOT EXISTS idx_run_task_id ON run(task_id);

-- 8. Bảng run_event
CREATE TABLE IF NOT EXISTS run_event (
    id TEXT PRIMARY KEY NOT NULL,
    run_id TEXT NOT NULL REFERENCES run(id) ON DELETE CASCADE,
    seq INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    data TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_run_event_run_id_seq ON run_event(run_id, seq);

-- 9. Bảng secret
CREATE TABLE IF NOT EXISTS secret (
    id TEXT PRIMARY KEY NOT NULL,
    label TEXT NOT NULL,
    nonce TEXT NOT NULL,
    ciphertext BLOB NOT NULL,
    created_at TEXT NOT NULL
);

-- 10. Bảng agent_account
CREATE TABLE IF NOT EXISTS agent_account (
    id TEXT PRIMARY KEY NOT NULL,
    provider TEXT NOT NULL,
    label TEXT NOT NULL,
    home_dir TEXT NOT NULL,
    auth_type TEXT NOT NULL CHECK (auth_type IN ('cli', 'api_key')),
    secret_id TEXT REFERENCES secret(id) ON DELETE RESTRICT,
    status TEXT NOT NULL CHECK (status IN ('not_installed', 'not_logged_in', 'ready', 'session_expired', 'unknown')),
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    priority INTEGER NOT NULL DEFAULT 100,
    exhausted_until TEXT,
    last_checked_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_agent_account_secret_id ON agent_account(secret_id);

-- 11. Bảng feature_module
CREATE TABLE IF NOT EXISTS feature_module (
    id TEXT PRIMARY KEY NOT NULL,
    group_name TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    version TEXT NOT NULL,
    requires TEXT NOT NULL,
    scopes TEXT NOT NULL,
    risk TEXT NOT NULL,
    effort TEXT NOT NULL,
    core INTEGER NOT NULL DEFAULT 0 CHECK (core IN (0, 1))
);

-- 12. Bảng feature_setting
CREATE TABLE IF NOT EXISTS feature_setting (
    module_id TEXT NOT NULL REFERENCES feature_module(id) ON DELETE CASCADE,
    scope_kind TEXT NOT NULL CHECK (scope_kind IN ('system', 'workspace', 'project')),
    scope_id TEXT NOT NULL,
    enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
    locked INTEGER NOT NULL DEFAULT 0 CHECK (locked IN (0, 1)),
    config TEXT NOT NULL DEFAULT '{}',
    changed_by TEXT NOT NULL,
    changed_at TEXT NOT NULL,
    PRIMARY KEY (module_id, scope_kind, scope_id)
);
CREATE INDEX IF NOT EXISTS idx_feature_setting_module_id ON feature_setting(module_id);

-- 13. Bảng feature_pack
CREATE TABLE IF NOT EXISTS feature_pack (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    modules TEXT NOT NULL
);

-- 14. Bảng feature_audit
CREATE TABLE IF NOT EXISTS feature_audit (
    id TEXT PRIMARY KEY NOT NULL,
    module_id TEXT NOT NULL,
    scope_kind TEXT NOT NULL CHECK (scope_kind IN ('system', 'workspace', 'project')),
    scope_id TEXT NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('enable', 'disable', 'configure', 'rollback')),
    performed_by TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_feature_audit_module_id ON feature_audit(module_id);

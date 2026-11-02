-- Migration 0001: Cost tracking tables
CREATE TABLE IF NOT EXISTS m_cost_tracking_pricing (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    input_token_price REAL NOT NULL,
    output_token_price REAL NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(project_id, provider, model)
);

CREATE TABLE IF NOT EXISTS m_cost_tracking_task_budget (
    task_id TEXT PRIMARY KEY,
    max_budget REAL NOT NULL,
    alerted_over_budget INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS m_cost_tracking_run_usage (
    run_id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    has_token_data INTEGER NOT NULL DEFAULT 0,
    input_tokens INTEGER,
    output_tokens INTEGER,
    total_tokens INTEGER,
    calculated_cost REAL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cost_tracking_run_usage_task ON m_cost_tracking_run_usage(task_id);
CREATE INDEX IF NOT EXISTS idx_cost_tracking_run_usage_project ON m_cost_tracking_run_usage(project_id);
CREATE INDEX IF NOT EXISTS idx_cost_tracking_pricing_project ON m_cost_tracking_pricing(project_id);

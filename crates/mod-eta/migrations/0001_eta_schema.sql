-- Migration cho module eta (P3-07)
CREATE TABLE IF NOT EXISTS m_eta_prediction (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    role TEXT NOT NULL,
    provider TEXT NOT NULL,
    sample_count INTEGER NOT NULL,
    predicted_median_secs REAL NOT NULL,
    predicted_p80_secs REAL NOT NULL,
    confidence TEXT NOT NULL,
    actual_duration_secs REAL,
    absolute_error_secs REAL,
    created_at TEXT NOT NULL,
    completed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_m_eta_prediction_ws ON m_eta_prediction(workspace_id);
CREATE INDEX IF NOT EXISTS idx_m_eta_prediction_task ON m_eta_prediction(task_id);
CREATE INDEX IF NOT EXISTS idx_m_eta_prediction_run ON m_eta_prediction(run_id);
CREATE INDEX IF NOT EXISTS idx_m_eta_prediction_project_role_provider ON m_eta_prediction(project_id, role, provider);

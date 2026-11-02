-- 0001_calendar.sql: Quản lý lịch định kỳ cho module calendar (P3-01)
CREATE TABLE IF NOT EXISTS m_calendar_schedule (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    schedule_type TEXT NOT NULL CHECK (schedule_type IN ('cron', 'interval')),
    schedule_expr TEXT NOT NULL,
    timezone TEXT NOT NULL DEFAULT 'UTC',
    task_template TEXT NOT NULL,
    auto_dispatch INTEGER NOT NULL DEFAULT 0 CHECK (auto_dispatch IN (0, 1)),
    missed_policy TEXT NOT NULL DEFAULT 'skip' CHECK (missed_policy IN ('skip', 'run_once')),
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    last_fire TEXT,
    next_fire TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_m_calendar_schedule_ws ON m_calendar_schedule(workspace_id);
CREATE INDEX IF NOT EXISTS idx_m_calendar_schedule_proj ON m_calendar_schedule(project_id);
CREATE INDEX IF NOT EXISTS idx_m_calendar_schedule_next ON m_calendar_schedule(enabled, next_fire);

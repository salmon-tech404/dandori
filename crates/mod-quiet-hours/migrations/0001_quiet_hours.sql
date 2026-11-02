-- Bảng lưu trữ cấu hình các khung giờ cấm chạy (Quiet Hours)
CREATE TABLE IF NOT EXISTS m_quiet_hours_window (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    project_id TEXT,
    name TEXT NOT NULL,
    days_of_week TEXT NOT NULL, -- Ví dụ: "1,2,3,4,5" (1=Thứ 2 .. 7=Chủ nhật)
    start_time TEXT NOT NULL,   -- "HH:MM", ví dụ "09:00" hoặc "22:00"
    end_time TEXT NOT NULL,     -- "HH:MM", ví dụ "18:00" hoặc "06:00"
    timezone TEXT NOT NULL DEFAULT 'UTC',
    block_manual_runs INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (workspace_id) REFERENCES workspace(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_m_quiet_hours_ws_prj 
ON m_quiet_hours_window(workspace_id, project_id, enabled);

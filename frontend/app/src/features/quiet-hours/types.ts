export interface QuietWindow {
  id: string;
  workspace_id: string;
  project_id: string | null;
  name: string;
  days_of_week: string;
  start_time: string;
  end_time: string;
  timezone: string;
  block_manual_runs: boolean;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateQuietWindowRequest {
  workspace_id: string;
  project_id?: string | null;
  name: string;
  days_of_week: string;
  start_time: string;
  end_time: string;
  timezone?: string;
  block_manual_runs?: boolean;
}

export interface UpdateQuietWindowRequest {
  name?: string;
  days_of_week?: string;
  start_time?: string;
  end_time?: string;
  timezone?: string;
  block_manual_runs?: boolean;
  enabled?: boolean;
}

export interface QuietStatus {
  in_quiet_window: boolean;
  active_window_names: string[];
  retry_after: string | null;
  blocks_manual_runs: boolean;
}

/**
 * Kiểu dữ liệu cho module run-history (P2-01)
 */

export interface RunSummary {
  id: string;
  task_id: string;
  task_title: string;
  agent: string;
  account: string;
  status: string;
  started_at: string;
  finished_at?: string | null;
  exit_code?: number | null;
  duration_seconds?: number | null;
  note?: string | null;
  tags: string[];
}

export interface RunDetail extends RunSummary {
  workspace_id: string;
  termination_reason?: string | null;
}

export interface RunEventItem {
  id: string;
  run_id: string;
  seq: number;
  event_type: string;
  data: string;
  created_at: string;
}

export interface RunHistoryNote {
  run_id: string;
  note: string;
  tags: string[];
  updated_at: string;
}

export interface ListRunsParams {
  task_id?: string;
  agent_provider?: string;
  agent_account_id?: string;
  status?: string;
  from?: string;
  to?: string;
  limit?: number;
  offset?: number;
}

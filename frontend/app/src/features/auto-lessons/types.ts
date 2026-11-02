export interface AutoLessonsConfig {
  project_id: string;
  workspace_id: string;
  enabled: boolean;
  trigger_on_done: boolean;
  trigger_on_failed: boolean;
  max_token_budget: number;
  min_content_length: number;
  created_at: string;
  updated_at: string;
}

export interface UpdateAutoLessonsConfigRequest {
  enabled?: boolean;
  trigger_on_done?: boolean;
  trigger_on_failed?: boolean;
  max_token_budget?: number;
  min_content_length?: number;
}

export interface AutoLessonsRunRecord {
  id: string;
  workspace_id: string;
  project_id: string;
  task_id: string;
  run_id?: string | null;
  status: string;
  lesson_id?: string | null;
  reason?: string | null;
  created_at: string;
  finished_at?: string | null;
}

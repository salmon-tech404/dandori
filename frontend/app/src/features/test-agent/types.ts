export interface TestAgentConfig {
  project_id: string;
  workspace_id: string;
  enabled: boolean;
  suggested_command: string | null;
  allowed_path_patterns: string;
  max_test_runs_per_worker: number;
  timeout_seconds: number;
  created_at: string;
  updated_at: string;
}

export interface UpdateTestAgentConfigRequest {
  enabled?: boolean;
  suggested_command?: string;
  allowed_path_patterns?: string[];
  max_test_runs_per_worker?: number;
  timeout_seconds?: number;
}

export interface TestAgentRun {
  id: string;
  task_id: string;
  project_id: string;
  workspace_id: string;
  run_id: string;
  trigger_run_id: string | null;
  status: "running" | "passed" | "failed";
  exit_code: number | null;
  summary: string | null;
  flagged_files: string;
  created_at: string;
  finished_at: string | null;
}

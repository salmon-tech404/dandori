export interface DocAgentConfig {
  project_id: string;
  workspace_id: string;
  enabled: boolean;
  allowed_path_patterns: string;
  prompt_template: string | null;
  timeout_seconds: number;
  created_at: string;
  updated_at: string;
}

export interface UpdateDocAgentConfigRequest {
  enabled?: boolean;
  allowed_path_patterns?: string[];
  prompt_template?: string;
  timeout_seconds?: number;
}

export interface DocAgentRun {
  id: string;
  task_id: string;
  project_id: string;
  workspace_id: string;
  run_id: string;
  status: "running" | "completed" | "failed";
  summary: string | null;
  changed_doc_files: string;
  flagged_files: string;
  created_at: string;
  finished_at: string | null;
}

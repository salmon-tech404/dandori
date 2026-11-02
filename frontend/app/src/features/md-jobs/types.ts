export interface MdJobDto {
  id: string;
  workspace_id: string;
  project_id: string;
  file_path: string;
  job_name: string;
  schedule_cron: string;
  agent_role: string;
  account_id: string | null;
  prompt: string;
  content_hash: string;
  approved_hash: string | null;
  status: "pending_approval" | "approved" | "rejected" | "syntax_error";
  next_fire: string | null;
  last_fire: string | null;
  created_at: string;
  updated_at: string;
}

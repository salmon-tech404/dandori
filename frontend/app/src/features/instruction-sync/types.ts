export interface InstructionSyncConfig {
  id: string;
  workspace_id: string;
  project_id: string;
  provider: string;
  target_filename: string;
  strategy: string;
  is_enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface UpdateConfigPayload {
  provider: string;
  target_filename: string;
  strategy: string;
  is_enabled: boolean;
}

export interface PreviewPayload {
  project_id: string;
  provider: string;
  role?: string;
}

export interface PreviewResult {
  target_filename: string;
  content: string;
  tech_profile_count: number;
  skills_count: number;
}

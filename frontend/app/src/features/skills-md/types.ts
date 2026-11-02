/**
 * Kiểu dữ liệu cho module skills-md (P3-08).
 */

export type SkillStatus = "pending_approval" | "approved" | "rejected";

export interface SkillSummary {
  id: string;
  workspace_id: string;
  name: string;
  description: string;
  version: string | null;
  author: string | null;
  tags: string[];
  has_scripts: boolean;
  status: SkillStatus;
  content_hash: string;
  is_hash_matching_approved: boolean;
  binding_count: number;
  created_at: string;
  updated_at: string;
}

export interface SkillBinding {
  id: string;
  workspace_id: string;
  skill_id: string;
  project_id: string | null;
  role: string | null;
  embed_full: boolean;
  created_at: string;
}

export interface SkillDetail {
  id: string;
  workspace_id: string;
  name: string;
  description: string;
  version: string | null;
  author: string | null;
  tags: string[];
  tools: string[];
  content: string;
  content_hash: string;
  approved_hash: string | null;
  has_scripts: boolean;
  script_files: string[];
  status: SkillStatus;
  bindings: SkillBinding[];
  created_at: string;
  updated_at: string;
}

export interface ImportDirPayload {
  dir_path: string;
}

export interface ImportZipPayload {
  file_name: string;
  base64_content: string;
}

export interface CreateBindingPayload {
  project_id?: string | null;
  role?: string | null;
  embed_full?: boolean;
}

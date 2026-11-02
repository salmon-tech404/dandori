/**
 * Kiểu dữ liệu cho module prompt-library (P1-07).
 */

export interface PromptSummary {
  id: string;
  workspace_id: string;
  project_id: string | null;
  name: string;
  role: string | null;
  skill_tag: string | null;
  current_version: number;
  created_at: string;
  updated_at: string;
}

export interface PromptDetail {
  id: string;
  workspace_id: string;
  project_id: string | null;
  name: string;
  role: string | null;
  skill_tag: string | null;
  version: number;
  content: string;
  created_at: string;
  updated_at: string;
}

export interface PromptVersion {
  id: string;
  version: number;
  content: string;
  created_at: string;
}

export interface TestRenderResult {
  rendered: string;
  variables_used: Record<string, string>;
}

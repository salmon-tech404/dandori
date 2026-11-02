/**
 * Kiểu dữ liệu cho module tech-profile (P1-08).
 */

export type TechProfileCategory =
  | 'stack'
  | 'commands'
  | 'naming'
  | 'protected_paths'
  | 'architecture'
  | 'notes';

export interface TechProfileEntry {
  id: string;
  workspace_id: string;
  project_id: string;
  category: string;
  key: string;
  value: string;
  sort_order: number;
  version: number;
  updated_by: string;
  created_at: string;
  updated_at: string;
}

export interface TechProfilePreview {
  project_id: string;
  preview: string;
  total_entries: number;
}

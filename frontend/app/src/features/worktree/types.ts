export interface WorktreeStatus {
  exists: boolean;
  workdir_path: string | null;
  branch: string | null;
  is_dirty: boolean;
  dirty_files: string[];
}

export interface CleanupResponse {
  success: boolean;
  message: string;
}

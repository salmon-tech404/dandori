export interface DiffFile {
  path: string;
  old_path: string | null;
  status: string;
  insertions: number;
  deletions: number;
  is_binary: boolean;
  diff: string;
  truncated: boolean;
}

export interface DiffStats {
  total_insertions: number;
  total_deletions: number;
  total_files_changed: number;
}

export interface DiffResult {
  base_ref: string;
  files: DiffFile[];
  stats: DiffStats;
}

export interface PreviewStatus {
  running: boolean;
  pid: number | null;
  port: number | null;
  host: string;
  url: string | null;
  command: string[];
  logs: string[];
}

export interface StartPreviewPayload {
  confirmed: boolean;
  command?: string[];
  port?: number;
}

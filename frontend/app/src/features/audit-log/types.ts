export interface AuditLogEntry {
  id: string;
  workspace_id: string;
  sequence: number;
  actor_id: string;
  actor_name: string;
  action: string;
  target_type: string;
  target_id: string;
  details: string;
  source: string;
  prev_hash: string;
  hash: string;
  created_at: string;
}

export interface VerifyResult {
  is_valid: boolean;
  records_checked: number;
  broken_at_id?: string | null;
  broken_at_sequence?: number | null;
  reason?: string | null;
}

export interface ListLogsQuery {
  action?: string;
  actor_id?: string;
  limit?: number;
  offset?: number;
}

export interface ExportResponse {
  workspace_id: string;
  exported_at: string;
  total_records: number;
  records: AuditLogEntry[];
}

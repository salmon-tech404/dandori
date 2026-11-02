export interface PublicMobileDevice {
  id: string;
  workspace_id: string;
  name: string;
  created_by: string;
  read_only: boolean;
  is_revoked: boolean;
  expires_at: string;
  last_used_at: string | null;
  created_at: string;
}

export interface CreateDeviceRequest {
  name: string;
  read_only?: boolean;
  ttl_hours?: number;
}

export interface CreateDeviceResponse {
  device: PublicMobileDevice;
  token: string;
}

export interface PublicMobileConfig {
  workspace_id: string;
  allowed_origins: string[];
  read_only: boolean;
}

export interface UpdateMobileConfigRequest {
  allowed_origins?: string[];
  read_only?: boolean;
}

export interface MobileStatusResponse {
  pending_approvals_count: number;
  active_runs_count: number;
  is_read_only: boolean;
  device_name: string;
}

export interface MobileApprovalItem {
  id: string;
  task_id: string;
  task_title: string;
  reason: string;
  status: string;
  created_at: string;
}

export interface MobileDecisionRequest {
  action: "approve" | "reject";
  comment?: string;
}

export interface MobileActionLog {
  id: string;
  workspace_id: string;
  device_id: string;
  action: string;
  target_id: string | null;
  detail: string | null;
  created_at: string;
}

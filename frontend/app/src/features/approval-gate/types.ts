export interface ApprovalRequest {
  id: string;
  workspace_id: string;
  project_id: string;
  task_id: string;
  kind: string;
  from_status: string;
  target_status: string;
  requester_id: string;
  status: "pending" | "approved" | "rejected" | "expired";
  reason?: string | null;
  decider_id?: string | null;
  created_at: string;
  decided_at?: string | null;
  expires_at: string;
  rejection_reason?: string | null;
}

export interface ApprovalPolicy {
  workspace_id: string;
  project_id: string;
  require_in_review_to_merge: boolean;
  disallow_self_approval: boolean;
  expiry_hours: number;
}

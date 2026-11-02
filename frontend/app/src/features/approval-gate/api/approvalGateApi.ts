import type { ApprovalPolicy, ApprovalRequest } from "../types";

export interface ListRequestsParams {
  workspace_id?: string;
  project_id?: string;
  task_id?: string;
  status?: string;
}

export const approvalGateApi = {
  async listRequests(params?: ListRequestsParams): Promise<ApprovalRequest[]> {
    const query = new URLSearchParams();
    if (params?.workspace_id) query.set("workspace_id", params.workspace_id);
    if (params?.project_id) query.set("project_id", params.project_id);
    if (params?.task_id) query.set("task_id", params.task_id);
    if (params?.status) query.set("status", params.status);

    const qs = query.toString();
    const res = await fetch(`/api/v1/m/approval-gate/requests${qs ? `?${qs}` : ""}`);
    if (!res.ok) {
      throw new Error(`Failed to list requests: ${res.statusText}`);
    }
    const data = (await res.json()) as { requests: ApprovalRequest[] };
    return data.requests;
  },

  async getRequest(id: string): Promise<ApprovalRequest> {
    const res = await fetch(`/api/v1/m/approval-gate/requests/${id}`);
    if (!res.ok) {
      throw new Error(`Failed to get request: ${res.statusText}`);
    }
    const data = (await res.json()) as { request: ApprovalRequest };
    return data.request;
  },

  async approveRequest(id: string, reason?: string): Promise<void> {
    const res = await fetch(`/api/v1/m/approval-gate/requests/${id}/approve`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ reason }),
    });
    if (!res.ok) {
      const err = (await res.json()) as { error?: string };
      throw new Error(err.error || `Failed to approve request: ${res.statusText}`);
    }
  },

  async rejectRequest(id: string, reason?: string): Promise<void> {
    const res = await fetch(`/api/v1/m/approval-gate/requests/${id}/reject`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ reason }),
    });
    if (!res.ok) {
      const err = (await res.json()) as { error?: string };
      throw new Error(err.error || `Failed to reject request: ${res.statusText}`);
    }
  },

  async getPolicy(projectId: string): Promise<ApprovalPolicy> {
    const res = await fetch(`/api/v1/m/approval-gate/policy/${projectId}`);
    if (!res.ok) {
      throw new Error(`Failed to get policy: ${res.statusText}`);
    }
    const data = (await res.json()) as { policy: ApprovalPolicy };
    return data.policy;
  },

  async updatePolicy(projectId: string, payload: Partial<ApprovalPolicy>): Promise<ApprovalPolicy> {
    const res = await fetch(`/api/v1/m/approval-gate/policy/${projectId}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    if (!res.ok) {
      throw new Error(`Failed to update policy: ${res.statusText}`);
    }
    const data = (await res.json()) as { policy: ApprovalPolicy };
    return data.policy;
  },
};

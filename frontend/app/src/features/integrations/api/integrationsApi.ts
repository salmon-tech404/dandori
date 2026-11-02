import type {
  CreateConnectionRequest,
  LinearImportRequest,
  LinearIssue,
  PublicIntegrationConnection,
} from "../types";

export const integrationsApi = {
  async listConnections(workspaceId: string): Promise<PublicIntegrationConnection[]> {
    const res = await fetch(`/api/v1/m/integrations/workspaces/${workspaceId}/connections`);
    if (!res.ok) {
      throw new Error(`Failed to list connections: ${res.statusText}`);
    }
    const data = await res.json();
    return data.connections ?? [];
  },

  async createConnection(
    workspaceId: string,
    req: CreateConnectionRequest,
  ): Promise<PublicIntegrationConnection> {
    const res = await fetch(`/api/v1/m/integrations/workspaces/${workspaceId}/connections`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(req),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({}));
      throw new Error(err.error ?? `Failed to create connection: ${res.statusText}`);
    }
    const data = await res.json();
    return data.connection ?? data;
  },

  async testConnection(connId: string): Promise<boolean> {
    const res = await fetch(`/api/v1/m/integrations/connections/${connId}/test`, {
      method: "POST",
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({}));
      throw new Error(err.error ?? `Test failed: ${res.statusText}`);
    }
    return true;
  },

  async deleteConnection(connId: string): Promise<void> {
    const res = await fetch(`/api/v1/m/integrations/connections/${connId}`, {
      method: "DELETE",
    });
    if (!res.ok) {
      throw new Error(`Failed to delete connection: ${res.statusText}`);
    }
  },

  async listLinearIssues(connId: string): Promise<LinearIssue[]> {
    const res = await fetch(`/api/v1/m/integrations/connections/${connId}/linear/issues`);
    if (!res.ok) {
      throw new Error(`Failed to list linear issues: ${res.statusText}`);
    }
    const data = await res.json();
    return data.issues ?? [];
  },

  async importLinearIssues(
    connId: string,
    req: LinearImportRequest,
  ): Promise<number> {
    const res = await fetch(`/api/v1/m/integrations/connections/${connId}/linear/import`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(req),
    });
    if (!res.ok) {
      throw new Error(`Failed to import linear issues: ${res.statusText}`);
    }
    const data = await res.json();
    return data.imported_count ?? 0;
  },
};

import type { DocAgentConfig, DocAgentRun, UpdateDocAgentConfigRequest } from "../types";

export const docAgentApi = {
  async getConfig(projectId: string): Promise<DocAgentConfig> {
    const res = await fetch(`/api/v1/m/doc-agent/projects/${projectId}/config`);
    if (!res.ok) {
      throw new Error(`Failed to fetch doc agent config: ${res.statusText}`);
    }
    const data = await res.json();
    return data.config ?? data;
  },

  async updateConfig(
    projectId: string,
    req: UpdateDocAgentConfigRequest,
  ): Promise<DocAgentConfig> {
    const res = await fetch(`/api/v1/m/doc-agent/projects/${projectId}/config`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(req),
    });
    if (!res.ok) {
      throw new Error(`Failed to update doc agent config: ${res.statusText}`);
    }
    const data = await res.json();
    return data.config ?? data;
  },

  async listResults(taskId: string): Promise<DocAgentRun[]> {
    const res = await fetch(`/api/v1/m/doc-agent/tasks/${taskId}/results`);
    if (!res.ok) {
      throw new Error(`Failed to list doc agent results: ${res.statusText}`);
    }
    const data = await res.json();
    return data.results ?? data;
  },
};

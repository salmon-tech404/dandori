import type { TestAgentConfig, TestAgentRun, UpdateTestAgentConfigRequest } from "../types";

export const testAgentApi = {
  async getConfig(projectId: string): Promise<TestAgentConfig> {
    const res = await fetch(`/api/v1/m/test-agent/projects/${projectId}/config`);
    if (!res.ok) {
      throw new Error(`Failed to get config: ${res.statusText}`);
    }
    const data = (await res.json()) as { config: TestAgentConfig };
    return data.config;
  },

  async updateConfig(
    projectId: string,
    req: UpdateTestAgentConfigRequest,
  ): Promise<TestAgentConfig> {
    const res = await fetch(`/api/v1/m/test-agent/projects/${projectId}/config`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(req),
    });
    if (!res.ok) {
      const err = (await res.json()) as { error?: string };
      throw new Error(err.error || `Failed to update config: ${res.statusText}`);
    }
    const data = (await res.json()) as { config: TestAgentConfig };
    return data.config;
  },

  async listResults(taskId: string): Promise<TestAgentRun[]> {
    const res = await fetch(`/api/v1/m/test-agent/tasks/${taskId}/results`);
    if (!res.ok) {
      throw new Error(`Failed to list test results: ${res.statusText}`);
    }
    const data = (await res.json()) as { results: TestAgentRun[] };
    return data.results;
  },
};

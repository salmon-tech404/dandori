import type { EtaAccuracyStats, EtaResult } from './types';

export async function fetchTaskEta(wsId: string, taskId: string): Promise<EtaResult> {
  const res = await fetch(`/api/v1/m/eta/workspaces/${wsId}/eta/task/${taskId}`);
  if (!res.ok) {
    throw new Error(`Failed to fetch ETA: ${res.statusText}`);
  }
  const data = await res.json();
  return data.eta;
}

export async function fetchEtaAccuracy(wsId: string, projectId?: string): Promise<EtaAccuracyStats> {
  const query = projectId ? `?project_id=${encodeURIComponent(projectId)}` : '';
  const res = await fetch(`/api/v1/m/eta/workspaces/${wsId}/eta/accuracy${query}`);
  if (!res.ok) {
    throw new Error(`Failed to fetch ETA accuracy: ${res.statusText}`);
  }
  const data = await res.json();
  return data.accuracy;
}

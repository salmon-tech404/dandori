import type {
  CurrentReportResponse,
  ReportConfig,
  ReportSnapshot,
} from './types';

const BASE_URL = '/api/v1/m/reports';

export async function fetchCurrentReport(
  workspaceId: string,
  periodType: 'daily' | 'weekly',
  timezone?: string
): Promise<CurrentReportResponse> {
  const params = new URLSearchParams({ period_type: periodType });
  if (timezone) params.append('timezone', timezone);

  const res = await fetch(`${BASE_URL}/workspaces/${workspaceId}/reports/current?${params}`);
  if (!res.ok) {
    throw new Error(`Failed to fetch current report: ${res.statusText}`);
  }
  return res.json();
}

export async function fetchReportSnapshots(workspaceId: string): Promise<ReportSnapshot[]> {
  const res = await fetch(`${BASE_URL}/workspaces/${workspaceId}/reports/snapshots`);
  if (!res.ok) {
    throw new Error(`Failed to fetch report snapshots: ${res.statusText}`);
  }
  const data = await res.json();
  return data.snapshots || [];
}

export async function fetchReportConfig(workspaceId: string): Promise<ReportConfig> {
  const res = await fetch(`${BASE_URL}/workspaces/${workspaceId}/reports/config`);
  if (!res.ok) {
    throw new Error(`Failed to fetch report config: ${res.statusText}`);
  }
  const data = await res.json();
  return data.config;
}

export async function updateReportConfig(
  workspaceId: string,
  payload: Partial<ReportConfig>
): Promise<void> {
  const res = await fetch(`${BASE_URL}/workspaces/${workspaceId}/reports/config`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  });
  if (!res.ok) {
    throw new Error(`Failed to update report config: ${res.statusText}`);
  }
}

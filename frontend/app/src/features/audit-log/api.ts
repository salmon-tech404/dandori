import type {
  AuditLogEntry,
  ExportResponse,
  ListLogsQuery,
  VerifyResult,
} from "./types";

async function jsonFetch<T>(url: string, init?: RequestInit): Promise<T> {
  const res = await fetch(url, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...init?.headers,
    },
  });
  if (!res.ok) {
    const err = (await res.json().catch(() => ({}))) as { error?: { message?: string } };
    throw new Error(err.error?.message || `Lỗi HTTP ${res.status}`);
  }
  return res.json() as Promise<T>;
}

export async function fetchAuditLogs(
  workspaceId: string,
  query?: ListLogsQuery,
): Promise<AuditLogEntry[]> {
  const params = new URLSearchParams();
  if (query?.action) params.set("action", query.action);
  if (query?.actor_id) params.set("actor_id", query.actor_id);
  if (query?.limit) params.set("limit", String(query.limit));
  if (query?.offset) params.set("offset", String(query.offset));

  const qs = params.toString();
  const url = `/api/v1/m/audit-log/workspaces/${encodeURIComponent(workspaceId)}/logs${
    qs ? `?${qs}` : ""
  }`;
  return jsonFetch<AuditLogEntry[]>(url);
}

export async function verifyAuditChain(workspaceId: string): Promise<VerifyResult> {
  return jsonFetch<VerifyResult>(
    `/api/v1/m/audit-log/workspaces/${encodeURIComponent(workspaceId)}/verify`,
  );
}

export async function exportAuditLogs(workspaceId: string): Promise<ExportResponse> {
  return jsonFetch<ExportResponse>(
    `/api/v1/m/audit-log/workspaces/${encodeURIComponent(workspaceId)}/export`,
  );
}

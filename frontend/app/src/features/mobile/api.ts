import type {
  CreateDeviceRequest,
  CreateDeviceResponse,
  MobileApprovalItem,
  MobileDecisionRequest,
  MobileStatusResponse,
  PublicMobileConfig,
  PublicMobileDevice,
  UpdateMobileConfigRequest,
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
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error?.message || `Lỗi HTTP ${res.status}`);
  }
  return res.json() as Promise<T>;
}

export async function fetchDevices(ws: string): Promise<PublicMobileDevice[]> {
  return jsonFetch<PublicMobileDevice[]>(`/api/v1/m/mobile/workspaces/${ws}/mobile/devices`);
}

export async function createDevice(
  ws: string,
  req: CreateDeviceRequest,
): Promise<CreateDeviceResponse> {
  return jsonFetch<CreateDeviceResponse>(`/api/v1/m/mobile/workspaces/${ws}/mobile/devices`, {
    method: "POST",
    body: JSON.stringify(req),
  });
}

export async function revokeDevice(ws: string, id: string): Promise<{ success: boolean }> {
  return jsonFetch<{ success: boolean }>(
    `/api/v1/m/mobile/workspaces/${ws}/mobile/devices/${id}/revoke`,
    { method: "POST" },
  );
}

export async function deleteDevice(ws: string, id: string): Promise<void> {
  const res = await fetch(`/api/v1/m/mobile/workspaces/${ws}/mobile/devices/${id}`, {
    method: "DELETE",
  });
  if (!res.ok) {
    throw new Error(`Lỗi xóa thiết bị: ${res.status}`);
  }
}

export async function fetchMobileConfig(ws: string): Promise<PublicMobileConfig> {
  return jsonFetch<PublicMobileConfig>(`/api/v1/m/mobile/workspaces/${ws}/mobile/config`);
}

export async function updateMobileConfig(
  ws: string,
  req: UpdateMobileConfigRequest,
): Promise<PublicMobileConfig> {
  return jsonFetch<PublicMobileConfig>(`/api/v1/m/mobile/workspaces/${ws}/mobile/config`, {
    method: "PUT",
    body: JSON.stringify(req),
  });
}

export async function fetchMobileStatus(token: string): Promise<MobileStatusResponse> {
  return jsonFetch<MobileStatusResponse>("/api/v1/m/mobile/mobile/status", {
    headers: { Authorization: `Bearer ${token}` },
  });
}

export async function fetchMobileApprovals(token: string): Promise<MobileApprovalItem[]> {
  return jsonFetch<MobileApprovalItem[]>("/api/v1/m/mobile/mobile/approvals", {
    headers: { Authorization: `Bearer ${token}` },
  });
}

export async function submitMobileDecision(
  token: string,
  id: string,
  req: MobileDecisionRequest,
): Promise<{ success: boolean; new_status: string }> {
  return jsonFetch<{ success: boolean; new_status: string }>(
    `/api/v1/m/mobile/mobile/approvals/${id}/decision`,
    {
      method: "POST",
      headers: { Authorization: `Bearer ${token}` },
      body: JSON.stringify(req),
    },
  );
}

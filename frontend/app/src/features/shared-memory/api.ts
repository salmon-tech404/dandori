/**
 * API client cho module shared-memory (P3-10).
 */

import type {
  BatchUpdateStatusPayload,
  CreateMemoryPayload,
  MemoryEntry,
  MemoryFilters,
  UpdateMemoryStatusPayload,
} from "./types";

const BASE_URL = "/api/v1/m/shared-memory";

export async function fetchMemories(
  workspaceId: string,
  filters: MemoryFilters = {}
): Promise<MemoryEntry[]> {
  const params = new URLSearchParams();
  if (filters.projectId) params.set("project_id", filters.projectId);
  if (filters.status) params.set("status", filters.status);
  if (filters.category) params.set("category", filters.category);

  const qs = params.toString();
  const url = `${BASE_URL}/workspaces/${workspaceId}/memories${qs ? `?${qs}` : ""}`;
  const res = await fetch(url);
  if (!res.ok) {
    throw new Error("Lỗi nạp danh sách bộ nhớ dùng chung");
  }
  const data = await res.json();
  if (Array.isArray(data)) return data;
  return data.memories || [];
}

export async function createMemory(
  workspaceId: string,
  payload: CreateMemoryPayload
): Promise<MemoryEntry> {
  const res = await fetch(`${BASE_URL}/workspaces/${workspaceId}/memories`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error?.message || "Lỗi tạo mục bộ nhớ");
  }
  const data = await res.json();
  return data.memory || data;
}

export async function updateMemoryStatus(
  workspaceId: string,
  memoryId: string,
  payload: UpdateMemoryStatusPayload
): Promise<MemoryEntry> {
  const res = await fetch(
    `${BASE_URL}/workspaces/${workspaceId}/memories/${memoryId}/status`,
    {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    }
  );
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error?.message || "Lỗi cập nhật trạng thái");
  }
  const data = await res.json();
  return data.memory || data;
}

export async function batchUpdateStatus(
  workspaceId: string,
  payload: BatchUpdateStatusPayload
): Promise<{ updated: number }> {
  const res = await fetch(
    `${BASE_URL}/workspaces/${workspaceId}/memories/batch-status`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    }
  );
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error?.message || "Lỗi cập nhật hàng loạt");
  }
  return res.json();
}

export async function deleteMemory(
  workspaceId: string,
  memoryId: string
): Promise<{ success: boolean }> {
  const res = await fetch(
    `${BASE_URL}/workspaces/${workspaceId}/memories/${memoryId}`,
    {
      method: "DELETE",
    }
  );
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error?.message || "Lỗi xóa mục bộ nhớ");
  }
  return res.json();
}

/**
 * Các hàm gọi REST API cho module instruction-sync (P3-09).
 */

import type {
  InstructionSyncConfig,
  PreviewPayload,
  PreviewResult,
  UpdateConfigPayload,
} from "./types";

export async function fetchInstructionConfigs(
  workspaceId: string,
  projectId: string
): Promise<InstructionSyncConfig[]> {
  const res = await fetch(
    `/api/v1/m/instruction-sync/workspaces/${workspaceId}/projects/${projectId}/config`
  );
  if (!res.ok) {
    throw new Error("Lỗi nạp cấu hình đồng bộ hướng dẫn");
  }
  return res.json();
}

export async function saveInstructionConfig(
  workspaceId: string,
  projectId: string,
  payload: UpdateConfigPayload
): Promise<InstructionSyncConfig> {
  const res = await fetch(
    `/api/v1/m/instruction-sync/workspaces/${workspaceId}/projects/${projectId}/config`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    }
  );
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error?.message || "Lỗi lưu cấu hình đồng bộ");
  }
  return res.json();
}

export async function fetchInstructionPreview(
  workspaceId: string,
  payload: PreviewPayload
): Promise<PreviewResult> {
  const res = await fetch(
    `/api/v1/m/instruction-sync/workspaces/${workspaceId}/preview`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    }
  );
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error?.message || "Lỗi tạo xem trước hướng dẫn");
  }
  return res.json();
}

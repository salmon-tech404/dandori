import type {
  AutoSwitchConfig,
  UpdateAutoSwitchConfigPayload,
  AutoSwitchHistoryItem,
} from "./types";

export async function fetchAutoSwitchConfig(projectId: string): Promise<AutoSwitchConfig> {
  const resp = await fetch(`/api/v1/m/auto-switch/projects/${projectId}/config`, {
    headers: { "Content-Type": "application/json" },
  });
  if (!resp.ok) {
    throw new Error(`Lỗi tải cấu hình tự chuyển tài khoản: ${resp.status}`);
  }
  return resp.json();
}

export async function updateAutoSwitchConfig(
  projectId: string,
  payload: UpdateAutoSwitchConfigPayload
): Promise<void> {
  const resp = await fetch(`/api/v1/m/auto-switch/projects/${projectId}/config`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!resp.ok) {
    throw new Error(`Lỗi cập nhật cấu hình tự chuyển tài khoản: ${resp.status}`);
  }
}

export async function fetchTaskSwitchHistory(taskId: string): Promise<AutoSwitchHistoryItem[]> {
  const resp = await fetch(`/api/v1/m/auto-switch/tasks/${taskId}/history`, {
    headers: { "Content-Type": "application/json" },
  });
  if (!resp.ok) {
    throw new Error(`Lỗi tải lịch sử chuyển tài khoản: ${resp.status}`);
  }
  return resp.json();
}

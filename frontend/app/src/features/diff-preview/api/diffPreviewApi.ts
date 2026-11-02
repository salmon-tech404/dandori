import type { DiffResult, PreviewStatus, StartPreviewPayload } from "../types";

export async function fetchDiff(taskId: string, baseRef?: string): Promise<DiffResult> {
  const query = baseRef ? `?base_ref=${encodeURIComponent(baseRef)}` : "";
  const resp = await fetch(`/api/v1/m/diff-preview/tasks/${taskId}/diff${query}`);
  if (!resp.ok) {
    throw new Error(`Lỗi tải diff: ${resp.status}`);
  }
  return resp.json();
}

export async function fetchPreviewStatus(taskId: string): Promise<PreviewStatus> {
  const resp = await fetch(`/api/v1/m/diff-preview/tasks/${taskId}/preview/status`);
  if (!resp.ok) {
    throw new Error(`Lỗi tải trạng thái xem trước: ${resp.status}`);
  }
  return resp.json();
}

export async function startPreview(taskId: string, payload: StartPreviewPayload): Promise<PreviewStatus> {
  const resp = await fetch(`/api/v1/m/diff-preview/tasks/${taskId}/preview/start`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!resp.ok) {
    const errText = await resp.text();
    throw new Error(errText || `Lỗi khởi chạy xem trước: ${resp.status}`);
  }
  return resp.json();
}

export async function stopPreview(taskId: string): Promise<{ stopped: boolean }> {
  const resp = await fetch(`/api/v1/m/diff-preview/tasks/${taskId}/preview/stop`, {
    method: "POST",
  });
  if (!resp.ok) {
    throw new Error(`Lỗi dừng xem trước: ${resp.status}`);
  }
  return resp.json();
}

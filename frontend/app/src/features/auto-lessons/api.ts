/**
 * API client cho module auto-lessons (P3-11).
 */

import type {
  AutoLessonsConfig,
  AutoLessonsRunRecord,
  UpdateAutoLessonsConfigRequest,
} from "./types";

const BASE_URL = "/api/v1/m/auto-lessons";

export async function fetchAutoLessonsConfig(
  projectId: string
): Promise<AutoLessonsConfig> {
  const res = await fetch(`${BASE_URL}/projects/${projectId}/auto-lessons/config`);
  if (!res.ok) {
    throw new Error("Lỗi nạp cấu hình auto-lessons");
  }
  return res.json();
}

export async function updateAutoLessonsConfig(
  projectId: string,
  payload: UpdateAutoLessonsConfigRequest
): Promise<AutoLessonsConfig> {
  const res = await fetch(`${BASE_URL}/projects/${projectId}/auto-lessons/config`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error?.message || "Lỗi cập nhật cấu hình auto-lessons");
  }
  return res.json();
}

export async function fetchAutoLessonsRuns(
  projectId: string
): Promise<AutoLessonsRunRecord[]> {
  const res = await fetch(`${BASE_URL}/projects/${projectId}/auto-lessons/runs`);
  if (!res.ok) {
    throw new Error("Lỗi nạp lịch sử auto-lessons");
  }
  const data = await res.json();
  if (Array.isArray(data)) return data;
  return [];
}

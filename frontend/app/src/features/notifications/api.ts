/**
 * API client cho module notifications (P3-12).
 */

import type {
  CreateChannelRequest,
  NotificationHistoryRecord,
  PublicNotificationChannel,
  UpdateChannelRequest,
} from "./types";

const BASE_URL = "/api/v1/m/notifications";

export async function fetchNotificationChannels(
  workspaceId: string
): Promise<PublicNotificationChannel[]> {
  const res = await fetch(`${BASE_URL}/workspaces/${workspaceId}/notifications/channels`);
  if (!res.ok) {
    throw new Error("Lỗi nạp danh sách kênh thông báo");
  }
  const data = await res.json();
  return Array.isArray(data) ? data : [];
}

export async function createNotificationChannel(
  workspaceId: string,
  payload: CreateChannelRequest
): Promise<PublicNotificationChannel> {
  const res = await fetch(`${BASE_URL}/workspaces/${workspaceId}/notifications/channels`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error || "Lỗi tạo kênh thông báo");
  }
  return res.json();
}

export async function updateNotificationChannel(
  workspaceId: string,
  channelId: string,
  payload: UpdateChannelRequest
): Promise<PublicNotificationChannel> {
  const res = await fetch(
    `${BASE_URL}/workspaces/${workspaceId}/notifications/channels/${channelId}`,
    {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    }
  );
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error || "Lỗi cập nhật kênh thông báo");
  }
  return res.json();
}

export async function deleteNotificationChannel(
  workspaceId: string,
  channelId: string
): Promise<void> {
  const res = await fetch(
    `${BASE_URL}/workspaces/${workspaceId}/notifications/channels/${channelId}`,
    { method: "DELETE" }
  );
  if (!res.ok) {
    throw new Error("Lỗi xóa kênh thông báo");
  }
}

export async function sendTestNotification(
  workspaceId: string,
  channelId: string
): Promise<{ success: boolean; message?: string; error?: string }> {
  const res = await fetch(
    `${BASE_URL}/workspaces/${workspaceId}/notifications/channels/${channelId}/test`,
    { method: "POST" }
  );
  return res.json();
}

export async function fetchNotificationHistory(
  workspaceId: string
): Promise<NotificationHistoryRecord[]> {
  const res = await fetch(`${BASE_URL}/workspaces/${workspaceId}/notifications/history`);
  if (!res.ok) {
    throw new Error("Lỗi nạp lịch sử thông báo");
  }
  const data = await res.json();
  return Array.isArray(data) ? data : [];
}

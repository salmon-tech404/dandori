import { render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { NotificationsPanel } from "./NotificationsPanel";
import type { NotificationHistoryRecord, PublicNotificationChannel } from "./types";

const mockChannels: PublicNotificationChannel[] = [
  {
    id: "chan-01",
    workspace_id: "ws-1",
    name: "DevOps Telegram",
    channel_type: "telegram",
    has_secret: true,
    chat_id: "-100123456",
    is_enabled: true,
    notify_on_needs_human: true,
    notify_on_failed: true,
    notify_on_approval_requested: true,
    notify_on_limit_warning: true,
    created_at: "2026-11-02T10:00:00Z",
    updated_at: "2026-11-02T10:00:00Z",
  },
];

const mockHistory: NotificationHistoryRecord[] = [
  {
    id: "hist-01",
    workspace_id: "ws-1",
    channel_id: "chan-01",
    event_type: "needs_human",
    task_id: "task-01",
    title: "[Dandori] Task task-01",
    status: "sent",
    created_at: "2026-11-02T10:05:00Z",
  },
];

function handleMockFetch(url: string): Promise<Response> {
  if (url.includes("/notifications/channels")) {
    return Promise.resolve({
      ok: true,
      json: async () => mockChannels,
    } as Response);
  }
  if (url.includes("/notifications/history")) {
    return Promise.resolve({
      ok: true,
      json: async () => mockHistory,
    } as Response);
  }
  return Promise.resolve({ ok: true, json: async () => ({}) } as Response);
}

describe("NotificationsPanel", () => {
  beforeEach(() => {
    vi.spyOn(global, "fetch").mockImplementation((url) =>
      handleMockFetch(String(url))
    );
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("hiển thị danh sách kênh và lịch sử thông báo", async () => {
    render(<NotificationsPanel workspaceId="ws-1" />);

    await waitFor(() => {
      expect(screen.getByText("DevOps Telegram")).toBeDefined();
      expect(screen.getByText("telegram")).toBeDefined();
      expect(screen.getByText("[Dandori] Task task-01")).toBeDefined();
      expect(screen.getByText("sent")).toBeDefined();
    });
  });
});

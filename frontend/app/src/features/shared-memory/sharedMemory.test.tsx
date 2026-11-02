import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { SharedMemoryPanel } from "./SharedMemoryPanel";
import type { MemoryEntry } from "./types";

const mockMemories: MemoryEntry[] = [
  {
    id: "mem-1",
    workspace_id: "ws-1",
    project_id: "proj-1",
    category: "decision",
    content: "Sử dụng SQLite WAL mode",
    source_type: "agent",
    source_ref: "agent:worker#run:1",
    status: "pending",
    reviewed_by: null,
    reviewed_at: null,
    created_at: "2026-11-02T10:00:00Z",
    updated_at: "2026-11-02T10:00:00Z",
  },
  {
    id: "mem-2",
    workspace_id: "ws-1",
    project_id: "proj-1",
    category: "convention",
    content: "Quy ước đặt tên biến",
    source_type: "human",
    source_ref: "user:admin",
    status: "approved",
    reviewed_by: "user:admin",
    reviewed_at: "2026-11-02T10:05:00Z",
    created_at: "2026-11-02T10:02:00Z",
    updated_at: "2026-11-02T10:05:00Z",
  },
];

function handleMockFetch(url: string, init?: RequestInit): Promise<Response> {
  if (url.includes("/memories") && (!init?.method || init.method === "GET")) {
    return Promise.resolve({ ok: true, json: async () => ({ memories: mockMemories }) } as Response);
  }
  if (url.includes("/status") && init?.method === "PATCH") {
    return Promise.resolve({
      ok: true,
      json: async () => ({ memory: { ...mockMemories[0], status: "approved" } }),
    } as Response);
  }
  if (url.includes("/batch-status") && init?.method === "POST") {
    return Promise.resolve({ ok: true, json: async () => ({ updated: 1 }) } as Response);
  }
  return Promise.resolve({ ok: true, json: async () => ({}) } as Response);
}

describe("SharedMemoryPanel", () => {
  beforeEach(() => {
    vi.spyOn(global, "fetch").mockImplementation((url, init) =>
      handleMockFetch(String(url), init)
    );
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders header and lists memory entries", async () => {
    render(<SharedMemoryPanel workspaceId="ws-1" projectId="proj-1" />);

    expect(screen.getByText("Bộ nhớ dùng chung")).toBeDefined();
    await waitFor(() => {
      expect(screen.getByText("Sử dụng SQLite WAL mode")).toBeDefined();
      expect(screen.getByText("Quy ước đặt tên biến")).toBeDefined();
    });
  });

  it("handles approve status update", async () => {
    render(<SharedMemoryPanel workspaceId="ws-1" projectId="proj-1" />);

    await waitFor(() => {
      expect(screen.getByText("Sử dụng SQLite WAL mode")).toBeDefined();
    });

    const approveButtons = screen.getAllByRole("button", { name: "Duyệt" });
    fireEvent.click(approveButtons[0]);

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining("/mem-1/status"),
        expect.objectContaining({ method: "PATCH" })
      );
    });
  });
});

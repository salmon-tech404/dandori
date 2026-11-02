import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { InstructionSyncPanel } from "./InstructionSyncPanel";
import type { InstructionSyncConfig, PreviewResult } from "./types";

const mockConfigs: InstructionSyncConfig[] = [
  {
    id: "cfg-1",
    workspace_id: "ws-1",
    project_id: "proj-1",
    provider: "claude",
    target_filename: "CLAUDE.md",
    strategy: "skip",
    is_enabled: true,
    created_at: "2026-11-02T10:00:00Z",
    updated_at: "2026-11-02T10:00:00Z",
  },
];

const mockPreview: PreviewResult = {
  target_filename: "CLAUDE.md",
  content: "<!-- Tệp sinh tự động bởi Dandori -->\n# Dandori Instructions\n## 1. Hồ sơ kỹ thuật",
  tech_profile_count: 3,
  skills_count: 2,
};

function handleMockFetch(url: string, init?: RequestInit): Promise<Response> {
  if (url.includes("/config") && (!init?.method || init.method === "GET")) {
    return Promise.resolve({ ok: true, json: async () => mockConfigs } as Response);
  }
  if (url.includes("/config") && init?.method === "POST") {
    return Promise.resolve({
      ok: true,
      json: async () => ({ ...mockConfigs[0], updated_at: "2026-11-02T10:05:00Z" }),
    } as Response);
  }
  if (url.includes("/preview")) {
    return Promise.resolve({ ok: true, json: async () => mockPreview } as Response);
  }
  return Promise.resolve({ ok: true, json: async () => ({}) } as Response);
}

describe("InstructionSyncPanel", () => {
  beforeEach(() => {
    vi.spyOn(global, "fetch").mockImplementation((url, init) =>
      handleMockFetch(String(url), init)
    );
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders instruction sync controls and loads config", async () => {
    render(<InstructionSyncPanel workspaceId="ws-1" projectId="proj-1" />);

    expect(screen.getByText("Đồng bộ tệp hướng dẫn CLI")).toBeDefined();
    await waitFor(() => {
      const input = screen.getByLabelText("Tệp hướng dẫn đích") as HTMLInputElement;
      expect(input.value).toBe("CLAUDE.md");
    });
  });

  it("triggers preview modal with generated content", async () => {
    render(<InstructionSyncPanel workspaceId="ws-1" projectId="proj-1" />);

    const previewBtn = screen.getByText("Xem trước nội dung sinh");
    fireEvent.click(previewBtn);

    await waitFor(() => {
      expect(screen.getByText(/Xem trước tệp hướng dẫn/)).toBeDefined();
      expect(screen.getByText(/Bao gồm 3 mục hồ sơ kỹ thuật và 2 kỹ năng đã duyệt/)).toBeDefined();
    });
  });

  it("saves configuration", async () => {
    render(<InstructionSyncPanel workspaceId="ws-1" projectId="proj-1" />);

    const saveBtn = screen.getByText("Lưu cấu hình");
    fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(screen.getByText("Đã lưu cấu hình đồng bộ thành công")).toBeDefined();
    });
  });
});

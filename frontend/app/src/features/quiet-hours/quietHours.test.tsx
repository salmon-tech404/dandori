import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { QuietHoursSettingsPanel } from "./QuietHoursSettingsPanel";
import { QuietHoursToolbarBadge } from "./QuietHoursToolbarBadge";
import type { QuietStatus, QuietWindow } from "./types";

describe("QuietHoursToolbarBadge", () => {
  it("hiển thị trạng thái đang trong giờ cấm", async () => {
    const mockStatus: QuietStatus = {
      in_quiet_window: true,
      active_window_names: ["Khung giờ cấm đêm"],
      retry_after: "2026-11-02T18:00:00Z",
      blocks_manual_runs: false,
    };
    vi.spyOn(global, "fetch").mockResolvedValueOnce({
      ok: true,
      json: async () => ({ status: mockStatus }),
    } as Response);

    render(<QuietHoursToolbarBadge workspaceId="ws-1" />);
    await waitFor(() => {
      expect(screen.getByText("Đang trong giờ cấm")).toBeDefined();
    });
  });

  it("hiển thị trạng thái ngoài giờ cấm", async () => {
    const mockStatus: QuietStatus = {
      in_quiet_window: false,
      active_window_names: [],
      retry_after: null,
      blocks_manual_runs: false,
    };
    vi.spyOn(global, "fetch").mockResolvedValueOnce({
      ok: true,
      json: async () => ({ status: mockStatus }),
    } as Response);

    render(<QuietHoursToolbarBadge workspaceId="ws-1" />);
    await waitFor(() => {
      expect(screen.getByText("Ngoài giờ cấm")).toBeDefined();
    });
  });
});

describe("QuietHoursSettingsPanel", () => {
  it("hiển thị danh sách khung giờ cấm đã cấu hình", async () => {
    const mockWindows: QuietWindow[] = [
      {
        id: "w-1",
        workspace_id: "ws-1",
        project_id: null,
        name: "Giờ nghỉ trưa",
        days_of_week: "1,2,3,4,5",
        start_time: "12:00",
        end_time: "13:00",
        timezone: "UTC",
        block_manual_runs: false,
        enabled: true,
        created_at: "2026-11-02T08:00:00Z",
        updated_at: "2026-11-02T08:00:00Z",
      },
    ];

    vi.spyOn(global, "fetch").mockResolvedValueOnce({
      ok: true,
      json: async () => ({ windows: mockWindows }),
    } as Response);

    render(<QuietHoursSettingsPanel workspaceId="ws-1" />);
    await waitFor(() => {
      expect(screen.getByText("Giờ nghỉ trưa")).toBeDefined();
      expect(screen.getByText("12:00 - 13:00")).toBeDefined();
      expect(screen.getByText("Thêm khung cấm")).toBeDefined();
    });
  });
});

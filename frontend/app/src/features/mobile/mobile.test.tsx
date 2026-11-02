import { render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MobileScreen } from "./MobileScreen";
import { MobileSettingsPanel } from "./MobileSettingsPanel";
import type { MobileApprovalItem, MobileStatusResponse, PublicMobileConfig, PublicMobileDevice } from "./types";

const mockStatus: MobileStatusResponse = {
  pending_approvals_count: 1,
  active_runs_count: 0,
  device_name: "iPhone Test",
  is_read_only: false,
};

const mockApprovals: MobileApprovalItem[] = [
  {
    id: "appr-1",
    task_id: "task-101",
    task_title: "Cập nhật tài liệu API",
    reason: "Cần quyền chạy lệnh git push",
    status: "pending",
    created_at: "2026-11-02T12:00:00Z",
  },
];

const mockDevices: PublicMobileDevice[] = [
  {
    id: "dev-1",
    workspace_id: "ws-1",
    name: "iPhone 15 Pro",
    created_by: "user-1",
    read_only: false,
    is_revoked: false,
    expires_at: "2026-12-01T00:00:00Z",
    last_used_at: null,
    created_at: "2026-11-02T00:00:00Z",
  },
];

const mockConfig: PublicMobileConfig = {
  workspace_id: "ws-1",
  allowed_origins: ["https://dandori.local"],
  read_only: false,
};

function handleMockFetch(url: string): Promise<Response> {
  if (url.includes("/mobile/status")) {
    return Promise.resolve({ ok: true, json: async () => mockStatus } as Response);
  }
  if (url.includes("/mobile/approvals")) {
    return Promise.resolve({ ok: true, json: async () => mockApprovals } as Response);
  }
  if (url.includes("/mobile/devices")) {
    return Promise.resolve({ ok: true, json: async () => mockDevices } as Response);
  }
  if (url.includes("/mobile/config")) {
    return Promise.resolve({ ok: true, json: async () => mockConfig } as Response);
  }
  return Promise.resolve({ ok: true, json: async () => ({}) } as Response);
}

describe("Mobile Feature Component Tests", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.stubGlobal("fetch", vi.fn(handleMockFetch));
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders TokenInputForm when no mobile token is set", () => {
    render(<MobileScreen />);
    expect(screen.getByText("Ghép đôi thiết bị di động")).toBeDefined();
    const input = screen.getByPlaceholderText("mob_...");
    expect(input.className).toContain("min-h-[44px]");
    const button = screen.getByRole("button", { name: "Kết nối" });
    expect(button.className).toContain("min-h-[44px]");
  });

  it("renders MobileHeader and ApprovalList when token is provided in localStorage", async () => {
    localStorage.setItem("dandori_mobile_token", "mob_testtoken123");
    render(<MobileScreen />);

    await waitFor(() => {
      expect(screen.getByText("Dandori Mobile")).toBeDefined();
      expect(screen.getByText("Cập nhật tài liệu API")).toBeDefined();
    });

    const approveBtn = screen.getByRole("button", { name: "Duyệt" });
    const rejectBtn = screen.getByRole("button", { name: "Từ chối" });
    expect(approveBtn.className).toContain("min-h-[44px]");
    expect(rejectBtn.className).toContain("min-h-[44px]");
  });

  it("renders MobileSettingsPanel with devices list and action buttons", async () => {
    render(<MobileSettingsPanel workspaceId="ws-1" />);

    await waitFor(() => {
      expect(screen.getByText("Thiết bị di động (Mobile Companion)")).toBeDefined();
      expect(screen.getByText("iPhone 15 Pro")).toBeDefined();
    });

    expect(screen.getByRole("button", { name: "Thêm thiết bị" })).toBeDefined();
    expect(screen.getByRole("button", { name: "Thu hồi" })).toBeDefined();
  });
});

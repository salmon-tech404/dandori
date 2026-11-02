import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { SkillsPanel } from "./SkillsPanel";
import type { SkillSummary } from "./types";

const mockSkills: SkillSummary[] = [
  {
    id: "skill-1",
    workspace_id: "ws-1",
    name: "code-review",
    description: "Rà soát mã nguồn Clean Code",
    version: "1.0.0",
    author: "Dandori",
    tags: ["quality"],
    has_scripts: true,
    status: "pending_approval",
    content_hash: "hash1",
    is_hash_matching_approved: false,
    binding_count: 2,
    created_at: "2026-11-02T10:00:00Z",
    updated_at: "2026-11-02T10:00:00Z",
  },
  {
    id: "skill-2",
    workspace_id: "ws-1",
    name: "sql-optimizer",
    description: "Tối ưu hóa câu lệnh SQLite",
    version: "0.9.0",
    author: null,
    tags: [],
    has_scripts: false,
    status: "approved",
    content_hash: "hash2",
    is_hash_matching_approved: true,
    binding_count: 1,
    created_at: "2026-11-02T10:00:00Z",
    updated_at: "2026-11-02T10:00:00Z",
  },
];

describe("SkillsPanel Rendering", () => {
  beforeEach(() => {
    vi.spyOn(global, "fetch").mockImplementation((url) => {
      const u = String(url);
      if (u.includes("/skills")) {
        return Promise.resolve({
          ok: true,
          json: async () => ({ skills: mockSkills }),
        } as Response);
      }
      return Promise.resolve({ ok: true, json: async () => ({}) } as Response);
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("AC 4 & AC 5: renders skills list, status, and script warning", async () => {
    render(<SkillsPanel workspaceId="ws-1" />);

    await waitFor(() => {
      expect(screen.getByText("code-review")).toBeDefined();
      expect(screen.getByText("sql-optimizer")).toBeDefined();
    });

    expect(screen.getByText(/Dandori tuyệt đối không bao giờ thực thi script/i)).toBeDefined();
    expect(screen.getByText("Có script")).toBeDefined();
    expect(screen.getByText("Chờ duyệt")).toBeDefined();
    expect(screen.getByText("Đã duyệt")).toBeDefined();
  });

  it("displays empty placeholder when no skills exist", async () => {
    vi.spyOn(global, "fetch").mockResolvedValueOnce({
      ok: true,
      json: async () => ({ skills: [] }),
    } as Response);

    render(<SkillsPanel workspaceId="ws-1" />);

    await waitFor(() => {
      expect(screen.getByText("Chưa có kỹ năng nào được nhập.")).toBeDefined();
    });
  });
});

describe("SkillsPanel Actions", () => {
  beforeEach(() => {
    vi.spyOn(global, "fetch").mockImplementation(() =>
      Promise.resolve({
        ok: true,
        json: async () => ({ skills: mockSkills }),
      } as Response)
    );
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("AC 5: triggers approve skill call when clicking approve button", async () => {
    render(<SkillsPanel workspaceId="ws-1" />);

    await waitFor(() => {
      expect(screen.getByText("code-review")).toBeDefined();
    });

    const approveBtn = screen.getByText("Phê duyệt");
    fireEvent.click(approveBtn);

    expect(global.fetch).toHaveBeenCalledWith(
      expect.stringContaining("/skills/skill-1/approve"),
      expect.objectContaining({ method: "POST" })
    );
  });

  it("AC 2: opens import modal and switches tabs between dir and zip", async () => {
    render(<SkillsPanel workspaceId="ws-1" />);

    const importBtn = screen.getByRole("button", { name: "Nhập kỹ năng" });
    fireEvent.click(importBtn);

    expect(screen.getByText("Nhập từ thư mục cục bộ")).toBeDefined();
    expect(screen.getByText("Nhập từ tệp nén ZIP")).toBeDefined();

    const zipTab = screen.getByText("Nhập từ tệp nén ZIP");
    fireEvent.click(zipTab);
    expect(screen.getByRole("button", { name: "Hủy" })).toBeDefined();
  });
});

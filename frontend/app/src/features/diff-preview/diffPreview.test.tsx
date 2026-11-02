import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { DiffPreviewPanel } from "./components/DiffPreviewPanel";
import { contributions } from "./index";

describe("DiffPreview Slot Contribution", () => {
  it("contributes diff-preview-tab to board.task-panel.tab slot", () => {
    expect(contributions["diff-preview-tab"]).toBeDefined();
  });
});

describe("DiffPreview File List & Stats", () => {
  beforeEach(() => { vi.restoreAllMocks(); });
  afterEach(() => { vi.restoreAllMocks(); });

  it("renders diff files list and stats", async () => {
    vi.spyOn(globalThis, "fetch").mockImplementation(async () => ({
      ok: true,
      json: async () => ({
        base_ref: "main",
        files: [{ path: "src/index.ts", old_path: null, status: "modified", insertions: 5, deletions: 2, is_binary: false, diff: "@@ -1 +1 @@\n-old\n+new", truncated: false }],
        stats: { total_insertions: 5, total_deletions: 2, total_files_changed: 1 },
      }),
    } as Response));

    render(<DiffPreviewPanel context={{ taskId: "task-1" }} />);
    await waitFor(() => {
      expect(screen.getByText("src/index.ts")).toBeDefined();
      expect(screen.getByText("+5")).toBeDefined();
    });
  });
});

describe("DiffPreview Truncation & XSS Safety (AC2)", () => {
  beforeEach(() => { vi.restoreAllMocks(); });
  afterEach(() => { vi.restoreAllMocks(); });

  it("renders truncation banner when file is truncated (AC2)", async () => {
    vi.spyOn(globalThis, "fetch").mockImplementation(async () => ({
      ok: true,
      json: async () => ({
        base_ref: "main",
        files: [{ path: "big.txt", old_path: null, status: "modified", insertions: 1000, deletions: 0, is_binary: false, diff: "...", truncated: true }],
        stats: { total_insertions: 1000, total_deletions: 0, total_files_changed: 1 },
      }),
    } as Response));

    render(<DiffPreviewPanel context={{ taskId: "task-1" }} />);
    await waitFor(() => {
      expect(screen.getByText(/cắt bớt do vượt quá kích thước/i)).toBeDefined();
    });
  });

  it("renders file named with script tags safely as plain text (AC2 XSS)", async () => {
    const maliciousName = "<script>alert('xss')</script>.txt";
    vi.spyOn(globalThis, "fetch").mockImplementation(async () => ({
      ok: true,
      json: async () => ({
        base_ref: "main",
        files: [{ path: maliciousName, old_path: null, status: "added", insertions: 1, deletions: 0, is_binary: false, diff: "+clean", truncated: false }],
        stats: { total_insertions: 1, total_deletions: 0, total_files_changed: 1 },
      }),
    } as Response));

    render(<DiffPreviewPanel context={{ taskId: "task-1" }} />);
    await waitFor(() => {
      const elem = screen.getByText(maliciousName);
      expect(elem).toBeDefined();
      expect(elem.tagName).toBe("SPAN");
    });
  });
});

describe("DiffPreview Confirmation & Error Handling (AC4)", () => {
  beforeEach(() => { vi.restoreAllMocks(); });
  afterEach(() => { vi.restoreAllMocks(); });

  it("requires confirmation checkbox before start preview button is enabled (AC4)", async () => {
    vi.spyOn(globalThis, "fetch").mockImplementation(async () => ({
      ok: true,
      json: async () => ({ base_ref: "main", files: [], stats: { total_insertions: 0, total_deletions: 0, total_files_changed: 0 } }),
    } as Response));

    render(<DiffPreviewPanel context={{ taskId: "task-1" }} />);
    await waitFor(() => { expect(screen.getByText("Bản xem trước")).toBeDefined(); });
    fireEvent.click(screen.getByText("Bản xem trước"));
    const startBtn = screen.getByText("Khởi chạy bản xem trước") as HTMLButtonElement;
    expect(startBtn.disabled).toBe(true);
    fireEvent.click(screen.getByRole("checkbox"));
    expect(startBtn.disabled).toBe(false);
  });

  it("renders error state when fetch fails", async () => {
    vi.spyOn(globalThis, "fetch").mockRejectedValue(new Error("Network Error"));
    render(<DiffPreviewPanel context={{ taskId: "task-1" }} />);
    await waitFor(() => { expect(screen.getByText("Network Error")).toBeDefined(); });
  });
});

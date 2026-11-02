import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { WorktreeSection } from "./WorktreeSection";

describe("WorktreeSection: Trạng thái rỗng", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("hiển thị trạng thái rỗng khi chưa có worktree", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        exists: false,
        workdir_path: null,
        branch: null,
        is_dirty: false,
        dirty_files: [],
      }),
    } as Response);

    render(<WorktreeSection context={{ taskId: "t-1" }} />);
    await waitFor(() => {
      expect(
        screen.getByText("Chưa tạo worktree riêng. Bấm 'Giao việc' để tạo tự động.")
      ).toBeDefined();
    });
  });
});

describe("WorktreeSection: Hiển thị chi tiết worktree", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("hiển thị tên nhánh, thư mục và cảnh báo tệp dirty", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        exists: true,
        workdir_path: "C:\\data\\worktrees\\p1\\t1",
        branch: "task/feature-abc-1234",
        is_dirty: true,
        dirty_files: ["file1.rs", "file2.rs"],
      }),
    } as Response);

    render(<WorktreeSection context={{ taskId: "t-1" }} />);
    await waitFor(() => {
      expect(screen.getByText("task/feature-abc-1234")).toBeDefined();
    });

    expect(screen.getByText("C:\\data\\worktrees\\p1\\t1")).toBeDefined();
    expect(screen.getByText("Có 2 tệp thay đổi chưa commit")).toBeDefined();
    expect(screen.getByRole("button", { name: "Dọn dẹp worktree" })).toBeDefined();
  });
});

function mockCleanupResponses() {
  const statusExists = {
    exists: true,
    workdir_path: "C:\\data\\worktrees\\p1\\t1",
    branch: "task/feature-abc-1234",
    is_dirty: true,
    dirty_files: ["file1.rs"],
  };
  const cleanupOk = {
    success: true,
    message: "Đã dọn dẹp worktree thành công",
  };
  const statusCleaned = {
    exists: false,
    workdir_path: null,
    branch: null,
    is_dirty: false,
    dirty_files: [],
  };

  vi.spyOn(globalThis, "fetch")
    .mockResolvedValueOnce({ ok: true, json: async () => statusExists } as Response)
    .mockResolvedValueOnce({ ok: true, json: async () => cleanupOk } as Response)
    .mockResolvedValueOnce({ ok: true, json: async () => statusCleaned } as Response);
}

describe("WorktreeSection: Hộp thoại xác nhận dọn dẹp", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("mở hộp thoại xác nhận và thực hiện dọn dẹp", async () => {
    mockCleanupResponses();
    render(<WorktreeSection context={{ taskId: "t-1" }} />);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Dọn dẹp worktree" })).toBeDefined();
    });

    fireEvent.click(screen.getByRole("button", { name: "Dọn dẹp worktree" }));
    expect(screen.getByText("Xác nhận dọn dẹp worktree")).toBeDefined();

    fireEvent.click(screen.getByRole("button", { name: "Xác nhận xóa" }));
    await waitFor(() => {
      expect(screen.getByText("Đã dọn dẹp worktree thành công")).toBeDefined();
    });
  });
});

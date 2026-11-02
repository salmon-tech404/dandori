import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { LeadWorkerDefaultsSection } from "./LeadWorkerDefaultsSection";
import { TaskAssignmentSection } from "./TaskAssignmentSection";

describe("LeadWorker: Render TaskAssignment", () => {
  beforeEach(() => { vi.restoreAllMocks(); });

  it("hiển thị thông tin phân công và nút Giao việc", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        task_id: "t-1",
        effective_lead_agent: "claude",
        effective_worker_agent: "gemini",
        task_lead_agent: "",
        task_worker_agent: "",
        is_lead_overridden: false,
        is_worker_overridden: false,
        ready_providers: ["claude", "gemini"],
      }),
    } as Response);

    render(<TaskAssignmentSection context={{ taskId: "t-1", title: "Task 1", description: "Desc" }} />);
    await waitFor(() => {
      expect(screen.getByText("Phân công Agent")).toBeDefined();
    });
    expect(screen.getByRole("button", { name: "Giao việc" })).toBeDefined();
  });
});

describe("LeadWorker: Dispatch Missing Worktree Error", () => {
  beforeEach(() => { vi.restoreAllMocks(); });

  it("hiển thị thông báo tiếng Việt khi thiếu worktree", async () => {
    vi.spyOn(globalThis, "fetch")
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          task_id: "t-1",
          effective_lead_agent: "claude",
          effective_worker_agent: "gemini",
          task_lead_agent: "",
          task_worker_agent: "",
          is_lead_overridden: false,
          is_worker_overridden: false,
          ready_providers: ["claude"],
        }),
      } as Response)
      .mockResolvedValueOnce({
        ok: false,
        status: 400,
        json: async () => ({
          error: {
            code: "workdir_provider_missing",
            message: "Thiếu môi trường làm việc (WorkdirProvider). Vui lòng bật module 'worktree' trong Cài đặt > Tính năng để bắt đầu giao việc.",
          },
        }),
      } as Response);

    render(<TaskAssignmentSection context={{ taskId: "t-1" }} />);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Giao việc" })).toBeDefined();
    });

    fireEvent.click(screen.getByRole("button", { name: "Giao việc" }));
    await waitFor(() => {
      expect(screen.getByRole("alert").textContent).toContain("Thiếu môi trường làm việc (WorkdirProvider)");
    });
  });
});

describe("LeadWorker: Defaults Section", () => {
  beforeEach(() => { vi.restoreAllMocks(); });

  it("hiển thị 4 vai trò mặc định và thao tác lưu", async () => {
    vi.spyOn(globalThis, "fetch")
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          defaults: [
            { role: "pm", provider_order: ["claude", "gemini", "codex"] },
            { role: "worker", provider_order: ["gemini", "claude", "codex"] },
            { role: "test", provider_order: ["codex", "gemini", "claude"] },
            { role: "doc", provider_order: ["claude", "gemini", "codex"] },
          ],
        }),
      } as Response)
      .mockResolvedValueOnce({ ok: true, json: async () => ({ status: "ok" }) } as Response);

    render(<LeadWorkerDefaultsSection context={{ projectId: "p-1" }} />);
    await waitFor(() => {
      expect(screen.getByText("PM (Trưởng nhóm / Phân tích)")).toBeDefined();
    });

    const saveBtn = screen.getByRole("button", { name: "Lưu cấu hình mặc định" });
    fireEvent.click(saveBtn);
    await waitFor(() => {
      expect(screen.getByText("Đã lưu thứ tự ưu tiên vai trò")).toBeDefined();
    });
  });
});

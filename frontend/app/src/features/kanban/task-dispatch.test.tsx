import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { TaskDetailPanel } from "./TaskDetailPanel";
import type { Task } from "../../../../../shared/types";

const mockBacklogTask: Task = {
  id: "t-dispatch-1",
  project_id: "p1",
  title: "Xây dựng pipeline CI/CD",
  description: "Thiết lập GitHub Actions cho repository",
  status: "backlog",
  lead_agent: "",
  worker_agent: "",
  workdir_path: null,
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
};

function renderPanel(task: Task, onUpdated?: () => void) {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>
        <TaskDetailPanel task={task} onClose={vi.fn()} onTaskUpdated={onUpdated} />
      </MemoryRouter>
    </QueryClientProvider>
  );
}

describe("UI-05: Task Dispatching", () => {
  beforeEach(() => { vi.restoreAllMocks(); });

  it("AC1: Dispatches task to agent successfully", async () => {
    const onUpdated = vi.fn();
    global.fetch = vi.fn().mockImplementation((url: string, opts) => {
      if (url.includes("/dispatch") && opts?.method === "POST") {
        return Promise.resolve({
          ok: true,
          status: 201,
          json: async () => ({ run_id: "run-101", status: "running", agent: "fake", account: "acc" }),
        });
      }
      return Promise.resolve({ ok: true, status: 200, json: async () => ({}) });
    });

    renderPanel(mockBacklogTask, onUpdated);
    const dispatchBtn = screen.getByTestId("btn-dispatch-task");
    fireEvent.click(dispatchBtn);

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        "/api/v1/tasks/t-dispatch-1/dispatch",
        expect.objectContaining({ method: "POST" })
      );
      expect(onUpdated).toHaveBeenCalled();
    });
  });
});

describe("UI-05: Missing Account Guidance", () => {
  beforeEach(() => { vi.restoreAllMocks(); });

  it("AC2: Shows guidance link to Agents screen when no agent account is ready", async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 400,
      json: async () => ({
        error: { code: "no_available_account", message: "Chưa cấu hình tài khoản CLI" },
      }),
    });

    renderPanel(mockBacklogTask);
    fireEvent.click(screen.getByTestId("btn-dispatch-task"));

    await waitFor(() => {
      expect(screen.getByTestId("no-account-alert")).toBeDefined();
      expect(screen.getByText("Agent và tài khoản")).toBeDefined();
    });
  });
});

function mockRunningFetch(onCancel: () => void) {
  global.fetch = vi.fn().mockImplementation((url: string, opts) => {
    if (url.includes("/cancel") && opts?.method === "POST") {
      onCancel();
      return Promise.resolve({ ok: true, status: 200, json: async () => ({ status: "cancelling" }) });
    }
    if (url.includes("/events")) {
      return Promise.resolve({
        ok: true,
        status: 200,
        json: async () => ({
          events: [{ id: "ev-1", run_id: "run-101", seq: 1, event_type: "output", data: "<script>alert('pwn')</script>" }],
        }),
      });
    }
    return Promise.resolve({ ok: true, status: 200, json: async () => ({ runs: [{ id: "run-101", status: "running" }] }) });
  });
}

describe("UI-05: Live Run Tracking & Cancel", () => {
  beforeEach(() => { vi.restoreAllMocks(); });

  it("AC3: Cancels running task and renders log text securely without innerHTML injection", async () => {
    const runningTask: Task = { ...mockBacklogTask, status: "running" };
    let cancelCalled = false;
    mockRunningFetch(() => { cancelCalled = true; });

    renderPanel(runningTask);

    await waitFor(() => {
      const logViewer = screen.getByTestId("task-live-logs");
      expect(logViewer.textContent).toContain("<script>alert('pwn')</script>");
      expect(document.querySelector("script[src*='pwn']")).toBeNull();
    });

    const cancelBtn = await screen.findByTestId("btn-cancel-run");
    fireEvent.click(cancelBtn);

    await waitFor(() => {
      expect(cancelCalled).toBe(true);
    });
  });
});

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import fs from "node:fs";
import path from "node:path";
import { MemoryRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { CreateTaskModal } from "./CreateTaskModal";
import { KanbanBoard } from "./KanbanBoard";

describe("UI-04: Static Code Inspection", () => {
  it("AC1: App.tsx must not contain hardcoded INITIAL_TASKS", () => {
    const appTsxPath = path.resolve(__dirname, "../../App.tsx");
    const content = fs.readFileSync(appTsxPath, "utf-8");
    expect(content.includes("INITIAL_TASKS")).toBe(false);
  });
});

function setupCreateTaskMock(onCreated: () => void, onClose: () => void) {
  global.fetch = vi.fn().mockImplementation((url, opts) => {
    if (url.includes("/api/v1/projects/p-test/tasks") && opts?.method === "POST") {
      return Promise.resolve({
        ok: true,
        status: 201,
        json: async () => ({
          task: { id: "task-new-1", project_id: "p-test", title: "Task 1", description: "Desc", status: "backlog" },
        }),
      });
    }
    return Promise.reject(new Error(`Unhandled fetch: ${url}`));
  });

  return render(
    <CreateTaskModal isOpen={true} onClose={onClose} projectId="p-test" onCreated={onCreated} />
  );
}

describe("UI-04: CreateTaskModal Wiring", () => {
  beforeEach(() => { vi.restoreAllMocks(); });

  it("AC2: validates empty title and calls real POST API on submit", async () => {
    const onCreated = vi.fn();
    const onClose = vi.fn();
    setupCreateTaskMock(onCreated, onClose);

    const submitBtn = screen.getByTestId("btn-submit-create-task");
    fireEvent.click(submitBtn);
    expect(screen.getByRole("alert")).toBeDefined();
    expect(onCreated).not.toHaveBeenCalled();

    fireEvent.change(screen.getByTestId("create-task-title-input"), { target: { value: "Task 1" } });
    fireEvent.change(screen.getByTestId("create-task-desc-input"), { target: { value: "Desc" } });
    fireEvent.click(submitBtn);

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        "/api/v1/projects/p-test/tasks",
        expect.objectContaining({ method: "POST" })
      );
      expect(onCreated).toHaveBeenCalled();
      expect(onClose).toHaveBeenCalled();
    });
  });
});

function setupSampleTasksMock(calls: { url: string; method?: string }[]) {
  const tasks: unknown[] = [];
  global.fetch = vi.fn().mockImplementation((url, opts) => {
    calls.push({ url, method: opts?.method });
    if (url.includes("/api/v1/projects/p-empty/tasks") && (!opts || opts.method === "GET")) {
      return Promise.resolve({ ok: true, status: 200, json: async () => ({ tasks }) });
    }
    if (url.includes("/api/v1/projects/p-empty/tasks") && opts?.method === "POST") {
      const body = JSON.parse(opts.body);
      const newTask = { id: `task-${Date.now()}`, project_id: "p-empty", title: body.title, status: "backlog" };
      tasks.push(newTask);
      return Promise.resolve({ ok: true, status: 201, json: async () => ({ task: newTask }) });
    }
    if (url.includes("/status") && opts?.method === "POST") {
      return Promise.resolve({ ok: true, status: 200, json: async () => ({ task: { status: "running" } }) });
    }
    return Promise.reject(new Error(`Unhandled fetch: ${url}`));
  });
}

describe("UI-04: Sample Tasks Loading", () => {
  beforeEach(() => { vi.restoreAllMocks(); });

  it("AC3: Empty project displays 'Nạp task mẫu' and clicking loads real sample tasks", async () => {
    const calls: { url: string; method?: string }[] = [];
    setupSampleTasksMock(calls);

    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    render(
      <QueryClientProvider client={queryClient}>
        <MemoryRouter><KanbanBoard context={{ projectId: "p-empty" }} /></MemoryRouter>
      </QueryClientProvider>
    );

    const loadSampleBtn = await screen.findByTestId("btn-load-sample-tasks");
    fireEvent.click(loadSampleBtn);

    await waitFor(() => {
      const postCalls = calls.filter((c) => c.url.includes("/api/v1/projects/p-empty/tasks") && c.method === "POST");
      expect(postCalls.length).toBeGreaterThanOrEqual(4);
    });
  });
});

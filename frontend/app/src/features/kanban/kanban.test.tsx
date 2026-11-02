import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { KanbanBoard } from "./KanbanBoard";
import { KanbanHeaderChip } from "./KanbanHeaderChip";
import type { Task } from "../../../../../shared/types";

const mockTasks: Task[] = [
  {
    id: "task-101",
    project_id: "p1",
    title: "Cấu hình Docker Compose",
    description: "Khởi tạo môi trường",
    status: "backlog",
    lead_agent: "",
    worker_agent: "",
    workdir_path: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: "task-102",
    project_id: "p1",
    title: "Sửa lỗi bảo mật cookie",
    description: "Bổ sung cờ HttpOnly",
    status: "running",
    lead_agent: "claude",
    worker_agent: "gemini",
    workdir_path: "worktrees/p1/task-102",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: "task-103",
    project_id: "p1",
    title: "Đánh giá chất lượng mã nguồn",
    description: "Chạy lint và test",
    status: "needs_human",
    lead_agent: "",
    worker_agent: "claude",
    workdir_path: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
];

function renderWithProviders(ui: React.ReactElement, initialEntries = ["/board"]) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={initialEntries}>
        {ui}
      </MemoryRouter>
    </QueryClientProvider>
  );
}

describe("P1-05: Kanban Column Rendering", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("AC1: Renders 5 main columns and places tasks in correct columns", () => {
    renderWithProviders(<KanbanBoard context={{ initialTasks: mockTasks }} />);

    expect(screen.getByTestId("kanban-column-backlog")).toBeDefined();
    expect(screen.getByTestId("kanban-column-running")).toBeDefined();
    expect(screen.getByTestId("kanban-column-needs_human")).toBeDefined();
    expect(screen.getByTestId("kanban-column-in_review")).toBeDefined();
    expect(screen.getByTestId("kanban-column-ready_to_merge")).toBeDefined();

    expect(screen.getByText("Cấu hình Docker Compose")).toBeDefined();
    expect(screen.getByText("Sửa lỗi bảo mật cookie")).toBeDefined();
    expect(screen.getByTestId("empty-column-in_review")).toBeDefined();
  });

  it("AC5: Header chips count running and needs_human tasks", () => {
    renderWithProviders(<KanbanHeaderChip context={{ tasks: mockTasks }} />);

    expect(screen.getByTestId("chip-running")).toBeDefined();
    expect(screen.getByTestId("chip-needs-human")).toBeDefined();
  });
});

describe("P1-05: Kanban Interactions and Detail Panel", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("AC2: Shows error alert when status transition fails", async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 400,
      json: async () => ({
        error: { message: "Không thể chuyển trạng thái" },
      }),
    });

    renderWithProviders(<KanbanBoard context={{ initialTasks: mockTasks }} />);

    const moveBtn = screen.getByLabelText("Chuyển trạng thái cho Cấu hình Docker Compose");
    fireEvent.click(moveBtn);
    fireEvent.change(screen.getByTestId("move-select-task-101"), { target: { value: "running" } });

    await waitFor(() => {
      expect(screen.getByTestId("kanban-error-alert")).toBeDefined();
    });
  });

  it("AC3: Selecting a card opens task detail panel", () => {
    renderWithProviders(<KanbanBoard context={{ initialTasks: mockTasks }} />);

    expect(screen.queryByTestId("task-detail-panel")).toBeNull();
    fireEvent.click(screen.getByTestId("kanban-card-task-102"));
    expect(screen.getByTestId("task-detail-panel")).toBeDefined();

    fireEvent.click(screen.getByLabelText("Đóng"));
    expect(screen.queryByTestId("task-detail-panel")).toBeNull();
  });

  it("AC4: Filters search and toggle completed tasks", () => {
    renderWithProviders(<KanbanBoard context={{ initialTasks: mockTasks }} />);

    expect(screen.queryByTestId("kanban-column-done")).toBeNull();
    fireEvent.click(screen.getByTestId("toggle-completed-checkbox"));
    expect(screen.getByTestId("kanban-column-done")).toBeDefined();

    fireEvent.change(screen.getByTestId("kanban-search-input"), { target: { value: "Docker" } });
    expect(screen.getByText("Cấu hình Docker Compose")).toBeDefined();
    expect(screen.queryByText("Sửa lỗi bảo mật cookie")).toBeNull();
  });
});

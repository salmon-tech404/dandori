import { render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AutoLessonsPanel } from "./AutoLessonsPanel";
import type { AutoLessonsConfig, AutoLessonsRunRecord } from "./types";

const mockConfig: AutoLessonsConfig = {
  project_id: "proj-1",
  workspace_id: "ws-1",
  enabled: true,
  trigger_on_done: true,
  trigger_on_failed: true,
  max_token_budget: 1000,
  min_content_length: 10,
  created_at: "2026-11-02T10:00:00Z",
  updated_at: "2026-11-02T10:00:00Z",
};

const mockRuns: AutoLessonsRunRecord[] = [
  {
    id: "run-rec-1",
    workspace_id: "ws-1",
    project_id: "proj-1",
    task_id: "task-101",
    run_id: "run-55",
    status: "completed",
    lesson_id: "lesson-99",
    reason: null,
    created_at: "2026-11-02T10:00:00Z",
    finished_at: "2026-11-02T10:01:00Z",
  },
];

function handleMockFetch(url: string): Promise<Response> {
  if (url.includes("/auto-lessons/config")) {
    return Promise.resolve({
      ok: true,
      json: async () => mockConfig,
    } as Response);
  }
  if (url.includes("/auto-lessons/runs")) {
    return Promise.resolve({
      ok: true,
      json: async () => mockRuns,
    } as Response);
  }
  return Promise.resolve({ ok: true, json: async () => ({}) } as Response);
}

describe("AutoLessonsPanel", () => {
  beforeEach(() => {
    vi.spyOn(global, "fetch").mockImplementation((url) =>
      handleMockFetch(String(url))
    );
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("hiển thị cấu hình và lịch sử rút bài học", async () => {
    render(<AutoLessonsPanel projectId="proj-1" />);

    await waitFor(() => {
      expect(screen.getByText("Tự động rút bài học")).toBeDefined();
      expect(screen.getByText("task-101")).toBeDefined();
      expect(screen.getByText("completed")).toBeDefined();
    });
  });
});

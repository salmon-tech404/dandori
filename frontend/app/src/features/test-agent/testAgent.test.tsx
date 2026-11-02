import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { contributions } from "./index";
import { TestResultBadge } from "./components/TestResultBadge";
import { TestTaskSection } from "./components/TestTaskSection";
import type { TestAgentRun } from "./types";

const mockPassedRun: TestAgentRun = {
  id: "run-1",
  task_id: "task-1",
  project_id: "prj-1",
  workspace_id: "ws-1",
  run_id: "r-1",
  trigger_run_id: "tr-1",
  status: "passed",
  exit_code: 0,
  summary: "All 12 tests passed",
  flagged_files: "[]",
  created_at: new Date().toISOString(),
  finished_at: new Date().toISOString(),
};

const mockFailedRun: TestAgentRun = {
  id: "run-2",
  task_id: "task-1",
  project_id: "prj-1",
  workspace_id: "ws-1",
  run_id: "r-2",
  trigger_run_id: "tr-2",
  status: "failed",
  exit_code: 1,
  summary: "2 tests failed",
  flagged_files: JSON.stringify(["src/main.rs"]),
  created_at: new Date().toISOString(),
  finished_at: new Date().toISOString(),
};

describe("TestAgent Slot Contribution", () => {
  it("contributes task-section, card-badge, and project-settings slots", () => {
    expect(contributions["test-agent-task-section"]).toBeDefined();
    expect(contributions["test-agent-card-badge"]).toBeDefined();
    expect(contributions["test-agent-project-settings"]).toBeDefined();
  });
});

describe("TestResultBadge", () => {
  it("renders passed badge correctly", () => {
    render(<TestResultBadge initialRuns={[mockPassedRun]} />);
    expect(screen.getByText(/Test: Đạt/)).toBeDefined();
  });

  it("renders failed badge with warning icon for flagged files", () => {
    render(<TestResultBadge initialRuns={[mockFailedRun]} />);
    expect(screen.getByText(/Test: Không đạt/)).toBeDefined();
    expect(screen.getByTestId("warning-icon")).toBeDefined();
  });
});

describe("TestTaskSection", () => {
  it("renders empty state when no results", async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ results: [] }),
    } as Response);

    render(<TestTaskSection taskId="task-empty" />);
    expect(await screen.findByText(/Chưa có kết quả kiểm thử/)).toBeDefined();
  });

  it("renders runs with summary and flagged files", async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ results: [mockFailedRun] }),
    } as Response);

    render(<TestTaskSection taskId="task-1" />);
    expect(await screen.findByText(/2 tests failed/)).toBeDefined();
    expect(screen.getByText("src/main.rs")).toBeDefined();
  });
});

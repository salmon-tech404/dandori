import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { contributions } from "./index";
import { DocTaskSection } from "./components/DocTaskSection";
import type { DocAgentRun } from "./types";

const mockDocRun: DocAgentRun = {
  id: "doc-run-1",
  task_id: "task-1",
  project_id: "prj-1",
  workspace_id: "ws-1",
  run_id: "r-1",
  status: "completed",
  summary: "Updated architecture docs and changelog",
  changed_doc_files: JSON.stringify(["docs/arch.md", "README.md"]),
  flagged_files: JSON.stringify(["crates/core/src/lib.rs"]),
  created_at: new Date().toISOString(),
  finished_at: new Date().toISOString(),
};

describe("DocAgent Slot Contribution", () => {
  it("contributes task-section and project-settings slots", () => {
    expect(contributions["doc-agent-task-section"]).toBeDefined();
    expect(contributions["doc-agent-project-settings"]).toBeDefined();
  });
});

describe("DocTaskSection", () => {
  it("renders empty state when no results", async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => [],
    } as Response);

    render(<DocTaskSection taskId="task-empty" />);
    expect(await screen.findByText(/Chưa có lượt chạy viết tài liệu nào/)).toBeDefined();
  });

  it("renders runs with summary, changed docs and flagged files", async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => [mockDocRun],
    } as Response);

    render(<DocTaskSection taskId="task-1" />);
    expect(await screen.findByText(/Tài liệu tự động/)).toBeDefined();
    expect(screen.getByText(/Updated architecture docs and changelog/)).toBeDefined();
    expect(screen.getByText("docs/arch.md")).toBeDefined();
    expect(screen.getByText("crates/core/src/lib.rs")).toBeDefined();
  });
});

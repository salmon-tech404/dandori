import type { TaskStatus } from "../../../../../shared/types";

export interface KanbanColumnDef {
  id: TaskStatus;
  labelKey: string;
  isTerminal?: boolean;
}

export const MAIN_COLUMNS: KanbanColumnDef[] = [
  { id: "backlog", labelKey: "status_backlog" },
  { id: "running", labelKey: "status_running" },
  { id: "needs_human", labelKey: "status_needs_human" },
  { id: "in_review", labelKey: "status_in_review" },
  { id: "ready_to_merge", labelKey: "status_ready_to_merge" },
];

export const TERMINAL_COLUMNS: KanbanColumnDef[] = [
  { id: "done", labelKey: "status_done", isTerminal: true },
  { id: "failed", labelKey: "status_failed", isTerminal: true },
  { id: "cancelled", labelKey: "status_cancelled", isTerminal: true },
];

export const VALID_TARGET_STATUSES: Record<TaskStatus, TaskStatus[]> = {
  backlog: ["running", "cancelled"],
  running: ["needs_human", "in_review", "failed", "cancelled"],
  needs_human: ["running", "failed", "cancelled"],
  in_review: ["ready_to_merge", "running", "failed", "cancelled"],
  ready_to_merge: ["done", "running", "failed", "cancelled"],
  failed: ["backlog", "cancelled"],
  cancelled: ["backlog"],
  done: [],
};

export interface KanbanFilterState {
  search: string;
  agent: string;
  showCompleted: boolean;
}

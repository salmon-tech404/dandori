import React from "react";
import { t } from "../../core/i18n";
import { useKanban } from "./useKanban";
import type { Task } from "../../../../../shared/types";

interface Props {
  context?: {
    tasks?: Task[];
    projectId?: string;
  };
}

const runningChipStyle: React.CSSProperties = {
  height: "var(--chip-height, 24px)",
  padding: "0 8px",
  borderRadius: "var(--radius-pill, 12px)",
  backgroundColor: "var(--grb)",
  color: "var(--gr)",
  fontSize: "var(--fs-12, 12px)",
  fontWeight: 600,
  display: "inline-flex",
  alignItems: "center",
};

const needsHumanChipStyle: React.CSSProperties = {
  ...runningChipStyle,
  backgroundColor: "var(--orb)",
  color: "var(--or)",
};

export function KanbanHeaderChip({ context }: Props) {
  const { tasks: fetchedTasks } = useKanban(context?.projectId);
  const tasks = context?.tasks || fetchedTasks || [];

  const runningCount = tasks.filter((t) => t.status === "running").length;
  const needsHumanCount = tasks.filter((t) => t.status === "needs_human").length;

  return (
    <div style={{ display: "inline-flex", gap: "8px", alignItems: "center" }}>
      {runningCount > 0 && (
        <span data-testid="chip-running" style={runningChipStyle}>
          {t("kanban_chip_running", { count: runningCount })}
        </span>
      )}
      {needsHumanCount > 0 && (
        <span data-testid="chip-needs-human" style={needsHumanChipStyle}>
          {t("kanban_chip_needs_human", { count: needsHumanCount })}
        </span>
      )}
    </div>
  );
}

import React from "react";
import { X } from "lucide-react";
import { t } from "../../core/i18n";
import { Slot } from "../../core/Slot";
import type { Task, TaskStatus } from "../../../../../shared/types";
import { StatusBadge } from "../../ui/components/StatusBadge";

interface Props {
  task: Task;
  onClose: () => void;
}

const panelStyle: React.CSSProperties = {
  width: "420px",
  minWidth: "380px",
  background: "var(--sf)",
  borderLeft: "1px solid var(--ln)",
  padding: "20px",
  display: "flex",
  flexDirection: "column",
  gap: "16px",
  overflowY: "auto",
};

const headerStyle: React.CSSProperties = {
  display: "flex",
  justifyContent: "space-between",
  alignItems: "flex-start",
  gap: "12px",
};

const closeBtnStyle: React.CSSProperties = {
  background: "none",
  border: "none",
  fontSize: "13px",
  color: "var(--mu)",
  cursor: "pointer",
  padding: "4px 8px",
};

export function TaskDetailPanel({ task, onClose }: Props) {
  const context = { taskId: task.id, task };

  return (
    <aside data-testid="task-detail-panel" style={panelStyle} aria-label="Task Detail Panel">
      <div style={headerStyle}>
        <div>
          <span style={{ fontSize: "11px", color: "var(--mu)", fontFamily: "var(--font-mono)" }}>
            Task #{task.id}
          </span>
          <h2 style={{ margin: "4px 0 8px 0", fontSize: "16px", fontWeight: 600 }}>{task.title}</h2>
          <StatusBadge status={task.status as TaskStatus} />
        </div>
        <button
          type="button"
          onClick={onClose}
          style={closeBtnStyle}
          aria-label={t("kanban_panel_close")}
        >
          <X size={14} />
          <span>{t("kanban_panel_close")}</span>
        </button>
      </div>

      {task.description && (
        <div style={{ fontSize: "13px", color: "var(--fg)", lineHeight: "1.4" }}>
          {task.description}
        </div>
      )}

      <div style={{ display: "flex", flexDirection: "column", gap: "12px", flex: 1 }}>
        <Slot name="board.task-panel.section" context={context} />
        <Slot name="board.task-panel.action" context={context} />
      </div>
    </aside>
  );
}

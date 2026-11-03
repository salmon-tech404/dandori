import type { CSSProperties } from "react";
import { X } from "lucide-react";
import { t } from "../../core/i18n";
import { Slot } from "../../core/Slot";
import type { Task, TaskStatus } from "../../../../../shared/types";
import { StatusBadge } from "../../ui/components/StatusBadge";
import { IconButton } from "../../ui/components/IconButton";
import { TaskDispatchCard } from "./TaskDispatchCard";
import { TaskLiveRunSection } from "./TaskLiveRunSection";

interface Props {
  task: Task;
  onClose: () => void;
  onTaskUpdated?: () => void;
}

const panelStyle: CSSProperties = {
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

export function TaskDetailPanel({ task, onClose, onTaskUpdated }: Props) {
  const context = { taskId: task.id, task };
  const isRunning = task.status === "running";

  return (
    <aside data-testid="task-detail-panel" style={panelStyle} aria-label={t("task_detail_panel_label")}>
      <PanelHeader task={task} onClose={onClose} />
      {task.description && (
        <div style={{ fontSize: "13px", color: "var(--fg)", lineHeight: "1.4" }}>
          {task.description}
        </div>
      )}
      {isRunning ? (
        <TaskLiveRunSection taskId={task.id} onRunFinished={onTaskUpdated} />
      ) : (
        <TaskDispatchCard task={task} onTaskUpdated={onTaskUpdated} />
      )}
      <div style={{ display: "flex", flexDirection: "column", gap: "12px", flex: 1 }}>
        <Slot name="board.task-panel.tab" asTabs={true} context={context} />
        <Slot name="board.task-panel.section" context={context} />
        <Slot name="board.task-panel.action" context={context} />
      </div>
    </aside>
  );
}

function PanelHeader({ task, onClose }: { task: Task; onClose: () => void }) {
  return (
    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: "12px" }}>
      <div>
        <span style={{ fontSize: "11px", color: "var(--mu)", fontFamily: "var(--font-mono)" }}>
          {t("task_number_prefix")}{task.id}
        </span>
        <h2 style={{ margin: "4px 0 8px 0", fontSize: "16px", fontWeight: 600 }}>{task.title}</h2>
        <StatusBadge status={task.status as TaskStatus} />
      </div>
      <IconButton variant="ghost" size="sm" type="button" aria-label={t("kanban_panel_close")} onClick={onClose} icon={<X size={14} />} />
    </div>
  );
}

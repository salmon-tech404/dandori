import React, { useState } from "react";
import { Bot } from "lucide-react";
import { t } from "../../core/i18n";
import type { Task, TaskStatus } from "../../../../../shared/types";
import { VALID_TARGET_STATUSES } from "./types";

interface Props {
  task: Task;
  isSelected: boolean;
  onSelect: () => void;
  onMove: (targetStatus: TaskStatus) => void;
}

const cardStyle: React.CSSProperties = {
  padding: "12px",
  background: "var(--sf)",
  border: "1px solid var(--ln)",
  borderRadius: "8px",
  display: "flex",
  flexDirection: "column",
  gap: "8px",
  cursor: "grab",
};

const selectedCardStyle: React.CSSProperties = {
  ...cardStyle,
  border: "2px solid var(--ac)",
  boxShadow: "0 2px 8px rgba(0,0,0,0.08)",
};

const branchStyle: React.CSSProperties = {
  fontFamily: "var(--font-mono)",
  fontSize: "11px",
  color: "var(--mu)",
  background: "var(--sf2)",
  padding: "2px 6px",
  borderRadius: "4px",
  alignSelf: "flex-start",
};

const menuSelectStyle: React.CSSProperties = {
  fontSize: "11px",
  padding: "2px 6px",
  background: "var(--sf2)",
  border: "1px solid var(--ln)",
  borderRadius: "4px",
  color: "var(--fg)",
  cursor: "pointer",
};

function formatAgentLabel(task: Task): string | null {
  if (task.lead_agent && task.worker_agent) {
    return `${task.lead_agent} → ${task.worker_agent}`;
  }
  return task.worker_agent || task.lead_agent || null;
}

function CardHeader({
  title,
  hasTargets,
  showMenu,
  onToggleMenu,
}: {
  title: string;
  hasTargets: boolean;
  showMenu: boolean;
  onToggleMenu: () => void;
}) {
  return (
    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: "8px" }}>
      <span style={{ fontWeight: 600, fontSize: "13px", lineHeight: "1.3" }}>{title}</span>
      {hasTargets && (
        <button
          type="button"
          aria-label={`Chuyển trạng thái cho ${title}`}
          style={{ border: "none", background: "none", cursor: "pointer", color: "var(--mu)", fontSize: "11px" }}
          onClick={(e) => {
            e.stopPropagation();
            onToggleMenu();
          }}
        >
          {showMenu ? "—" : t("kanban_move_to")}
        </button>
      )}
    </div>
  );
}

function MoveDropdown({
  taskId,
  validTargets,
  onSelect,
}: {
  taskId: string;
  validTargets: TaskStatus[];
  onSelect: (target: TaskStatus) => void;
}) {
  return (
    <div style={{ padding: "4px 0" }} onClick={(e) => e.stopPropagation()}>
      <select
        data-testid={`move-select-${taskId}`}
        defaultValue=""
        onChange={(e) => onSelect(e.target.value as TaskStatus)}
        style={menuSelectStyle}
      >
        <option value="" disabled>{t("kanban_move_to")}</option>
        {validTargets.map((target) => (
          <option key={target} value={target}>{t(`status_${target}`)}</option>
        ))}
      </select>
    </div>
  );
}

function CardFooter({ workdirPath, agentLabel }: { workdirPath: string | null; agentLabel: string | null }) {
  if (!workdirPath && !agentLabel) return null;
  return (
    <>
      {workdirPath && <span style={branchStyle}>{workdirPath.split(/[\\/]/).pop()}</span>}
      {agentLabel && (
        <div style={{ display: "inline-flex", alignItems: "center", gap: "4px", fontSize: "11px", color: "var(--mu)" }}>
          <Bot size={13} />
          <span>{agentLabel}</span>
        </div>
      )}
    </>
  );
}

export function KanbanCard({ task, isSelected, onSelect, onMove }: Props) {
  const [showMoveMenu, setShowMoveMenu] = useState(false);
  const currentStatus = task.status as TaskStatus;
  const validTargets = VALID_TARGET_STATUSES[currentStatus] || [];
  const agentLabel = formatAgentLabel(task);

  const handleSelectMove = (target: TaskStatus) => {
    onMove(target);
    setShowMoveMenu(false);
  };

  return (
    <div
      role="button"
      tabIndex={0}
      data-testid={`kanban-card-${task.id}`}
      style={isSelected ? selectedCardStyle : cardStyle}
      draggable
      onDragStart={(e) => e.dataTransfer.setData("text/plain", task.id)}
      onClick={onSelect}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onSelect();
        }
      }}
    >
      <CardHeader
        title={task.title}
        hasTargets={validTargets.length > 0}
        showMenu={showMoveMenu}
        onToggleMenu={() => setShowMoveMenu(!showMoveMenu)}
      />
      {showMoveMenu && (
        <MoveDropdown taskId={task.id} validTargets={validTargets} onSelect={handleSelectMove} />
      )}
      <CardFooter workdirPath={task.workdir_path} agentLabel={agentLabel} />
    </div>
  );
}

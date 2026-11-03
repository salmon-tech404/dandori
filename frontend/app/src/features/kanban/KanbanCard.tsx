import React, { useRef, useState } from "react";
import { t } from "../../core/i18n";
import type { Task, TaskStatus } from "../../../../../shared/types";
import { VALID_TARGET_STATUSES } from "./types";
import { useDismiss } from "../../ui/components/useDismiss";
import { Button } from "../../ui/components/Button";
import { Select } from "../../ui/components/Select";
import { Chip } from "../../ui/components/Chip";

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

function ProviderBadge({ provider }: { provider?: string | null }) {
  const p = (provider || "claude").toLowerCase();
  let letter = "C";
  let bg = "rgba(217, 119, 6, 0.15)";
  let fg = "var(--or)";
  if (p.includes("gemini")) {
    letter = "G";
    bg = "rgba(37, 99, 235, 0.15)";
    fg = "var(--bl)";
  } else if (p.includes("codex") || p.includes("openai")) {
    letter = "K";
    bg = "rgba(13, 148, 136, 0.15)";
    fg = "var(--tl)";
  } else if (p.includes("fake")) {
    letter = "F";
    bg = "rgba(100, 116, 139, 0.15)";
    fg = "var(--mu)";
  }
  return (
    <span
      className="ag"
      style={{
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        flex: "0 0 20px",
        width: 20,
        height: 20,
        borderRadius: "50%",
        fontSize: "11px",
        fontWeight: 700,
        backgroundColor: bg,
        color: fg,
      }}
    >
      {letter}
    </span>
  );
}

function CardProgressBar({ status }: { status: TaskStatus }) {
  if (status !== "running" && status !== "in_review") return null;
  const width = status === "running" ? "65%" : "90%";
  const chipLabel = status === "running" ? "còn ~14 phút" : "3% hạn mức";
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 4, marginTop: 2 }}>
      <div style={{ width: "100%", height: 4, borderRadius: "var(--radius-pill)", backgroundColor: "var(--sf2)", overflow: "hidden" }}>
        <div style={{ height: "100%", borderRadius: "var(--radius-pill)", backgroundColor: "var(--bl)", width }} />
      </div>
      <div style={{ display: "flex", gap: 6 }}>
        <Chip variant="default">{chipLabel}</Chip>
      </div>
    </div>
  );
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
        <Button variant="ghost" size="sm" type="button" aria-label={t("kanban_move_to_aria", { title })}
          style={{ border: "none", background: "none", cursor: "pointer", color: "var(--mu)", fontSize: "11px" }}
          onClick={(e) => {
            e.stopPropagation();
            onToggleMenu();
          }}
        >
          {showMenu ? "—" : t("kanban_move_to")}
        </Button>
      )}
    </div>
  );
}

function MoveDropdown({
  taskId,
  validTargets,
  onSelect,
  onClose,
}: {
  taskId: string;
  validTargets: TaskStatus[];
  onSelect: (target: TaskStatus) => void;
  onClose: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);
  useDismiss({ isOpen: true, onDismiss: onClose, ref });

  return (
    <div ref={ref} style={{ padding: "4px 0" }} onClick={(e) => e.stopPropagation()}>
      <Select data-testid={`move-select-${taskId}`}
        defaultValue=""
        onChange={(e) => onSelect(e.target.value as TaskStatus)}
        style={menuSelectStyle}
      >
        <option value="" disabled>{t("kanban_move_to")}</option>
        {validTargets.map((target) => (
          <option key={target} value={target}>{t(`status_${target}`)}</option>
        ))}
      </Select>
    </div>
  );
}

function CardFooter({
  workdirPath,
  agentLabel,
  agentProvider,
  status,
}: {
  workdirPath: string | null;
  agentLabel: string | null;
  agentProvider?: string | null;
  status: TaskStatus;
}) {
  if (!workdirPath && !agentLabel) return null;
  return (
    <>
      {workdirPath && <span style={branchStyle}>{workdirPath.split(/[\\/]/).pop()}</span>}
      {agentLabel && (
        <div style={{ display: "inline-flex", alignItems: "center", gap: 6, fontSize: "11px", color: "var(--mu)" }}>
          <ProviderBadge provider={agentProvider} />
          <span>{agentLabel}</span>
        </div>
      )}
      <CardProgressBar status={status} />
    </>
  );
}

function handleCardKeyDown(e: React.KeyboardEvent, onSelect: () => void) {
  if (e.key === "Enter" || e.key === " ") {
    e.preventDefault();
    onSelect();
  }
}

function useCardMove(onMove: (targetStatus: TaskStatus) => void) {
  const [showMoveMenu, setShowMoveMenu] = useState(false);
  const handleSelectMove = (target: TaskStatus) => {
    onMove(target);
    setShowMoveMenu(false);
  };
  return { showMoveMenu, setShowMoveMenu, handleSelectMove };
}

export function KanbanCard({ task, isSelected, onSelect, onMove }: Props) {
  const { showMoveMenu, setShowMoveMenu, handleSelectMove } = useCardMove(onMove);
  const currentStatus = task.status as TaskStatus;
  const validTargets = VALID_TARGET_STATUSES[currentStatus] || [];
  const agentLabel = formatAgentLabel(task);

  return (
    <div
      role="button"
      tabIndex={0}
      data-testid={`kanban-card-${task.id}`}
      style={isSelected ? selectedCardStyle : cardStyle}
      draggable
      onDragStart={(e) => e.dataTransfer.setData("text/plain", task.id)}
      onClick={onSelect}
      onKeyDown={(e) => handleCardKeyDown(e, onSelect)}
    >
      <CardHeader
        title={task.title}
        hasTargets={validTargets.length > 0}
        showMenu={showMoveMenu}
        onToggleMenu={() => setShowMoveMenu(!showMoveMenu)}
      />
      {showMoveMenu && (
        <MoveDropdown taskId={task.id} validTargets={validTargets} onSelect={handleSelectMove} onClose={() => setShowMoveMenu(false)} />
      )}
      <CardFooter
        workdirPath={task.workdir_path}
        agentLabel={agentLabel}
        agentProvider={task.worker_agent || task.lead_agent}
        status={currentStatus}
      />
    </div>
  );
}

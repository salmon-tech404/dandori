import React, { useState } from "react";
import { t } from "../../core/i18n";
import type { Task, TaskStatus } from "../../../../../shared/types";
import { KanbanCard } from "./KanbanCard";

interface Props {
  id: TaskStatus;
  title: string;
  tasks: Task[];
  selectedTaskId: string | null;
  onSelectTask: (task: Task) => void;
  onMoveTask: (taskId: string, targetStatus: TaskStatus) => void;
  onQuickCreateTask?: (title: string) => void;
}

const columnStyle: React.CSSProperties = {
  flex: "1",
  minWidth: "260px",
  maxWidth: "320px",
  background: "var(--sf2)",
  borderRadius: "10px",
  padding: "12px",
  display: "flex",
  flexDirection: "column",
  gap: "10px",
};

const countBadgeStyle: React.CSSProperties = {
  fontSize: "12px",
  fontWeight: 600,
  padding: "2px 8px",
  borderRadius: "10px",
  background: "var(--sf)",
  color: "var(--mu)",
};

const quickInputStyle: React.CSSProperties = {
  width: "100%",
  padding: "6px 8px",
  borderRadius: "6px",
  border: "1px solid var(--ln)",
  fontSize: "12px",
  boxSizing: "border-box",
  background: "var(--sf)",
  color: "var(--fg)",
};

function QuickCreateForm({ onQuickCreate }: { onQuickCreate: (title: string) => void }) {
  const [quickTitle, setQuickTitle] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (quickTitle.trim()) {
      onQuickCreate(quickTitle.trim());
      setQuickTitle("");
    }
  };

  return (
    <form onSubmit={handleSubmit} style={{ marginBottom: "4px" }}>
      <input
        data-testid="quick-create-input"
        type="text"
        placeholder={t("kanban_quick_create_placeholder")}
        value={quickTitle}
        onChange={(e) => setQuickTitle(e.target.value)}
        style={quickInputStyle}
      />
    </form>
  );
}

function ColumnTaskList({
  id,
  tasks,
  selectedTaskId,
  onSelectTask,
  onMoveTask,
}: {
  id: TaskStatus;
  tasks: Task[];
  selectedTaskId: string | null;
  onSelectTask: (task: Task) => void;
  onMoveTask: (taskId: string, targetStatus: TaskStatus) => void;
}) {
  if (tasks.length === 0) {
    return (
      <div data-testid={`empty-column-${id}`} style={{ padding: "24px 12px", textAlign: "center", color: "var(--mu)", fontSize: "12px", fontStyle: "italic" }}>
        {t("kanban_empty_column")}
      </div>
    );
  }
  return (
    <>
      {tasks.map((task) => (
        <KanbanCard
          key={task.id}
          task={task}
          isSelected={task.id === selectedTaskId}
          onSelect={() => onSelectTask(task)}
          onMove={(targetStatus) => onMoveTask(task.id, targetStatus)}
        />
      ))}
    </>
  );
}

function ColumnHeader({ title, count }: { title: string; count: number }) {
  return (
    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", paddingBottom: "8px", borderBottom: "1px solid var(--ln)" }}>
      <span style={{ fontWeight: 600, fontSize: "14px" }}>{title}</span>
      <span style={countBadgeStyle}>{count}</span>
    </div>
  );
}

export function KanbanColumn({
  id,
  title,
  tasks,
  selectedTaskId,
  onSelectTask,
  onMoveTask,
  onQuickCreateTask,
}: Props) {
  const isBacklog = id === "backlog";

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    const taskId = e.dataTransfer.getData("text/plain");
    if (taskId) {
      onMoveTask(taskId, id);
    }
  };

  return (
    <div
      data-testid={`kanban-column-${id}`}
      style={columnStyle}
      onDragOver={(e) => e.preventDefault()}
      onDrop={handleDrop}
    >
      <ColumnHeader title={title} count={tasks.length} />

      {isBacklog && onQuickCreateTask && (
        <QuickCreateForm onQuickCreate={onQuickCreateTask} />
      )}

      <div style={{ display: "flex", flexDirection: "column", gap: "8px", flex: 1 }}>
        <ColumnTaskList
          id={id}
          tasks={tasks}
          selectedTaskId={selectedTaskId}
          onSelectTask={onSelectTask}
          onMoveTask={onMoveTask}
        />
      </div>
    </div>
  );
}

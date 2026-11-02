import React from "react";
import { X } from "lucide-react";
import { t } from "../../core/i18n";
import type { Task, TaskStatus } from "../../../../../shared/types";
import { MAIN_COLUMNS, TERMINAL_COLUMNS, type KanbanColumnDef } from "./types";
import { useKanban, useKanbanFilters } from "./useKanban";
import { KanbanColumn } from "./KanbanColumn";
import { TaskDetailPanel } from "./TaskDetailPanel";

interface Props {
  context?: {
    projectId?: string;
    initialTasks?: Task[];
  };
}

const boardContainerStyle: React.CSSProperties = {
  display: "flex",
  flexDirection: "row",
  height: "calc(100vh - 120px)",
  gap: "16px",
  overflow: "hidden",
};

const toolbarStyle: React.CSSProperties = {
  display: "flex",
  justifyContent: "space-between",
  alignItems: "center",
  flexWrap: "wrap",
  gap: "12px",
  paddingBottom: "8px",
};

const errorAlertStyle: React.CSSProperties = {
  padding: "8px 12px",
  background: "var(--rdb)",
  color: "var(--rd)",
  borderRadius: "6px",
  fontSize: "13px",
  display: "flex",
  justifyContent: "space-between",
  alignItems: "center",
};

function filterTasks(tasks: Task[], search: string, agent: string): Task[] {
  return tasks.filter((task) => {
    if (search && !task.title.toLowerCase().includes(search.toLowerCase())) {
      return false;
    }
    if (agent && task.worker_agent !== agent && task.lead_agent !== agent) {
      return false;
    }
    return true;
  });
}

function FilterBar({
  filters,
  onChange,
}: {
  filters: { search: string; agent: string; showCompleted: boolean };
  onChange: (next: Partial<{ search: string; agent: string; showCompleted: boolean }>) => void;
}) {
  return (
    <div style={toolbarStyle}>
      <span style={{ fontSize: "12px", color: "var(--mu)" }}>{t("kanban_hint_click_card")}</span>
      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <input
          data-testid="kanban-search-input"
          type="text"
          placeholder={t("kanban_search_placeholder")}
          value={filters.search}
          onChange={(e) => onChange({ search: e.target.value })}
          style={{ padding: "4px 8px", borderRadius: "6px", border: "1px solid var(--ln)", fontSize: "12px" }}
        />
        <label style={{ fontSize: "12px", display: "inline-flex", alignItems: "center", gap: "4px", cursor: "pointer" }}>
          <input
            data-testid="toggle-completed-checkbox"
            type="checkbox"
            checked={filters.showCompleted}
            onChange={(e) => onChange({ showCompleted: e.target.checked })}
          />
          {t("kanban_filter_show_completed")}
        </label>
      </div>
    </div>
  );
}

function ColumnsList({
  columns,
  tasks,
  selectedTaskId,
  onSelectTask,
  onMoveTask,
  onQuickCreateTask,
}: {
  columns: KanbanColumnDef[];
  tasks: Task[];
  selectedTaskId: string | null;
  onSelectTask: (t: Task) => void;
  onMoveTask: (taskId: string, targetStatus: TaskStatus) => void;
  onQuickCreateTask: (title: string) => void;
}) {
  return (
    <div style={{ display: "flex", gap: "14px", overflowX: "auto", paddingBottom: "16px", flex: 1 }}>
      {columns.map((col) => (
        <KanbanColumn
          key={col.id}
          id={col.id}
          title={t(col.labelKey)}
          tasks={tasks.filter((t) => t.status === col.id)}
          selectedTaskId={selectedTaskId}
          onSelectTask={onSelectTask}
          onMoveTask={onMoveTask}
          onQuickCreateTask={onQuickCreateTask}
        />
      ))}
    </div>
  );
}

export function KanbanBoard({ context }: Props) {
  const { filters, setFilters } = useKanbanFilters();
  const {
    tasks,
    selectedTask,
    setSelectedTask,
    errorMessage,
    setErrorMessage,
    moveTask,
    quickCreateTask,
  } = useKanban(context?.projectId, context?.initialTasks);

  const activeColumns = filters.showCompleted ? [...MAIN_COLUMNS, ...TERMINAL_COLUMNS] : MAIN_COLUMNS;
  const filteredTasks = filterTasks(tasks, filters.search, filters.agent);

  return (
    <div style={boardContainerStyle}>
      <div style={{ flex: 1, display: "flex", flexDirection: "column", gap: "12px", minWidth: 0 }}>
        <FilterBar filters={filters} onChange={setFilters} />
        {errorMessage && (
          <div data-testid="kanban-error-alert" style={errorAlertStyle}>
            <span>{errorMessage}</span>
            <button type="button" onClick={() => setErrorMessage(null)} style={{ background: "none", border: "none", color: "inherit", cursor: "pointer" }}>
              <X size={14} />
            </button>
          </div>
        )}
        <ColumnsList
          columns={activeColumns}
          tasks={filteredTasks}
          selectedTaskId={selectedTask?.id || null}
          onSelectTask={setSelectedTask}
          onMoveTask={(taskId, targetStatus) => moveTask({ taskId, status: targetStatus })}
          onQuickCreateTask={quickCreateTask}
        />
      </div>
      {selectedTask && (
        <TaskDetailPanel task={selectedTask} onClose={() => setSelectedTask(null)} />
      )}
    </div>
  );
}

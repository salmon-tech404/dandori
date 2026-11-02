import React, { useContext, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { X } from "lucide-react";
import { t } from "../../core/i18n";
import { ProjectContext } from "../../core/project";
import { Button } from "../../ui/components/Button";
import type { Task, TaskStatus } from "../../../../../shared/types";
import { MAIN_COLUMNS, TERMINAL_COLUMNS, type KanbanColumnDef } from "./types";
import { useKanban, useKanbanFilters } from "./useKanban";
import { createSampleTasks } from "./sampleTasks";
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

function EmptyBoardBanner({
  loading,
  onLoad,
}: {
  loading: boolean;
  onLoad: () => void;
}) {
  return (
    <div
      data-testid="kanban-empty-banner"
      style={{
        padding: "12px 16px",
        backgroundColor: "var(--sf)",
        border: "1px dashed var(--ln)",
        borderRadius: "8px",
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        gap: "12px",
      }}
    >
      <span style={{ fontSize: "13px", color: "var(--mu)" }}>
        {t("kanban_empty_project_tasks")}
      </span>
      <Button
        variant="primary"
        data-testid="btn-load-sample-tasks"
        disabled={loading}
        onClick={onLoad}
      >
        {loading ? t("btn_loading_sample_tasks") : t("btn_load_sample_tasks")}
      </Button>
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

function useSampleLoader(projectId: string, refetch: () => void) {
  const [loading, setLoading] = useState(false);
  const queryClient = useQueryClient();

  const handleLoadSample = async () => {
    setLoading(true);
    try {
      await createSampleTasks(projectId);
      queryClient.setQueryData(["is-sample-data", projectId], true);
      queryClient.invalidateQueries({ queryKey: ["kanban-tasks", projectId] });
      refetch();
    } finally {
      setLoading(false);
    }
  };

  return { loading, handleLoadSample };
}

function BoardAlerts({
  errorMessage,
  onClearError,
}: {
  errorMessage: string | null;
  onClearError: () => void;
}) {
  if (!errorMessage) return null;
  return (
    <div data-testid="kanban-error-alert" style={errorAlertStyle}>
      <span>{errorMessage}</span>
      <button type="button" onClick={onClearError} style={{ background: "none", border: "none", color: "inherit", cursor: "pointer" }}>
        <X size={14} />
      </button>
    </div>
  );
}

function BoardContent({
  projectId,
  tasks,
  errorMessage,
  onClearError,
  columns,
  selectedTaskId,
  onSelectTask,
  onMoveTask,
  onQuickCreate,
  onRefetch,
}: {
  projectId: string;
  tasks: Task[];
  errorMessage: string | null;
  onClearError: () => void;
  columns: KanbanColumnDef[];
  selectedTaskId: string | null;
  onSelectTask: (t: Task) => void;
  onMoveTask: (id: string, s: TaskStatus) => void;
  onQuickCreate: (title: string) => void;
  onRefetch: () => void;
}) {
  const { loading, handleLoadSample } = useSampleLoader(projectId, onRefetch);

  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column", gap: "12px", minWidth: 0 }}>
      {tasks.length === 0 && <EmptyBoardBanner loading={loading} onLoad={handleLoadSample} />}
      <BoardAlerts errorMessage={errorMessage} onClearError={onClearError} />
      <ColumnsList
        columns={columns}
        tasks={tasks}
        selectedTaskId={selectedTaskId}
        onSelectTask={onSelectTask}
        onMoveTask={onMoveTask}
        onQuickCreateTask={onQuickCreate}
      />
    </div>
  );
}

function useBoardData(context?: { projectId?: string; initialTasks?: Task[] }) {
  const projectContext = useContext(ProjectContext);
  const projectId = context?.projectId || projectContext?.currentProject?.id || "p-default";
  const { filters, setFilters } = useKanbanFilters();
  const kanban = useKanban(projectId, context?.initialTasks);
  const activeColumns = filters.showCompleted ? [...MAIN_COLUMNS, ...TERMINAL_COLUMNS] : MAIN_COLUMNS;
  const filteredTasks = filterTasks(kanban.tasks, filters.search, filters.agent);
  return { projectId, filters, setFilters, kanban, activeColumns, filteredTasks };
}

export function KanbanBoard({ context }: Props) {
  const { projectId, filters, setFilters, kanban, activeColumns, filteredTasks } = useBoardData(context);

  return (
    <div style={boardContainerStyle}>
      <div style={{ flex: 1, display: "flex", flexDirection: "column", gap: "12px", minWidth: 0 }}>
        <FilterBar filters={filters} onChange={setFilters} />
        <BoardContent
          projectId={projectId}
          tasks={filteredTasks}
          errorMessage={kanban.errorMessage}
          onClearError={() => kanban.setErrorMessage(null)}
          columns={activeColumns}
          selectedTaskId={kanban.selectedTask?.id || null}
          onSelectTask={kanban.setSelectedTask}
          onMoveTask={(id, s) => kanban.moveTask({ taskId: id, status: s })}
          onQuickCreate={kanban.quickCreateTask}
          onRefetch={kanban.refetch}
        />
      </div>
      {kanban.selectedTask && (
        <TaskDetailPanel
          task={kanban.selectedTask}
          onClose={() => kanban.setSelectedTask(null)}
          onTaskUpdated={kanban.refetch}
        />
      )}
    </div>
  );
}

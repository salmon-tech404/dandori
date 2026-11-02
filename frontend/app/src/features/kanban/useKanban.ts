import { useState, useCallback } from "react";
import { useSearchParams } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import type { Task, TaskStatus } from "../../../../../shared/types";
import type { KanbanFilterState } from "./types";

export async function fetchProjectTasks(projectId: string): Promise<Task[]> {
  const res = await fetch(`/api/v1/projects/${projectId}/tasks`);
  if (!res.ok) {
    throw new Error(`Lỗi tải danh sách task: ${res.status}`);
  }
  const data = await res.json();
  return data.tasks || [];
}

export async function updateTaskStatusApi(taskId: string, targetStatus: TaskStatus): Promise<Task> {
  const res = await fetch(`/api/v1/tasks/${taskId}/status`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ status: targetStatus }),
  });
  if (!res.ok) {
    const data = await res.json().catch(() => ({}));
    const message = data.error?.message || `Chuyển trạng thái thất bại: ${res.status}`;
    throw new Error(message);
  }
  const data = await res.json();
  return data.task;
}

export async function quickCreateTaskApi(projectId: string, title: string): Promise<Task> {
  const res = await fetch(`/api/v1/projects/${projectId}/tasks`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ title, description: title }),
  });
  if (!res.ok) {
    const data = await res.json().catch(() => ({}));
    const message = data.error?.message || `Tạo task thất bại: ${res.status}`;
    throw new Error(message);
  }
  const data = await res.json();
  return data.task;
}

export function useKanbanFilters() {
  const [searchParams, setSearchParams] = useSearchParams();

  const filters: KanbanFilterState = {
    search: searchParams.get("search") || "",
    agent: searchParams.get("agent") || "",
    showCompleted: searchParams.get("show_done") === "true",
  };

  const setFilters = useCallback((next: Partial<KanbanFilterState>) => {
    setSearchParams((prev) => {
      const sp = new URLSearchParams(prev);
      if (next.search !== undefined) {
        if (next.search) sp.set("search", next.search);
        else sp.delete("search");
      }
      if (next.agent !== undefined) {
        if (next.agent) sp.set("agent", next.agent);
        else sp.delete("agent");
      }
      if (next.showCompleted !== undefined) {
        if (next.showCompleted) sp.set("show_done", "true");
        else sp.delete("show_done");
      }
      return sp;
    });
  }, [setSearchParams]);

  return { filters, setFilters };
}

export function useKanban(projectId: string = "p-default", initialTasks: Task[] = []) {
  const queryClient = useQueryClient();
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [selectedTask, setSelectedTask] = useState<Task | null>(null);

  const { data: tasks = initialTasks, refetch } = useQuery<Task[]>({
    queryKey: ["kanban-tasks", projectId],
    queryFn: () => fetchProjectTasks(projectId),
    initialData: initialTasks.length > 0 ? initialTasks : undefined,
  });

  const moveTaskMutation = useMutation({
    mutationFn: ({ taskId, status }: { taskId: string; status: TaskStatus }) =>
      updateTaskStatusApi(taskId, status),
    onSuccess: () => {
      setErrorMessage(null);
      queryClient.invalidateQueries({ queryKey: ["kanban-tasks", projectId] });
    },
    onError: (err: Error) => {
      setErrorMessage(err.message);
    },
  });

  const createTaskMutation = useMutation({
    mutationFn: (title: string) => quickCreateTaskApi(projectId, title),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["kanban-tasks", projectId] });
    },
  });

  return {
    tasks,
    selectedTask,
    setSelectedTask,
    errorMessage,
    setErrorMessage,
    refetch,
    moveTask: moveTaskMutation.mutate,
    quickCreateTask: createTaskMutation.mutate,
  };
}

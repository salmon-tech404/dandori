import { useState, useEffect, useCallback } from "react";
import type { WorktreeStatus, CleanupResponse } from "./types";

export async function fetchWorktreeStatus(taskId: string): Promise<WorktreeStatus> {
  const resp = await fetch(`/api/v1/m/worktree/tasks/${taskId}/worktree`, {
    credentials: "same-origin",
  });
  if (!resp.ok) {
    throw new Error(`Lỗi tải trạng thái worktree: ${resp.status}`);
  }
  return resp.json();
}

export async function executeCleanup(taskId: string, force: boolean): Promise<CleanupResponse> {
  const resp = await fetch(`/api/v1/m/worktree/tasks/${taskId}/cleanup`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    credentials: "same-origin",
    body: JSON.stringify({ force }),
  });
  const data = await resp.json();
  if (!resp.ok) {
    throw new Error(data?.error?.message || "Dọn dẹp worktree thất bại");
  }
  return data;
}

export function useWorktree(taskId?: string) {
  const [status, setStatus] = useState<WorktreeStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!taskId) return;
    setLoading(true);
    setError(null);
    try {
      const data = await fetchWorktreeStatus(taskId);
      setStatus(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [taskId]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { status, loading, error, refresh };
}

import { useState, useCallback } from "react";
import { RoleDefaultItem, TaskAssignment } from "./types";

export function useProjectDefaults(projectId: string) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchDefaults = useCallback(async () => {
    setLoading(true);
    try {
      const res = await fetch(`/api/v1/m/lead-worker/projects/${projectId}/defaults`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      return (data.defaults || []) as RoleDefaultItem[];
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return [];
    } finally {
      setLoading(false);
    }
  }, [projectId]);

  const saveDefaults = useCallback(async (defaults: RoleDefaultItem[]) => {
    setLoading(true);
    try {
      const res = await fetch(`/api/v1/m/lead-worker/projects/${projectId}/defaults`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ defaults }),
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      return true;
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return false;
    } finally {
      setLoading(false);
    }
  }, [projectId]);

  return { loading, error, setError, fetchDefaults, saveDefaults };
}

async function fetchTaskAssignment(taskId: string): Promise<TaskAssignment | null> {
  const res = await fetch(`/api/v1/m/lead-worker/tasks/${taskId}/assignment`);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return (await res.json()) as TaskAssignment;
}

async function patchTaskAgent(taskId: string, title: string, desc: string, lead: string, worker: string) {
  const res = await fetch(`/api/v1/tasks/${taskId}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ title, description: desc, lead_agent: lead, worker_agent: worker }),
  });
  const data = await res.json();
  if (!res.ok) throw new Error(data.error?.message || `HTTP ${res.status}`);
  return true;
}

async function executeDispatch(taskId: string) {
  const res = await fetch(`/api/v1/tasks/${taskId}/dispatch`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
  });
  const data = await res.json();
  if (!res.ok) throw new Error(data.error?.message || `HTTP ${res.status}`);
  return data;
}

export function useTaskAssignment(taskId: string) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchAssignment = useCallback(async () => {
    setLoading(true);
    try { return await fetchTaskAssignment(taskId); }
    catch (err) { setError(err instanceof Error ? err.message : String(err)); return null; }
    finally { setLoading(false); }
  }, [taskId]);

  const assignAgent = useCallback(async (title: string, desc: string, lead: string, worker: string) => {
    setLoading(true);
    try { return await patchTaskAgent(taskId, title, desc, lead, worker); }
    catch (err) { setError(err instanceof Error ? err.message : String(err)); return false; }
    finally { setLoading(false); }
  }, [taskId]);

  const dispatch = useCallback(async () => {
    setLoading(true);
    try { return await executeDispatch(taskId); }
    catch (err) { setError(err instanceof Error ? err.message : String(err)); return null; }
    finally { setLoading(false); }
  }, [taskId]);

  return { loading, error, setError, fetchAssignment, assignAgent, dispatch };
}

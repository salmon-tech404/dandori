import { useCallback, useEffect, useState } from 'react';
import { TechProfileEntry } from './types';

export function useTechProfile(projectId?: string, workspaceId?: string) {
  const [entries, setEntries] = useState<TechProfileEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchEntries = useCallback(async () => {
    if (!projectId) {
      setEntries([]);
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const qs = workspaceId ? `?workspace_id=${encodeURIComponent(workspaceId)}` : '';
      const res = await fetch(`/api/v1/m/tech-profile/projects/${encodeURIComponent(projectId)}/entries${qs}`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      setEntries(data.entries || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [projectId, workspaceId]);

  useEffect(() => {
    fetchEntries();
  }, [fetchEntries]);

  return { entries, loading, error, refresh: fetchEntries };
}

export async function fetchProfilePreview(projectId: string, workspaceId?: string): Promise<string> {
  const qs = workspaceId ? `?workspace_id=${encodeURIComponent(workspaceId)}` : '';
  const res = await fetch(`/api/v1/m/tech-profile/projects/${encodeURIComponent(projectId)}/preview${qs}`);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  const data = await res.json();
  return data.preview || '';
}

export async function deleteProfileEntry(id: string, workspaceId: string): Promise<void> {
  const res = await fetch(`/api/v1/m/tech-profile/entries/${encodeURIComponent(id)}?workspace_id=${encodeURIComponent(workspaceId)}`, {
    method: 'DELETE',
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
}

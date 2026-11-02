import { useCallback, useEffect, useState } from 'react';
import { PromptDetail, PromptSummary, PromptVersion } from './types';

export function usePromptLibrary(workspaceId?: string) {
  const [prompts, setPrompts] = useState<PromptSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchPrompts = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const url = workspaceId
        ? `/api/v1/m/prompt-library/prompts?workspace_id=${encodeURIComponent(workspaceId)}`
        : '/api/v1/m/prompt-library/prompts';
      const res = await fetch(url);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      setPrompts(data.prompts || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [workspaceId]);

  useEffect(() => {
    fetchPrompts();
  }, [fetchPrompts]);

  return { prompts, loading, error, refresh: fetchPrompts };
}

export async function fetchPromptDetail(id: string): Promise<PromptDetail> {
  const res = await fetch(`/api/v1/m/prompt-library/prompts/${encodeURIComponent(id)}`);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json();
}

export async function fetchPromptVersions(id: string): Promise<PromptVersion[]> {
  const res = await fetch(`/api/v1/m/prompt-library/prompts/${encodeURIComponent(id)}/versions`);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  const data = await res.json();
  return data.versions || [];
}

export async function restorePromptVersion(id: string, version: number): Promise<void> {
  const res = await fetch(`/api/v1/m/prompt-library/prompts/${encodeURIComponent(id)}/restore/${version}`, {
    method: 'POST',
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
}

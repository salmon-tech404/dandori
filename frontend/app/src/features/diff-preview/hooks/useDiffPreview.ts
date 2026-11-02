import { useCallback, useEffect, useState } from "react";
import type { DiffResult, PreviewStatus } from "../types";
import { fetchDiff, fetchPreviewStatus, startPreview, stopPreview } from "../api/diffPreviewApi";

export function useDiffPreview(taskId: string) {
  const [diff, setDiff] = useState<DiffResult | null>(null);
  const [status, setStatus] = useState<PreviewStatus | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const [diffRes, statusRes] = await Promise.all([
        fetchDiff(taskId),
        fetchPreviewStatus(taskId).catch(() => null),
      ]);
      setDiff(diffRes);
      setStatus(statusRes);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [taskId]);

  useEffect(() => {
    void loadData();
  }, [loadData]);

  const handleStart = useCallback(async (port: number, command?: string[]) => {
    const res = await startPreview(taskId, { confirmed: true, port, command });
    setStatus(res);
  }, [taskId]);

  const handleStop = useCallback(async () => {
    await stopPreview(taskId);
    const updated = await fetchPreviewStatus(taskId).catch(() => null);
    setStatus(updated);
  }, [taskId]);

  return { diff, status, loading, error, refresh: loadData, handleStart, handleStop };
}

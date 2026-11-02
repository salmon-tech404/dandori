/**
 * Hook quản lý trạng thái module auto-lessons (P3-11).
 */

import { useCallback, useEffect, useState } from "react";
import {
  fetchAutoLessonsConfig,
  fetchAutoLessonsRuns,
  updateAutoLessonsConfig,
} from "./api";
import type {
  AutoLessonsConfig,
  AutoLessonsRunRecord,
  UpdateAutoLessonsConfigRequest,
} from "./types";

export function useAutoLessons(projectId: string | null) {
  const [config, setConfig] = useState<AutoLessonsConfig | null>(null);
  const [runs, setRuns] = useState<AutoLessonsRunRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    if (!projectId) return;
    setLoading(true);
    setError(null);
    try {
      const [c, r] = await Promise.all([
        fetchAutoLessonsConfig(projectId),
        fetchAutoLessonsRuns(projectId),
      ]);
      setConfig(c);
      setRuns(r);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Đã có lỗi xảy ra");
    } finally {
      setLoading(false);
    }
  }, [projectId]);

  useEffect(() => {
    void loadData();
  }, [loadData]);

  const saveConfig = useCallback(
    async (payload: UpdateAutoLessonsConfigRequest) => {
      if (!projectId) return;
      const updated = await updateAutoLessonsConfig(projectId, payload);
      setConfig(updated);
    },
    [projectId]
  );

  return { config, runs, loading, error, refresh: loadData, saveConfig };
}

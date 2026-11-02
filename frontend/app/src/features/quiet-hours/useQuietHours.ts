import { useCallback, useEffect, useState } from "react";
import {
  createQuietWindow,
  deleteQuietWindow,
  fetchQuietWindows,
  toggleQuietWindow,
} from "./api";
import type { CreateQuietWindowRequest, QuietWindow } from "./types";

export function useQuietHours(workspaceId: string) {
  const [windows, setWindows] = useState<QuietWindow[]>([]);
  const [loading, setLoading] = useState(false);

  const reload = useCallback(async () => {
    setLoading(true);
    try {
      const data = await fetchQuietWindows(workspaceId);
      setWindows(data);
    } catch {
      setWindows([]);
    } finally {
      setLoading(false);
    }
  }, [workspaceId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const handleCreate = async (req: CreateQuietWindowRequest) => {
    await createQuietWindow(workspaceId, req);
    await reload();
  };

  const handleToggle = async (id: string) => {
    await toggleQuietWindow(workspaceId, id);
    await reload();
  };

  const handleDelete = async (id: string) => {
    await deleteQuietWindow(workspaceId, id);
    await reload();
  };

  return { windows, loading, handleCreate, handleToggle, handleDelete };
}

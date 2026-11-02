import { useCallback, useEffect, useState } from "react";
import * as api from "./api";
import type { CreateMemoryPayload, MemoryEntry, MemoryFilters } from "./types";

export function useMemorySelection() {
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const toggleSelect = useCallback((id: string) => {
    setSelectedIds((prev) =>
      prev.includes(id) ? prev.filter((i) => i !== id) : [...prev, id]
    );
  }, []);
  const selectAll = useCallback((ids: string[]) => setSelectedIds(ids), []);
  const clearSelection = useCallback(() => setSelectedIds([]), []);
  return { selectedIds, toggleSelect, selectAll, clearSelection };
}

export function useMemoryData(workspaceId: string, filters: MemoryFilters) {
  const [memories, setMemories] = useState<MemoryEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadMemories = useCallback(async () => {
    if (!workspaceId) return;
    setLoading(true);
    setError(null);
    try {
      setMemories(await api.fetchMemories(workspaceId, filters));
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Đã xảy ra lỗi");
    } finally {
      setLoading(false);
    }
  }, [workspaceId, filters]);

  useEffect(() => {
    loadMemories();
  }, [loadMemories]);

  return { memories, loading, error, loadMemories };
}

export function useMemoryActions(
  workspaceId: string,
  loadMemories: () => Promise<void>,
  clearSelection: () => void
) {
  const updateStatus = useCallback(
    async (id: string, status: string) => {
      await api.updateMemoryStatus(workspaceId, id, { status });
      await loadMemories();
    },
    [workspaceId, loadMemories]
  );

  const batchUpdate = useCallback(
    async (ids: string[], status: string) => {
      if (ids.length === 0) return;
      await api.batchUpdateStatus(workspaceId, { memory_ids: ids, status });
      clearSelection();
      await loadMemories();
    },
    [workspaceId, clearSelection, loadMemories]
  );

  const createItem = useCallback(
    async (payload: CreateMemoryPayload) => {
      await api.createMemory(workspaceId, payload);
      await loadMemories();
    },
    [workspaceId, loadMemories]
  );

  const deleteItem = useCallback(
    async (id: string) => {
      await api.deleteMemory(workspaceId, id);
      await loadMemories();
    },
    [workspaceId, loadMemories]
  );

  return { updateStatus, batchUpdate, createItem, deleteItem };
}

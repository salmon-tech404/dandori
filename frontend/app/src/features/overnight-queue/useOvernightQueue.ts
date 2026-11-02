import { useCallback, useEffect, useState } from 'react';
import {
  fetchMorningSummary,
  fetchQueueDetails,
  pauseQueue,
  removeQueueItem,
  reorderQueue,
  resumeQueue,
  startQueueNow,
} from './api';
import type {
  MorningSummaryDto,
  OvernightQueueConfig,
  QueueItemWithTask,
} from './types';

function swapItems(items: QueueItemWithTask[], idx: number, dir: 'up' | 'down'): string[] | null {
  const target = dir === 'up' ? idx - 1 : idx + 1;
  if (target < 0 || target >= items.length) return null;
  const copy = [...items];
  const tmp = copy[idx];
  copy[idx] = copy[target];
  copy[target] = tmp;
  return copy.map((it) => it.id);
}

export function useOvernightQueue(workspaceId: string) {
  const [config, setConfig] = useState<OvernightQueueConfig | null>(null);
  const [items, setItems] = useState<QueueItemWithTask[]>([]);
  const [summary, setSummary] = useState<MorningSummaryDto | null>(null);
  const [loading, setLoading] = useState(false);

  const reload = useCallback(async () => {
    setLoading(true);
    try {
      const details = await fetchQueueDetails(workspaceId);
      setConfig(details.config);
      setItems(details.items);
      setSummary(await fetchMorningSummary(workspaceId));
    } catch {
      // Giữ trạng thái an toàn
    } finally {
      setLoading(false);
    }
  }, [workspaceId]);

  useEffect(() => { void reload(); }, [reload]);

  const act = async (p: Promise<void>) => { await p; await reload(); };
  const handleMove = async (i: number, d: 'up' | 'down') => {
    const ids = swapItems(items, i, d);
    if (ids) await act(reorderQueue(workspaceId, ids));
  };

  return {
    config,
    items,
    summary,
    loading,
    reload,
    handlePause: () => act(pauseQueue(workspaceId)),
    handleResume: () => act(resumeQueue(workspaceId)),
    handleStartNow: () => act(startQueueNow(workspaceId)),
    handleRemoveItem: (id: string) => act(removeQueueItem(workspaceId, id)),
    handleMoveItem: handleMove,
  };
}

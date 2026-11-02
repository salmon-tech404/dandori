/**
 * Hook quản lý trạng thái kênh thông báo (P3-12).
 */

import { useCallback, useEffect, useState } from "react";
import {
  createNotificationChannel,
  deleteNotificationChannel,
  fetchNotificationChannels,
  fetchNotificationHistory,
  sendTestNotification,
  updateNotificationChannel,
} from "./api";
import type {
  CreateChannelRequest,
  NotificationHistoryRecord,
  PublicNotificationChannel,
  UpdateChannelRequest,
} from "./types";

function useChannelData(workspaceId: string) {
  const [channels, setChannels] = useState<PublicNotificationChannel[]>([]);
  const [history, setHistory] = useState<NotificationHistoryRecord[]>([]);
  const [loading, setLoading] = useState(false);

  const reload = useCallback(async () => {
    if (!workspaceId) return;
    setLoading(true);
    try {
      const [cList, hList] = await Promise.all([
        fetchNotificationChannels(workspaceId),
        fetchNotificationHistory(workspaceId),
      ]);
      setChannels(cList);
      setHistory(hList);
    } catch {
      // Bỏ qua lỗi nạp dữ liệu nền
    } finally {
      setLoading(false);
    }
  }, [workspaceId]);

  return { channels, history, loading, reload };
}

function useChannelMutations(workspaceId: string, reload: () => Promise<void>) {
  const [testResult, setTestResult] = useState<string | null>(null);

  const addChannel = useCallback(
    async (p: CreateChannelRequest) => {
      await createNotificationChannel(workspaceId, p);
      await reload();
    },
    [workspaceId, reload]
  );

  const editChannel = useCallback(
    async (id: string, p: UpdateChannelRequest) => {
      await updateNotificationChannel(workspaceId, id, p);
      await reload();
    },
    [workspaceId, reload]
  );

  const removeChannel = useCallback(
    async (id: string) => {
      await deleteNotificationChannel(workspaceId, id);
      await reload();
    },
    [workspaceId, reload]
  );

  const testChannel = useCallback(
    async (id: string) => {
      setTestResult(null);
      const res = await sendTestNotification(workspaceId, id);
      setTestResult(res.success ? "success" : res.error || "failed");
      await reload();
    },
    [workspaceId, reload]
  );

  return { testResult, addChannel, editChannel, removeChannel, testChannel };
}

export function useNotifications(workspaceId: string) {
  const { channels, history, loading, reload } = useChannelData(workspaceId);
  const mutations = useChannelMutations(workspaceId, reload);

  useEffect(() => {
    void reload();
  }, [reload]);

  return { channels, history, loading, reload, ...mutations };
}

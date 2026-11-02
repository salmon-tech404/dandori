import { useEffect, useRef, useState } from 'react';
import { ActiveAgent, ConnectionState, LogEntry, MAX_LOG_ENTRIES } from './types';

interface SseEnvelope {
  id: string;
  kind: string;
  at: string;
  scope: { workspace_id?: string; project_id?: string; task_id?: string; run_id?: string };
  payload: Record<string, unknown>;
}

function handleRunStarted(
  payload: Record<string, unknown>,
  scope: SseEnvelope['scope'],
  at: string,
  prev: ActiveAgent[],
): ActiveAgent[] {
  const runId = String(payload.run_id || scope.run_id || '');
  const taskId = String(scope.task_id || '');
  if (prev.some((a) => a.runId === runId)) return prev;
  return [
    ...prev,
    {
      runId,
      taskId,
      taskTitle: `Nhiệm vụ #${taskId.slice(0, 6)}`,
      agent: String(payload.agent || 'fake-agent'),
      account: String(payload.account || 'acc-default'),
      assignedBy: 'Hệ thống',
      startedAt: at,
      status: 'running',
    },
  ];
}

function appendLogEntry(
  prevLogs: Record<string, LogEntry[]>,
  runId: string,
  entry: LogEntry,
): Record<string, LogEntry[]> {
  const current = prevLogs[runId] || [];
  const updated = [...current, entry];
  const capped = updated.length > MAX_LOG_ENTRIES ? updated.slice(-MAX_LOG_ENTRIES) : updated;
  return { ...prevLogs, [runId]: capped };
}

function handleEnvelope(
  env: SseEnvelope,
  setActiveAgents: React.Dispatch<React.SetStateAction<ActiveAgent[]>>,
  setLogsByRunId: React.Dispatch<React.SetStateAction<Record<string, LogEntry[]>>>,
) {
  if (env.kind === 'run_started') {
    setActiveAgents((prev) => handleRunStarted(env.payload, env.scope, env.at, prev));
  } else if (env.kind === 'run_output') {
    const runId = String(env.payload.run_id || env.scope.run_id || '');
    const entry: LogEntry = {
      id: env.id,
      seq: Number(env.payload.seq || 0),
      stream: (env.payload.stream as 'stdout' | 'stderr') || 'stdout',
      chunk: String(env.payload.chunk || ''),
      timestamp: env.at,
    };
    setLogsByRunId((prev) => appendLogEntry(prev, runId, entry));
  } else if (env.kind === 'run_finished' || env.kind === 'run_failed') {
    const runId = String(env.payload.run_id || env.scope.run_id || '');
    const status = env.kind === 'run_finished' ? 'completed' : 'failed';
    setActiveAgents((prev) => prev.map((a) => (a.runId === runId ? { ...a, status } : a)));
  }
}

function bindEventSource(
  es: EventSource,
  onEnvelope: (env: SseEnvelope) => void,
  onDisabled: () => void,
) {
  const handler = (e: MessageEvent) => {
    try {
      onEnvelope(JSON.parse(e.data));
    } catch {
      // Bỏ qua frame không parse được
    }
  };
  es.addEventListener('run_started', handler);
  es.addEventListener('run_output', handler);
  es.addEventListener('run_finished', handler);
  es.addEventListener('run_failed', handler);
  es.addEventListener('module_disabled', onDisabled);
}

export function useLiveStatus(workspaceId?: string) {
  const [connectionState, setConnectionState] = useState<ConnectionState>('connecting');
  const [activeAgents, setActiveAgents] = useState<ActiveAgent[]>([]);
  const [logsByRunId, setLogsByRunId] = useState<Record<string, LogEntry[]>>({});
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const lastIdRef = useRef<string | null>(null);

  useEffect(() => {
    if (!workspaceId) {
      setConnectionState('disconnected');
      return undefined;
    }
    setConnectionState('connecting');
    const query = new URLSearchParams({ workspace_id: workspaceId });
    if (lastIdRef.current) query.set('last_event_id', lastIdRef.current);
    const es = new EventSource(`/api/v1/m/live-status/stream?${query.toString()}`);

    es.onopen = () => setConnectionState('connected');
    es.onerror = () => {
      setConnectionState('reconnecting');
      es.close();
    };
    bindEventSource(
      es,
      (env) => {
        lastIdRef.current = env.id;
        handleEnvelope(env, setActiveAgents, setLogsByRunId);
      },
      () => {
        setConnectionState('disconnected');
        es.close();
      },
    );
    return () => es.close();
  }, [workspaceId]);

  return { connectionState, activeAgents, logsByRunId, selectedRunId, setSelectedRunId };
}

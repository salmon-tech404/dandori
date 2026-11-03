import { useEffect, useRef, useState } from 'react';
import type { ActiveAgent, ConnectionState, LogEntry } from './types';

interface SseEnvelope {
  id: string;
  kind: string;
  scope: Record<string, unknown>;
  payload: Record<string, unknown>;
  at: string;
}

function appendLogEntry(
  prev: Record<string, LogEntry[]>,
  runId: string,
  entry: LogEntry,
): Record<string, LogEntry[]> {
  const existing = prev[runId] || [];
  const next = [...existing, entry];
  if (next.length > 500) next.shift();
  return { ...prev, [runId]: next };
}

function parseRunStartedAgent(env: SseEnvelope): ActiveAgent {
  return {
    runId: String(env.payload.run_id || env.scope.run_id || ''),
    taskId: String(env.payload.task_id || env.scope.task_id || ''),
    taskTitle: String(env.payload.task_title || env.payload.title || 'Task'),
    agent: String(env.payload.agent || 'Agent'),
    account: String(env.payload.account || 'default'),
    assignedBy: String(env.payload.assigned_by || 'system'),
    startedAt: env.at,
    status: 'running',
  };
}

function handleRunStarted(
  env: SseEnvelope,
  setActiveAgents: React.Dispatch<React.SetStateAction<ActiveAgent[]>>,
) {
  const newAgent = parseRunStartedAgent(env);
  setActiveAgents((prev) => {
    const idx = prev.findIndex((a) => a.runId === newAgent.runId);
    if (idx >= 0) {
      const next = [...prev];
      next[idx] = newAgent;
      return next;
    }
    return [newAgent, ...prev];
  });
}

function handleEnvelope(
  env: SseEnvelope,
  setActiveAgents: React.Dispatch<React.SetStateAction<ActiveAgent[]>>,
  setLogsByRunId: React.Dispatch<React.SetStateAction<Record<string, LogEntry[]>>>,
) {
  if (env.kind === 'run_started') {
    handleRunStarted(env, setActiveAgents);
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

interface StreamCallbacks {
  onStateChange: (state: ConnectionState) => void;
  onEnvelope: (env: SseEnvelope) => void;
}

function initEventSourceHandlers(
  es: EventSource,
  callbacks: StreamCallbacks,
  getStopped: () => boolean,
  scheduleRetry: () => void,
) {
  es.onopen = () => {
    if (!getStopped()) callbacks.onStateChange('connected');
  };
  es.addEventListener('ping', () => {
    if (!getStopped()) callbacks.onStateChange('connected');
  });
  es.onerror = () => {
    if (getStopped()) return;
    callbacks.onStateChange('reconnecting');
    if (es.readyState === EventSource.CLOSED) {
      es.close();
      scheduleRetry();
    }
  };
  bindEventSource(
    es,
    callbacks.onEnvelope,
    () => {
      if (!getStopped()) {
        callbacks.onStateChange('disconnected');
        es.close();
      }
    },
  );
}

function setupLiveStream(
  workspaceId: string,
  lastIdRef: React.MutableRefObject<string | null>,
  callbacks: StreamCallbacks,
): () => void {
  let retryTimer: ReturnType<typeof setTimeout> | null = null;
  let es: EventSource | null = null;
  let stopped = false;

  const connect = () => {
    if (stopped) return;
    callbacks.onStateChange('connecting');
    const query = new URLSearchParams({ workspace_id: workspaceId });
    if (lastIdRef.current) query.set('last_event_id', lastIdRef.current);
    try {
      es = new EventSource(`/api/v1/m/live-status/stream?${query.toString()}`);
    } catch {
      callbacks.onStateChange('disconnected');
      return;
    }
    initEventSourceHandlers(es, callbacks, () => stopped, () => {
      retryTimer = setTimeout(connect, 3000);
    });
  };

  connect();

  return () => {
    stopped = true;
    if (retryTimer) clearTimeout(retryTimer);
    if (es) es.close();
  };
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
    return setupLiveStream(workspaceId, lastIdRef, {
      onStateChange: setConnectionState,
      onEnvelope: (env) => {
        lastIdRef.current = env.id;
        handleEnvelope(env, setActiveAgents, setLogsByRunId);
      },
    });
  }, [workspaceId]);

  return { connectionState, activeAgents, logsByRunId, selectedRunId, setSelectedRunId };
}

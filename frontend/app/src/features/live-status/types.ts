export type ConnectionState =
  | 'connecting'
  | 'connected'
  | 'disconnected'
  | 'reconnecting'
  | 'error';

export interface ActiveAgent {
  runId: string;
  taskId: string;
  taskTitle: string;
  agent: string;
  account: string;
  assignedBy: string;
  startedAt: string;
  status: 'running' | 'completed' | 'failed' | 'cancelled';
}

export interface LogEntry {
  id: string;
  seq: number;
  stream: 'stdout' | 'stderr';
  chunk: string;
  timestamp: string;
}

export const MAX_LOG_ENTRIES = 500;

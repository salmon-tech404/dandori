import type { ListRunsParams, RunEventItem, RunHistoryNote, RunSummary } from '../types';

export async function fetchRuns(params: ListRunsParams = {}): Promise<{ runs: RunSummary[]; limit: number; offset: number }> {
  const query = new URLSearchParams();
  if (params.task_id) query.set('task_id', params.task_id);
  if (params.agent_provider) query.set('agent_provider', params.agent_provider);
  if (params.agent_account_id) query.set('agent_account_id', params.agent_account_id);
  if (params.status) query.set('status', params.status);
  if (params.from) query.set('from', params.from);
  if (params.to) query.set('to', params.to);
  if (params.limit !== undefined) query.set('limit', String(params.limit));
  if (params.offset !== undefined) query.set('offset', String(params.offset));

  const url = `/api/v1/m/run-history/runs${query.toString() ? `?${query.toString()}` : ''}`;
  const resp = await fetch(url);
  if (!resp.ok) {
    throw new Error(`Lỗi tải danh sách lần chạy: ${resp.status}`);
  }
  return resp.json();
}

export async function fetchRunEvents(runId: string, limit = 200, offset = 0): Promise<{ events: RunEventItem[] }> {
  const resp = await fetch(`/api/v1/m/run-history/runs/${encodeURIComponent(runId)}/events?limit=${limit}&offset=${offset}`);
  if (!resp.ok) {
    throw new Error(`Lỗi tải sự kiện lần chạy: ${resp.status}`);
  }
  return resp.json();
}

export async function fetchNote(runId: string): Promise<RunHistoryNote> {
  const resp = await fetch(`/api/v1/m/run-history/runs/${encodeURIComponent(runId)}/note`);
  if (!resp.ok) {
    throw new Error(`Lỗi tải ghi chú: ${resp.status}`);
  }
  return resp.json();
}

export async function saveNote(runId: string, note: string, tags: string[]): Promise<RunHistoryNote> {
  const resp = await fetch(`/api/v1/m/run-history/runs/${encodeURIComponent(runId)}/note`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ note, tags }),
  });
  if (!resp.ok) {
    const errText = await resp.text();
    throw new Error(errText || `Lỗi lưu ghi chú: ${resp.status}`);
  }
  return resp.json();
}

export async function exportRun(runId: string, format: 'markdown' | 'json'): Promise<string> {
  const resp = await fetch(`/api/v1/m/run-history/runs/${encodeURIComponent(runId)}/export?format=${format}`);
  if (!resp.ok) {
    throw new Error(`Lỗi xuất dữ liệu lần chạy: ${resp.status}`);
  }
  return resp.text();
}

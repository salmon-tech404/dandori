import type {
  MorningSummaryDto,
  QueueDetailsResponse,
  UpdateQueueConfigRequest,
} from './types';

const BASE_URL = '/api/v1/m/overnight-queue';

export async function fetchQueueDetails(workspaceId: string): Promise<QueueDetailsResponse> {
  const res = await fetch(`${BASE_URL}/workspaces/${workspaceId}/queue`);
  if (!res.ok) {
    throw new Error(`Failed to fetch overnight queue: ${res.statusText}`);
  }
  return res.json();
}

export async function updateQueueConfig(
  workspaceId: string,
  req: UpdateQueueConfigRequest
): Promise<void> {
  const res = await fetch(`${BASE_URL}/workspaces/${workspaceId}/queue`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  });
  if (!res.ok) {
    throw new Error(`Failed to update overnight queue config: ${res.statusText}`);
  }
}

export async function enqueueTasks(workspaceId: string, taskIds: string[]): Promise<void> {
  const res = await fetch(`${BASE_URL}/workspaces/${workspaceId}/queue/items`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ task_ids: taskIds }),
  });
  if (!res.ok) {
    throw new Error(`Failed to enqueue tasks: ${res.statusText}`);
  }
}

export async function removeQueueItem(workspaceId: string, itemId: string): Promise<void> {
  const res = await fetch(`${BASE_URL}/workspaces/${workspaceId}/queue/items/${itemId}`, {
    method: 'DELETE',
  });
  if (!res.ok) {
    throw new Error(`Failed to remove queue item: ${res.statusText}`);
  }
}

export async function reorderQueue(
  workspaceId: string,
  orderedItemIds: string[]
): Promise<void> {
  const res = await fetch(`${BASE_URL}/workspaces/${workspaceId}/queue/reorder`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ ordered_item_ids: orderedItemIds }),
  });
  if (!res.ok) {
    throw new Error(`Failed to reorder queue: ${res.statusText}`);
  }
}

export async function pauseQueue(workspaceId: string): Promise<void> {
  const res = await fetch(`${BASE_URL}/workspaces/${workspaceId}/queue/pause`, {
    method: 'POST',
  });
  if (!res.ok) {
    throw new Error(`Failed to pause queue: ${res.statusText}`);
  }
}

export async function resumeQueue(workspaceId: string): Promise<void> {
  const res = await fetch(`${BASE_URL}/workspaces/${workspaceId}/queue/resume`, {
    method: 'POST',
  });
  if (!res.ok) {
    throw new Error(`Failed to resume queue: ${res.statusText}`);
  }
}

export async function startQueueNow(workspaceId: string): Promise<void> {
  const res = await fetch(`${BASE_URL}/workspaces/${workspaceId}/queue/start-now`, {
    method: 'POST',
  });
  if (!res.ok) {
    throw new Error(`Failed to start queue now: ${res.statusText}`);
  }
}

export async function fetchMorningSummary(
  workspaceId: string
): Promise<MorningSummaryDto | null> {
  const res = await fetch(`${BASE_URL}/workspaces/${workspaceId}/queue/summary`);
  if (!res.ok) {
    throw new Error(`Failed to fetch morning summary: ${res.statusText}`);
  }
  const data = await res.json();
  return data.summary || null;
}

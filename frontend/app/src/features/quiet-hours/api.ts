import type {
  CreateQuietWindowRequest,
  QuietStatus,
  QuietWindow,
  UpdateQuietWindowRequest,
} from "./types";

export async function fetchQuietWindows(workspaceId: string): Promise<QuietWindow[]> {
  const res = await fetch(`/api/v1/m/quiet-hours/workspaces/${workspaceId}/windows`);
  if (!res.ok) {
    throw new Error(`Failed to fetch quiet windows: ${res.statusText}`);
  }
  const data = await res.json();
  return data.windows || [];
}

export async function createQuietWindow(
  workspaceId: string,
  req: CreateQuietWindowRequest
): Promise<QuietWindow> {
  const res = await fetch(`/api/v1/m/quiet-hours/workspaces/${workspaceId}/windows`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(req),
  });
  if (!res.ok) {
    throw new Error(`Failed to create quiet window: ${res.statusText}`);
  }
  const data = await res.json();
  return data.window;
}

export async function updateQuietWindow(
  workspaceId: string,
  windowId: string,
  req: UpdateQuietWindowRequest
): Promise<QuietWindow> {
  const res = await fetch(
    `/api/v1/m/quiet-hours/workspaces/${workspaceId}/windows/${windowId}`,
    {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(req),
    }
  );
  if (!res.ok) {
    throw new Error(`Failed to update quiet window: ${res.statusText}`);
  }
  const data = await res.json();
  return data.window;
}

export async function deleteQuietWindow(
  workspaceId: string,
  windowId: string
): Promise<void> {
  const res = await fetch(
    `/api/v1/m/quiet-hours/workspaces/${workspaceId}/windows/${windowId}`,
    {
      method: "DELETE",
    }
  );
  if (!res.ok) {
    throw new Error(`Failed to delete quiet window: ${res.statusText}`);
  }
}

export async function toggleQuietWindow(
  workspaceId: string,
  windowId: string
): Promise<QuietWindow> {
  const res = await fetch(
    `/api/v1/m/quiet-hours/workspaces/${workspaceId}/windows/${windowId}/toggle`,
    {
      method: "POST",
    }
  );
  if (!res.ok) {
    throw new Error(`Failed to toggle quiet window: ${res.statusText}`);
  }
  const data = await res.json();
  return data.window;
}

export async function fetchQuietStatus(
  workspaceId: string,
  projectId?: string
): Promise<QuietStatus> {
  const query = projectId ? `?project_id=${projectId}` : "";
  const res = await fetch(`/api/v1/m/quiet-hours/workspaces/${workspaceId}/status${query}`);
  if (!res.ok) {
    throw new Error(`Failed to fetch quiet status: ${res.statusText}`);
  }
  const data = await res.json();
  return data.status;
}

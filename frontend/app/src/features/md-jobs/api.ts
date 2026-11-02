import type { MdJobDto } from "./types";

export async function fetchJobs(projectId: string): Promise<MdJobDto[]> {
  const res = await fetch(`/api/v1/m/md-jobs/jobs?project_id=${projectId}`);
  if (!res.ok) {
    throw new Error(`Failed to fetch jobs: ${res.statusText}`);
  }
  const data = await res.json();
  return data.jobs || [];
}

export async function syncJobs(projectId: string): Promise<number> {
  const res = await fetch("/api/v1/m/md-jobs/sync", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ project_id: projectId }),
  });
  if (!res.ok) {
    throw new Error(`Failed to sync jobs: ${res.statusText}`);
  }
  const data = await res.json();
  return data.scanned_files || 0;
}

export async function approveJob(id: string): Promise<void> {
  const res = await fetch(`/api/v1/m/md-jobs/jobs/${id}/approve`, {
    method: "POST",
  });
  if (!res.ok) {
    throw new Error(`Failed to approve job: ${res.statusText}`);
  }
}

export async function rejectJob(id: string): Promise<void> {
  const res = await fetch(`/api/v1/m/md-jobs/jobs/${id}/reject`, {
    method: "POST",
  });
  if (!res.ok) {
    throw new Error(`Failed to reject job: ${res.statusText}`);
  }
}

export async function fireJobNow(id: string): Promise<void> {
  const res = await fetch(`/api/v1/m/md-jobs/jobs/${id}/fire-now`, {
    method: "POST",
  });
  if (!res.ok) {
    throw new Error(`Failed to fire job: ${res.statusText}`);
  }
}

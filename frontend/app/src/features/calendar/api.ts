import type {
  CreateScheduleRequest,
  ScheduleDto,
  UpdateScheduleRequest,
} from "./types";

export async function fetchSchedules(projectId: string): Promise<ScheduleDto[]> {
  const res = await fetch(`/api/v1/m/calendar/schedules?project_id=${projectId}`);
  if (!res.ok) {
    throw new Error(`Failed to fetch schedules: ${res.statusText}`);
  }
  const data = await res.json();
  return data.schedules || [];
}

export async function createSchedule(req: CreateScheduleRequest): Promise<ScheduleDto> {
  const res = await fetch("/api/v1/m/calendar/schedules", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(req),
  });
  if (!res.ok) {
    throw new Error(`Failed to create schedule: ${res.statusText}`);
  }
  const data = await res.json();
  return data.schedule;
}

export async function updateSchedule(
  id: string,
  req: UpdateScheduleRequest
): Promise<ScheduleDto> {
  const res = await fetch(`/api/v1/m/calendar/schedules/${id}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(req),
  });
  if (!res.ok) {
    throw new Error(`Failed to update schedule: ${res.statusText}`);
  }
  const data = await res.json();
  return data.schedule;
}

export async function deleteSchedule(id: string): Promise<void> {
  const res = await fetch(`/api/v1/m/calendar/schedules/${id}`, {
    method: "DELETE",
  });
  if (!res.ok) {
    throw new Error(`Failed to delete schedule: ${res.statusText}`);
  }
}

export async function pauseSchedule(id: string): Promise<void> {
  const res = await fetch(`/api/v1/m/calendar/schedules/${id}/pause`, {
    method: "POST",
  });
  if (!res.ok) {
    throw new Error(`Failed to pause schedule: ${res.statusText}`);
  }
}

export async function resumeSchedule(id: string): Promise<void> {
  const res = await fetch(`/api/v1/m/calendar/schedules/${id}/resume`, {
    method: "POST",
  });
  if (!res.ok) {
    throw new Error(`Failed to resume schedule: ${res.statusText}`);
  }
}

export async function fireScheduleNow(id: string): Promise<void> {
  const res = await fetch(`/api/v1/m/calendar/schedules/${id}/fire-now`, {
    method: "POST",
  });
  if (!res.ok) {
    throw new Error(`Failed to fire schedule: ${res.statusText}`);
  }
}

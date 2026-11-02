import type { PricingRule, SetPricingRequest, TaskCostSummary } from "./types";

export async function fetchTaskCost(taskId: string): Promise<TaskCostSummary> {
  const res = await fetch(`/api/v1/m/cost-tracking/tasks/${taskId}/cost`);
  if (!res.ok) {
    throw new Error(`Failed to fetch task cost: ${res.statusText}`);
  }
  return res.json();
}

export async function setTaskBudget(taskId: string, maxBudget: number): Promise<void> {
  const res = await fetch(`/api/v1/m/cost-tracking/tasks/${taskId}/budget`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ max_budget: maxBudget }),
  });
  if (!res.ok) {
    throw new Error(`Failed to set task budget: ${res.statusText}`);
  }
}

export async function fetchProjectPricing(projectId: string): Promise<PricingRule[]> {
  const res = await fetch(`/api/v1/m/cost-tracking/projects/${projectId}/pricing`);
  if (!res.ok) {
    throw new Error(`Failed to fetch pricing: ${res.statusText}`);
  }
  const data = await res.json();
  return data.pricing || [];
}

export async function setProjectPricing(projectId: string, req: SetPricingRequest): Promise<void> {
  const res = await fetch(`/api/v1/m/cost-tracking/projects/${projectId}/pricing`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(req),
  });
  if (!res.ok) {
    throw new Error(`Failed to set pricing: ${res.statusText}`);
  }
}

export async function deleteProjectPricing(projectId: string, pricingId: string): Promise<void> {
  const res = await fetch(`/api/v1/m/cost-tracking/projects/${projectId}/pricing/${pricingId}`, {
    method: "DELETE",
  });
  if (!res.ok) {
    throw new Error(`Failed to delete pricing: ${res.statusText}`);
  }
}

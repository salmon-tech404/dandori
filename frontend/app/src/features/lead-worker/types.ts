export interface RoleDefaultItem {
  role: string;
  provider_order: string[];
}

export interface TaskAssignment {
  task_id: string;
  effective_lead_agent: string;
  effective_worker_agent: string;
  task_lead_agent: string;
  task_worker_agent: string;
  is_lead_overridden: boolean;
  is_worker_overridden: boolean;
  ready_providers: string[];
}

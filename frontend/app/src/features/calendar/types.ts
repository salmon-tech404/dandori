/**
 * Kiểu dữ liệu TypeScript cho tính năng Calendar (P3-01).
 */

export interface TaskTemplateDto {
  title: string;
  description: string;
  lead_agent: string;
  worker_agent: string;
}

export interface ScheduleDto {
  id: string;
  workspace_id: string;
  project_id: string;
  title: string;
  schedule_type: "cron" | "interval";
  schedule_expr: string;
  timezone: string;
  task_template: TaskTemplateDto;
  auto_dispatch: boolean;
  missed_policy: "skip" | "run_once";
  enabled: boolean;
  last_fire: string | null;
  next_fire: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateScheduleRequest {
  project_id: string;
  title: string;
  schedule_type: "cron" | "interval";
  schedule_expr: string;
  timezone?: string;
  task_template: TaskTemplateDto;
  auto_dispatch?: boolean;
  missed_policy?: "skip" | "run_once";
}

export interface UpdateScheduleRequest {
  title?: string;
  schedule_type?: "cron" | "interval";
  schedule_expr?: string;
  timezone?: string;
  task_template?: TaskTemplateDto;
  auto_dispatch?: boolean;
  missed_policy?: "skip" | "run_once";
  enabled?: boolean;
}

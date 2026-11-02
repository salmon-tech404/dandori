export interface ReportCounts {
  completed: number;
  failed: number;
  needs_human: number;
  running: number;
  total: number;
}

export interface DurationStats {
  total_duration_secs: number;
  median_duration_secs: number;
  avg_duration_secs: number;
}

export interface ProviderAccountUsage {
  agent: string;
  account: string;
  run_count: number;
}

export interface AccountSwitchStats {
  switch_count: number | null;
}

export interface CostStats {
  total_cost: number | null;
  currency: string | null;
  has_data: boolean;
  missing_reason: string | null;
}

export interface TopTaskItem {
  task_id: string;
  title: string;
  status: string;
  duration_secs: number | null;
}

export interface ReportData {
  workspace_id: string;
  period_type: 'daily' | 'weekly';
  period_start: string;
  period_end: string;
  timezone: string;
  counts: ReportCounts;
  duration: DurationStats;
  usage: ProviderAccountUsage[];
  switches: AccountSwitchStats;
  cost: CostStats;
  top_tasks: TopTaskItem[];
  generated_at: string;
}

export interface ReportSnapshot {
  id: string;
  workspace_id: string;
  period_type: 'daily' | 'weekly';
  period_start: string;
  period_end: string;
  timezone: string;
  data_json: string;
  markdown: string;
  created_at: string;
}

export interface ReportConfig {
  workspace_id: string;
  auto_generate: number;
  timezone: string;
  daily_time: string;
  weekly_day: number;
  weekly_time: string;
  updated_at: string;
}

export interface CurrentReportResponse {
  report: ReportData;
  markdown: string;
}

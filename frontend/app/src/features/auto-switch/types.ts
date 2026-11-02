export interface AutoSwitchConfig {
  project_id: string;
  max_switches: number;
  allow_cross_provider: boolean;
  allowed_providers: string[];
}

export interface UpdateAutoSwitchConfigPayload {
  max_switches?: number;
  allow_cross_provider?: boolean;
  allowed_providers?: string[];
}

export interface AutoSwitchHistoryItem {
  id: string;
  task_id: string;
  from_account_id: string;
  to_account_id: string;
  from_provider: string;
  to_provider: string;
  reason: string;
  switch_count: number;
  created_at: string;
}

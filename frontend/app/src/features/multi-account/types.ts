export interface RunningTaskInfo {
  task_id: string;
  task_title: string;
  run_id: string;
}

export interface AccountDetail {
  id: string;
  provider: string;
  label: string;
  home_dir: string;
  status: string;
  enabled: boolean;
  priority: number;
  last_checked_at?: string;
  running_task?: RunningTaskInfo;
}

export interface ProviderGroup {
  provider: string;
  supports_home_isolation: boolean;
  accounts: AccountDetail[];
}

export interface UpdateAccountPayload {
  priority?: number;
  enabled?: boolean;
  label?: string;
}

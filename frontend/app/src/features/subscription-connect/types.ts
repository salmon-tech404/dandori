export interface AgentAccount {
  id: string;
  provider: string;
  label: string;
  home_dir: string;
  auth_type: string;
  status: string;
  enabled: boolean;
  priority: number;
  last_checked_at?: string;
}

export interface LoginInstructions {
  account_id: string;
  provider: string;
  home_dir: string;
  command: string;
  instructions: string;
}

export interface SpendingLimit {
  period: 'daily' | 'weekly' | 'monthly';
  limit_type: 'currency' | 'credits';
  limit_value: number;
  used_value: number;
  period_start: string;
  alert_threshold: number;
}

export interface ApiKeyAccount {
  id: string;
  provider: string;
  label: string;
  auth_type: string;
  masked_key: string;
  status: string;
  enabled: boolean;
  priority: number;
  created_at: string;
  spending_limit: SpendingLimit | null;
}

export interface ProjectFallbackConfig {
  project_id: string;
  fallback_enabled: boolean;
  allowed_accounts: string[];
}

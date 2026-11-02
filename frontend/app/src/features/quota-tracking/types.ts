export type QuotaSource = "reported" | "estimated" | "no_data";

export interface QuotaStatusResponse {
  account_id: string;
  source: QuotaSource;
  used_5h: number;
  limit_5h: number | null;
  percentage_5h: number | null;
  used_weekly: number;
  limit_weekly: number | null;
  percentage_weekly: number | null;
  alert_threshold_percent: number;
  exhausted_until: string | null;
}

export interface AccountQuotaConfigPayload {
  limit_5h?: number | null;
  limit_weekly?: number | null;
  alert_threshold_percent?: number | null;
}

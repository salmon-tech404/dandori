import type { QuotaStatusResponse, AccountQuotaConfigPayload } from "./types";

export async function fetchQuotaStatus(accountId: string): Promise<QuotaStatusResponse> {
  const resp = await fetch(`/api/v1/m/quota-tracking/accounts/${accountId}/status`, {
    headers: { "Content-Type": "application/json" },
  });
  if (!resp.ok) {
    throw new Error(`Lỗi tải hạn mức tài khoản: ${resp.status}`);
  }
  return resp.json();
}

export async function updateQuotaConfig(
  accountId: string,
  payload: AccountQuotaConfigPayload
): Promise<void> {
  const resp = await fetch(`/api/v1/m/quota-tracking/accounts/${accountId}/config`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!resp.ok) {
    throw new Error(`Lỗi cập nhật cấu hình hạn mức: ${resp.status}`);
  }
}

export async function markAccountExhausted(
  accountId: string,
  waitHours = 5,
  reason = "Quota limit reached"
): Promise<void> {
  const resp = await fetch(`/api/v1/m/quota-tracking/accounts/${accountId}/mark-exhausted`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ wait_hours: waitHours, reason }),
  });
  if (!resp.ok) {
    throw new Error(`Lỗi đánh dấu hết hạn mức: ${resp.status}`);
  }
}

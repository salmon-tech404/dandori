import type { ProviderGroup, AccountDetail, UpdateAccountPayload } from "./types";

export async function fetchGroupedAccounts(): Promise<ProviderGroup[]> {
  const resp = await fetch("/api/v1/m/multi-account/accounts", {
    headers: { "Content-Type": "application/json" },
  });
  if (!resp.ok) {
    throw new Error(`Lỗi tải danh sách tài khoản: ${resp.status}`);
  }
  return resp.json();
}

export async function updateAccount(id: string, payload: UpdateAccountPayload): Promise<AccountDetail> {
  const resp = await fetch(`/api/v1/m/multi-account/accounts/${id}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!resp.ok) {
    throw new Error(`Lỗi cập nhật tài khoản: ${resp.status}`);
  }
  return resp.json();
}

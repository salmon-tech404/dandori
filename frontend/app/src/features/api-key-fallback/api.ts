import type { ApiKeyAccount, ProjectFallbackConfig, SpendingLimit } from './types';

export async function fetchApiKeyAccounts(): Promise<ApiKeyAccount[]> {
  const res = await fetch('/api/v1/m/api-key-fallback/accounts');
  if (!res.ok) {
    throw new Error('Lỗi lấy danh sách tài khoản API key');
  }
  const data = await res.json();
  return data.accounts ?? [];
}

export async function createApiKeyAccount(payload: {
  provider: string;
  label: string;
  api_key: string;
  spending_limit: Omit<SpendingLimit, 'used_value' | 'period_start'> | null;
}): Promise<ApiKeyAccount> {
  const res = await fetch('/api/v1/m/api-key-fallback/accounts', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.message || 'Lỗi thêm tài khoản API key');
  }
  return res.json();
}

export async function toggleApiKeyAccount(id: string): Promise<{ id: string; enabled: boolean }> {
  const res = await fetch(`/api/v1/m/api-key-fallback/accounts/${id}/toggle`, {
    method: 'PATCH',
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.message || 'Lỗi bật/tắt tài khoản API key');
  }
  return res.json();
}

export async function updateSpendingLimit(
  id: string,
  limit: Omit<SpendingLimit, 'used_value' | 'period_start'>
): Promise<{ success: boolean }> {
  const res = await fetch(`/api/v1/m/api-key-fallback/accounts/${id}/spending-limit`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(limit),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.message || 'Lỗi cập nhật trần chi tiêu');
  }
  return res.json();
}

export async function deleteApiKeyAccount(id: string): Promise<void> {
  const res = await fetch(`/api/v1/m/api-key-fallback/accounts/${id}`, {
    method: 'DELETE',
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.message || 'Lỗi xóa tài khoản API key');
  }
}

export async function fetchProjectFallbackConfig(
  projectId: string
): Promise<ProjectFallbackConfig> {
  const res = await fetch(`/api/v1/m/api-key-fallback/projects/${projectId}/config`);
  if (!res.ok) {
    throw new Error('Lỗi lấy cấu hình dự phòng dự án');
  }
  return res.json();
}

export async function updateProjectFallbackConfig(
  projectId: string,
  config: { fallback_enabled: boolean; allowed_accounts: string[] }
): Promise<ProjectFallbackConfig> {
  const res = await fetch(`/api/v1/m/api-key-fallback/projects/${projectId}/config`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(config),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.message || 'Lỗi cập nhật cấu hình dự phòng dự án');
  }
  return res.json();
}

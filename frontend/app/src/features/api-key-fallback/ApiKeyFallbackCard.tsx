import { useState, useEffect } from 'react';
import { t } from '../../core/i18n';
import type { ApiKeyAccount, SpendingLimit } from './types';
import {
  fetchApiKeyAccounts,
  createApiKeyAccount,
  toggleApiKeyAccount,
  deleteApiKeyAccount,
} from './api';
import { AddApiKeyModal } from './AddApiKeyModal';

export function ApiKeySpendingBar({ limit }: { limit: SpendingLimit }) {
  const pct = limit.limit_value > 0 ? (limit.used_value / limit.limit_value) * 100 : 0;
  const isAlert = pct >= limit.alert_threshold;
  const isCap = pct >= 100;
  const fillClass = isCap ? 'bg-danger' : isAlert ? 'bg-warning' : 'bg-primary';

  return (
    <div className="spending-bar-container">
      <div className="spending-labels">
        <span>Đã dùng: {limit.used_value.toFixed(2)} / {limit.limit_value.toFixed(2)} ({limit.period})</span>
        <span className={`pct-label ${isCap ? 'cap' : isAlert ? 'alert' : ''}`}>{pct.toFixed(0)}%</span>
      </div>
      <div className="progress-bar-track">
        <div
          className={`progress-bar-fill ${fillClass}`}
          style={{ width: `${Math.min(pct, 100)}%` }}
          data-testid="spending-progress-bar"
        />
      </div>
    </div>
  );
}

export function ApiKeyItem({
  acc,
  onToggle,
  onDelete,
}: {
  acc: ApiKeyAccount;
  onToggle: (id: string) => void;
  onDelete: (id: string) => void;
}) {
  return (
    <div className="api-key-item">
      <div className="key-main-info">
        <span className="key-provider-badge">{acc.provider}</span>
        <span className="key-label">{acc.label}</span>
        <span className="masked-key-badge" data-testid="masked-key">{acc.masked_key}</span>
        <span className={`key-status-badge ${acc.enabled ? 'enabled' : 'disabled'}`}>
          {acc.enabled ? t('api_key_fallback_status_enabled') : t('api_key_fallback_status_disabled')}
        </span>
      </div>
      {acc.spending_limit && <ApiKeySpendingBar limit={acc.spending_limit} />}
      <div className="key-actions">
        <button className="btn-secondary" onClick={() => onToggle(acc.id)}>
          {acc.enabled ? t('api_key_fallback_toggle_off') : t('api_key_fallback_toggle_on')}
        </button>
        <button className="btn-danger" onClick={() => onDelete(acc.id)}>
          {t('api_key_fallback_delete_btn')}
        </button>
      </div>
    </div>
  );
}

function useApiKeyAccounts() {
  const [accounts, setAccounts] = useState<ApiKeyAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadData = () => {
    setLoading(true);
    fetchApiKeyAccounts()
      .then((data) => setAccounts(data))
      .catch((err: unknown) => setError(err instanceof Error ? err.message : 'Lỗi tải'))
      .finally(() => setLoading(false));
  };

  useEffect(() => { loadData(); }, []);

  const handleToggle = (id: string) => {
    toggleApiKeyAccount(id).then(loadData).catch((err: unknown) => {
      setError(err instanceof Error ? err.message : 'Không thể đổi trạng thái');
    });
  };

  const handleDelete = (id: string) => {
    if (!window.confirm('Bạn có chắc chắn muốn xóa khóa API này?')) return;
    deleteApiKeyAccount(id).then(loadData).catch((err: unknown) => {
      setError(err instanceof Error ? err.message : 'Không thể xóa');
    });
  };

  return { accounts, loading, error, loadData, handleToggle, handleDelete };
}

export function ApiKeyFallbackCard() {
  const { accounts, loading, error, loadData, handleToggle, handleDelete } = useApiKeyAccounts();
  const [isModalOpen, setIsModalOpen] = useState(false);

  return (
    <div className="card api-key-fallback-card">
      <div className="card-header">
        <h3>{t('api_key_fallback_title')}</h3>
        <button className="btn-primary" onClick={() => setIsModalOpen(true)}>
          {t('api_key_fallback_add_btn')}
        </button>
      </div>
      {error && <div className="alert-error" role="alert">{error}</div>}
      {loading ? <div>{t('api_key_fallback_loading')}</div> : (
        <div className="account-list">
          {accounts.map((acc) => (
            <ApiKeyItem key={acc.id} acc={acc} onToggle={handleToggle} onDelete={handleDelete} />
          ))}
        </div>
      )}
      <AddApiKeyModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        onSubmit={async (data) => {
          await createApiKeyAccount(data);
          loadData();
        }}
      />
    </div>
  );
}

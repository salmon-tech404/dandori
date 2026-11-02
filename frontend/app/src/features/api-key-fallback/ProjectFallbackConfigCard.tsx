import { useState, useEffect, type ChangeEvent } from 'react';
import { t } from '../../core/i18n';
import { fetchProjectFallbackConfig, updateProjectFallbackConfig } from './api';

export function ProjectFallbackConfigCard({ projectId }: { projectId: string }) {
  const [enabled, setEnabled] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchProjectFallbackConfig(projectId)
      .then((cfg) => setEnabled(cfg.fallback_enabled))
      .catch((err: unknown) => setError(err instanceof Error ? err.message : 'Lỗi tải'));
  }, [projectId]);

  const handleToggle = async (e: ChangeEvent<HTMLInputElement>) => {
    const val = e.target.checked;
    setError(null);
    try {
      await updateProjectFallbackConfig(projectId, { fallback_enabled: val, allowed_accounts: [] });
      setEnabled(val);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Không thể lưu');
    }
  };

  return (
    <div className="card project-fallback-config-card">
      <h4>{t('api_key_fallback_project_config_title')}</h4>
      {error && <div className="alert-error" role="alert">{error}</div>}
      <div className="checkbox-control">
        <label>
          <input type="checkbox" checked={enabled} onChange={handleToggle} />
          <strong>{t('api_key_fallback_project_config_checkbox')}</strong>
        </label>
      </div>
    </div>
  );
}

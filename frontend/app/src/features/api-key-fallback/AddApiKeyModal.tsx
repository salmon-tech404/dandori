import { useState, type FormEvent } from 'react';
import { t } from '../../core/i18n';
import type { SpendingLimit } from './types';

interface AddApiKeyModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (data: {
    provider: string;
    label: string;
    api_key: string;
    spending_limit: Omit<SpendingLimit, 'used_value' | 'period_start'>;
  }) => Promise<void>;
}

function PeriodTypeSelect(props: {
  period: 'daily' | 'weekly' | 'monthly';
  setPeriod: (v: 'daily' | 'weekly' | 'monthly') => void;
  limitType: 'currency' | 'credits';
  setLimitType: (v: 'currency' | 'credits') => void;
}) {
  return (
    <div className="form-row">
      <div>
        <label htmlFor="limit-period">{t('api_key_fallback_period_label')}</label>
        <select
          id="limit-period"
          value={props.period}
          onChange={(e) => props.setPeriod(e.target.value as 'daily' | 'weekly' | 'monthly')}
        >
          <option value="daily">Hàng ngày (Daily)</option>
          <option value="weekly">Hàng tuần (Weekly)</option>
          <option value="monthly">Hàng tháng (Monthly)</option>
        </select>
      </div>
      <div>
        <label htmlFor="limit-type">{t('api_key_fallback_type_label')}</label>
        <select
          id="limit-type"
          value={props.limitType}
          onChange={(e) => props.setLimitType(e.target.value as 'currency' | 'credits')}
        >
          <option value="currency">USD / Tiền tệ</option>
          <option value="credits">Credits</option>
        </select>
      </div>
    </div>
  );
}

function LimitThresholdInputs(props: {
  limitValue: string;
  setLimitValue: (v: string) => void;
  alertThreshold: string;
  setAlertThreshold: (v: string) => void;
}) {
  return (
    <div className="form-row">
      <div>
        <label htmlFor="limit-val">{t('api_key_fallback_max_label')}</label>
        <input
          id="limit-val"
          type="number"
          step="0.1"
          required
          value={props.limitValue}
          onChange={(e) => props.setLimitValue(e.target.value)}
        />
      </div>
      <div>
        <label htmlFor="alert-threshold">{t('api_key_fallback_alert_label')}</label>
        <input
          id="alert-threshold"
          type="number"
          required
          value={props.alertThreshold}
          onChange={(e) => props.setAlertThreshold(e.target.value)}
        />
      </div>
    </div>
  );
}

function ProviderSelect(props: { provider: string; setProvider: (v: string) => void }) {
  return (
    <div className="form-group">
      <label htmlFor="key-provider">{t('api_key_fallback_provider_label')}</label>
      <select
        id="key-provider"
        value={props.provider}
        onChange={(e) => props.setProvider(e.target.value)}
      >
        <option value="claude">Claude (Anthropic)</option>
        <option value="codex">Codex (OpenAI)</option>
        <option value="agy">Antigravity (Google)</option>
      </select>
    </div>
  );
}

function KeyTextInputs(props: {
  label: string;
  setLabel: (v: string) => void;
  apiKey: string;
  setApiKey: (v: string) => void;
}) {
  return (
    <>
      <div className="form-group">
        <label htmlFor="key-label">{t('api_key_fallback_label_label')}</label>
        <input
          id="key-label"
          type="text"
          required
          value={props.label}
          onChange={(e) => props.setLabel(e.target.value)}
          placeholder="VD: Claude Backup Team Dev"
        />
      </div>
      <div className="form-group">
        <label htmlFor="key-secret">{t('api_key_fallback_secret_label')}</label>
        <input
          id="key-secret"
          type="password"
          required
          value={props.apiKey}
          onChange={(e) => props.setApiKey(e.target.value)}
          placeholder="sk-..."
        />
      </div>
    </>
  );
}

interface FormState {
  provider: string;
  label: string;
  apiKey: string;
  period: 'daily' | 'weekly' | 'monthly';
  limitType: 'currency' | 'credits';
  limitValue: string;
  alertThreshold: string;
}

async function doSubmit(
  state: FormState,
  onSubmit: AddApiKeyModalProps['onSubmit'],
  onClose: () => void,
  setError: (e: string | null) => void,
  setSubmitting: (s: boolean) => void
) {
  const val = parseFloat(state.limitValue);
  if (isNaN(val) || val <= 0) {
    setError('Trần chi tiêu là bắt buộc và phải lớn hơn 0');
    return;
  }
  setSubmitting(true);
  try {
    await onSubmit({
      provider: state.provider,
      label: state.label,
      api_key: state.apiKey,
      spending_limit: {
        period: state.period,
        limit_type: state.limitType,
        limit_value: val,
        alert_threshold: parseFloat(state.alertThreshold) || 80,
      },
    });
    onClose();
  } catch (err: unknown) {
    setError(err instanceof Error ? err.message : 'Có lỗi xảy ra');
  } finally {
    setSubmitting(false);
  }
}

function useApiKeyForm(
  onSubmit: AddApiKeyModalProps['onSubmit'],
  onClose: () => void
) {
  const [provider, setProvider] = useState('claude');
  const [label, setLabel] = useState('');
  const [apiKey, setApiKey] = useState('');
  const [period, setPeriod] = useState<'daily' | 'weekly' | 'monthly'>('daily');
  const [limitType, setLimitType] = useState<'currency' | 'credits'>('currency');
  const [limitValue, setLimitValue] = useState('10');
  const [alertThreshold, setAlertThreshold] = useState('80');
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    void doSubmit(
      { provider, label, apiKey, period, limitType, limitValue, alertThreshold },
      onSubmit,
      onClose,
      setError,
      setSubmitting
    );
  };

  return {
    state: { provider, label, apiKey, period, limitType, limitValue, alertThreshold, error, submitting },
    actions: { setProvider, setLabel, setApiKey, setPeriod, setLimitType, setLimitValue, setAlertThreshold, handleSubmit },
  };
}

function SpendingLimitSection(props: {
  period: 'daily' | 'weekly' | 'monthly';
  setPeriod: (v: 'daily' | 'weekly' | 'monthly') => void;
  limitType: 'currency' | 'credits';
  setLimitType: (v: 'currency' | 'credits') => void;
  limitValue: string;
  setLimitValue: (v: string) => void;
  alertThreshold: string;
  setAlertThreshold: (v: string) => void;
}) {
  return (
    <fieldset className="spending-limit-fieldset">
      <legend>{t('api_key_fallback_limit_legend')}</legend>
      <PeriodTypeSelect
        period={props.period}
        setPeriod={props.setPeriod}
        limitType={props.limitType}
        setLimitType={props.setLimitType}
      />
      <LimitThresholdInputs
        limitValue={props.limitValue}
        setLimitValue={props.setLimitValue}
        alertThreshold={props.alertThreshold}
        setAlertThreshold={props.setAlertThreshold}
      />
    </fieldset>
  );
}

export function AddApiKeyModal({ isOpen, onClose, onSubmit }: AddApiKeyModalProps) {
  const { state, actions } = useApiKeyForm(onSubmit, onClose);
  if (!isOpen) return null;

  return (
    <div className="modal-backdrop" role="dialog" aria-modal="true">
      <div className="modal-content">
        <h3>{t('api_key_fallback_modal_title')}</h3>
        {state.error && <div className="alert-error" role="alert">{state.error}</div>}
        <form onSubmit={actions.handleSubmit}>
          <ProviderSelect provider={state.provider} setProvider={actions.setProvider} />
          <KeyTextInputs
            label={state.label}
            setLabel={actions.setLabel}
            apiKey={state.apiKey}
            setApiKey={actions.setApiKey}
          />
          <SpendingLimitSection
            period={state.period}
            setPeriod={actions.setPeriod}
            limitType={state.limitType}
            setLimitType={actions.setLimitType}
            limitValue={state.limitValue}
            setLimitValue={actions.setLimitValue}
            alertThreshold={state.alertThreshold}
            setAlertThreshold={actions.setAlertThreshold}
          />
          <div className="modal-actions">
            <button type="button" onClick={onClose} disabled={state.submitting}>
              {t('api_key_fallback_cancel_btn')}
            </button>
            <button type="submit" disabled={state.submitting}>
              {state.submitting ? t('api_key_fallback_saving') : t('api_key_fallback_save_btn')}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

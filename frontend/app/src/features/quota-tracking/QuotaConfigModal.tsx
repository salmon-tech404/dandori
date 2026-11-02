import { useState, type FormEvent } from "react";
import { t } from "../../core/i18n";
import type { AccountQuotaConfigPayload } from "./types";

interface QuotaConfigModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (payload: AccountQuotaConfigPayload) => Promise<void>;
  initial5h?: number | null;
  initialWeekly?: number | null;
  initialThreshold?: number;
}

interface FormFieldProps {
  labelKey: "quota_limit_5h_label" | "quota_limit_weekly_label" | "quota_alert_threshold_label";
  value: string;
  onChange: (val: string) => void;
  placeholder: string;
}

function FormField({ labelKey, value, onChange, placeholder }: FormFieldProps) {
  return (
    <label className="block text-xs font-medium text-slate-700 dark:text-slate-300">
      <span>{t(labelKey)}</span>
      <input
        type="number"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="mt-1 w-full rounded border border-slate-300 bg-white px-2.5 py-1.5 text-xs text-slate-900 focus:border-blue-500 focus:outline-none dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
      />
    </label>
  );
}

interface ModalActionsProps {
  isSaving: boolean;
  onClose: () => void;
}

function ModalActions({ isSaving, onClose }: ModalActionsProps) {
  return (
    <div className="flex justify-end gap-2 pt-2">
      <button
        type="button"
        onClick={onClose}
        className="rounded border border-slate-300 px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-100 dark:border-slate-600 dark:text-slate-300 dark:hover:bg-slate-800"
      >
        {t("quota_cancel_btn")}
      </button>
      <button
        type="submit"
        disabled={isSaving}
        className="rounded bg-blue-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-blue-700 disabled:opacity-50"
      >
        {t("quota_save_btn")}
      </button>
    </div>
  );
}

interface ConfigFormProps {
  onClose: () => void;
  onSave: (payload: AccountQuotaConfigPayload) => Promise<void>;
  initial5h?: number | null;
  initialWeekly?: number | null;
  initialThreshold: number;
}

function ConfigForm({ onClose, onSave, initial5h, initialWeekly, initialThreshold }: ConfigFormProps) {
  const [limit5h, setLimit5h] = useState(initial5h?.toString() ?? "");
  const [limitWeekly, setLimitWeekly] = useState(initialWeekly?.toString() ?? "");
  const [threshold, setThreshold] = useState(initialThreshold.toString());
  const [isSaving, setIsSaving] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setIsSaving(true);
    try {
      await onSave({
        limit_5h: limit5h ? parseInt(limit5h, 10) : null,
        limit_weekly: limitWeekly ? parseInt(limitWeekly, 10) : null,
        alert_threshold_percent: threshold ? parseInt(threshold, 10) : 80,
      });
      onClose();
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="mt-3 space-y-3">
      <FormField labelKey="quota_limit_5h_label" value={limit5h} onChange={setLimit5h} placeholder="10" />
      <FormField labelKey="quota_limit_weekly_label" value={limitWeekly} onChange={setLimitWeekly} placeholder="50" />
      <FormField labelKey="quota_alert_threshold_label" value={threshold} onChange={setThreshold} placeholder="80" />
      <ModalActions isSaving={isSaving} onClose={onClose} />
    </form>
  );
}

export function QuotaConfigModal({
  isOpen,
  onClose,
  onSave,
  initial5h,
  initialWeekly,
  initialThreshold = 80,
}: QuotaConfigModalProps) {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
      <div className="w-full max-w-sm rounded-lg bg-white p-4 shadow-xl dark:bg-slate-900">
        <h3 className="text-sm font-semibold text-slate-900 dark:text-slate-100">
          {t("quota_modal_title")}
        </h3>
        <ConfigForm
          onClose={onClose}
          onSave={onSave}
          initial5h={initial5h}
          initialWeekly={initialWeekly}
          initialThreshold={initialThreshold}
        />
      </div>
    </div>
  );
}

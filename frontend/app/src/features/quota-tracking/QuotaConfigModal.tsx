import { useState, type FormEvent } from "react";
import { t } from "../../core/i18n";
import { Modal } from "../../ui/components/Modal";
import { Button } from "../../ui/components/Button";
import { Input } from "../../ui/components/Input";
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
    <label style={{ display: "block", fontSize: "var(--fs-12)", fontWeight: 500, color: "var(--fg)" }}>
      <span>{t(labelKey)}</span>
      <Input
        type="number"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        style={{ marginTop: 4 }}
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
    <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--sp-8)", paddingTop: "var(--sp-8)" }}>
      <Button
        type="button"
        variant="secondary"
        size="sm"
        onClick={onClose}
      >
        {t("quota_cancel_btn")}
      </Button>
      <Button
        type="submit"
        variant="primary"
        size="sm"
        disabled={isSaving}
      >
        {t("quota_save_btn")}
      </Button>
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
    <form onSubmit={handleSubmit} style={{ marginTop: "var(--sp-12)", display: "flex", flexDirection: "column", gap: "var(--sp-12)" }}>
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
  return (
    <Modal open={isOpen} onClose={onClose} title={t("quota_modal_title")} size="sm">
      <ConfigForm
        onClose={onClose}
        onSave={onSave}
        initial5h={initial5h}
        initialWeekly={initialWeekly}
        initialThreshold={initialThreshold}
      />
    </Modal>
  );
}

/**
 * Form cấu hình cho module auto-lessons (P3-11).
 */

import React, { useState } from "react";
import { t } from "../../core/i18n";
import type { AutoLessonsConfig, UpdateAutoLessonsConfigRequest } from "./types";

interface AutoLessonsConfigFormProps {
  config: AutoLessonsConfig;
  onSave: (payload: UpdateAutoLessonsConfigRequest) => Promise<void>;
}

const ToggleItem: React.FC<{
  id: string;
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}> = ({ id, label, checked, onChange }) => (
  <div className="flex items-center space-x-2">
    <input
      type="checkbox"
      id={id}
      checked={checked}
      onChange={(e) => onChange(e.target.checked)}
    />
    <label htmlFor={id} className="text-sm">
      {label}
    </label>
  </div>
);

const BudgetInput: React.FC<{
  budget: number;
  onChange: (v: number) => void;
}> = ({ budget, onChange }) => (
  <div className="space-y-1">
    <label htmlFor="al-budget" className="text-xs text-muted-foreground">
      {t("auto_lessons_max_budget")}
    </label>
    <input
      type="number"
      id="al-budget"
      value={budget}
      onChange={(e) => onChange(Number(e.target.value))}
      className="w-full rounded border px-3 py-1 text-sm"
      min={100}
      max={10000}
    />
  </div>
);

const FormFields: React.FC<{
  enabled: boolean;
  setEnabled: (v: boolean) => void;
  onDone: boolean;
  setOnDone: (v: boolean) => void;
  onFailed: boolean;
  setOnFailed: (v: boolean) => void;
  budget: number;
  setBudget: (v: number) => void;
}> = (props) => (
  <>
    <ToggleItem
      id="al-enabled"
      label={t("auto_lessons_enable")}
      checked={props.enabled}
      onChange={props.setEnabled}
    />
    <div className="grid grid-cols-2 gap-4">
      <ToggleItem
        id="al-done"
        label={t("auto_lessons_trigger_done")}
        checked={props.onDone}
        onChange={props.setOnDone}
      />
      <ToggleItem
        id="al-failed"
        label={t("auto_lessons_trigger_failed")}
        checked={props.onFailed}
        onChange={props.setOnFailed}
      />
    </div>
    <BudgetInput budget={props.budget} onChange={props.setBudget} />
  </>
);

function useConfigForm(
  config: AutoLessonsConfig,
  onSave: (payload: UpdateAutoLessonsConfigRequest) => Promise<void>
) {
  const [enabled, setEnabled] = useState(config.enabled);
  const [onDone, setOnDone] = useState(config.trigger_on_done);
  const [onFailed, setOnFailed] = useState(config.trigger_on_failed);
  const [budget, setBudget] = useState(config.max_token_budget);
  const [saving, setSaving] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    try {
      await onSave({
        enabled,
        trigger_on_done: onDone,
        trigger_on_failed: onFailed,
        max_token_budget: Number(budget),
      });
    } finally {
      setSaving(false);
    }
  };

  return {
    enabled,
    setEnabled,
    onDone,
    setOnDone,
    onFailed,
    setOnFailed,
    budget,
    setBudget,
    saving,
    handleSubmit,
  };
}

export const AutoLessonsConfigForm: React.FC<AutoLessonsConfigFormProps> = ({
  config,
  onSave,
}) => {
  const f = useConfigForm(config, onSave);

  return (
    <form onSubmit={f.handleSubmit} className="space-y-4 rounded border p-4">
      <FormFields
        enabled={f.enabled}
        setEnabled={f.setEnabled}
        onDone={f.onDone}
        setOnDone={f.setOnDone}
        onFailed={f.onFailed}
        setOnFailed={f.setOnFailed}
        budget={f.budget}
        setBudget={f.setBudget}
      />
      <button
        type="submit"
        disabled={f.saving}
        className="rounded bg-primary px-4 py-2 text-sm text-primary-foreground disabled:opacity-50"
      >
        {t("auto_lessons_save_config")}
      </button>
    </form>
  );
};

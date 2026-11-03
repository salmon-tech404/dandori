/**
 * Form cấu hình cho module auto-lessons (P3-11).
 */

import React, { useState } from "react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { Input } from "../../ui/components/Input";
import { Checkbox } from "../../ui/components/Checkbox";
import type { AutoLessonsConfig, UpdateAutoLessonsConfigRequest } from "./types";

interface AutoLessonsConfigFormProps {
  config: AutoLessonsConfig;
  onSave: (payload: UpdateAutoLessonsConfigRequest) => Promise<void>;
}

const inputStyle: React.CSSProperties = {
  width: "100%",
  height: "var(--btn-height)",
  padding: "0 12px",
  fontSize: "var(--fs-13)",
  fontFamily: "var(--font-sans)",
  color: "var(--fg)",
  backgroundColor: "var(--sf)",
  border: "1px solid var(--ln)",
  borderRadius: "var(--radius-8)",
  outline: "none",
  boxSizing: "border-box",
};

const labelStyle: React.CSSProperties = {
  display: "block",
  fontSize: "var(--fs-12)",
  fontWeight: 500,
  color: "var(--mu)",
  marginBottom: "var(--sp-4)",
};

const ToggleItem: React.FC<{
  id: string;
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}> = ({ id, label, checked, onChange }) => (
  <Checkbox
    id={id}
    checked={checked}
    onChange={(e) => onChange(e.target.checked)}
    label={label}
  />
);

const BudgetInput: React.FC<{
  budget: number;
  onChange: (v: number) => void;
}> = ({ budget, onChange }) => (
  <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-4)" }}>
    <label htmlFor="al-budget" style={labelStyle}>
      {t("auto_lessons_max_budget")}
    </label>
    <Input
      type="number"
      id="al-budget"
      value={budget}
      onChange={(e) => onChange(Number(e.target.value))}
      style={inputStyle}
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
  <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)" }}>
    <ToggleItem
      id="al-enabled"
      label={t("auto_lessons_enable")}
      checked={props.enabled}
      onChange={props.setEnabled}
    />
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--sp-12)" }}>
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
  </div>
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
    <form onSubmit={f.handleSubmit} style={{ backgroundColor: "var(--sf2)", borderRadius: "var(--radius-8)", border: "1px solid var(--ln)", padding: "var(--sp-16)", display: "flex", flexDirection: "column", gap: "var(--sp-16)" }}>
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
      <div>
        <Button
          type="submit"
          variant="primary"
          size="sm"
          disabled={f.saving}
          loading={f.saving}
        >
          {t("auto_lessons_save_config")}
        </Button>
      </div>
    </form>
  );
};

import React, { useEffect, useState } from "react";
import { t } from "../../../core/i18n";
import { Button } from "../../../ui/components/Button";
import { Input } from "../../../ui/components/Input";
import { Textarea } from "../../../ui/components/Textarea";
import { Checkbox } from "../../../ui/components/Checkbox";
import { docAgentApi } from "../api/docAgentApi";
import type { DocAgentConfig } from "../types";

export interface DocAgentSettingsProps {
  projectId: string;
}

interface FormState {
  enabled: boolean;
  promptTemplate: string;
  timeout: number;
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

const textareaStyle: React.CSSProperties = {
  width: "100%",
  padding: "8px 12px",
  fontSize: "var(--fs-13)",
  fontFamily: "var(--font-sans)",
  color: "var(--fg)",
  backgroundColor: "var(--sf)",
  border: "1px solid var(--ln)",
  borderRadius: "var(--radius-8)",
  outline: "none",
  boxSizing: "border-box",
  resize: "vertical",
};

const labelStyle: React.CSSProperties = {
  display: "block",
  fontSize: "var(--fs-12)",
  fontWeight: 500,
  color: "var(--mu)",
  marginBottom: "var(--sp-4)",
};

const BasicFields: React.FC<{
  enabled: boolean;
  prompt: string;
  onChange: (f: Partial<FormState>) => void;
}> = ({ enabled, prompt, onChange }) => (
  <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)", marginBottom: "var(--sp-12)" }}>
    <Checkbox
      checked={enabled}
      onChange={(e) => onChange({ enabled: e.target.checked })}
      label={<span style={{ fontWeight: 500 }}>{t("doc_agent_enabled_label")}</span>}
    />
    <div>
      <label style={labelStyle}>
        {t("doc_agent_prompt_label")}
      </label>
      <Textarea
        rows={3}
        style={textareaStyle}
        placeholder={t("doc_agent_prompt_placeholder")}
        value={prompt}
        onChange={(e) => onChange({ promptTemplate: e.target.value })}
      />
    </div>
  </div>
);

const TimeoutField: React.FC<{
  timeout: number;
  onChange: (f: Partial<FormState>) => void;
}> = ({ timeout, onChange }) => (
  <div style={{ marginBottom: "var(--sp-16)" }}>
    <label style={labelStyle}>
      {t("doc_agent_timeout_label")}
    </label>
    <Input
      type="number"
      min={30}
      max={3600}
      style={inputStyle}
      value={timeout}
      onChange={(e) => onChange({ timeout: Number(e.target.value) })}
    />
  </div>
);

function useDocAgentForm(projectId: string) {
  const [form, setForm] = useState<FormState>({
    enabled: true,
    promptTemplate: "",
    timeout: 600,
  });
  const [saving, setSaving] = useState(false);
  const [msg, setMsg] = useState<string | null>(null);

  useEffect(() => {
    docAgentApi
      .getConfig(projectId)
      .then((cfg: DocAgentConfig) => {
        setForm({
          enabled: cfg.enabled,
          promptTemplate: cfg.prompt_template ?? "",
          timeout: cfg.timeout_seconds,
        });
      })
      .catch(() => {});
  }, [projectId]);

  const save = async () => {
    setSaving(true);
    setMsg(null);
    try {
      await docAgentApi.updateConfig(projectId, {
        enabled: form.enabled,
        prompt_template: form.promptTemplate || undefined,
        timeout_seconds: form.timeout,
      });
      setMsg(t("doc_agent_save_success"));
    } finally {
      setSaving(false);
    }
  };

  return { form, setForm, saving, msg, save };
}

export const DocAgentSettings: React.FC<DocAgentSettingsProps> = ({ projectId }) => {
  const { form, setForm, saving, msg, save } = useDocAgentForm(projectId);

  return (
    <div style={{ backgroundColor: "var(--sf)", borderRadius: "var(--radius-10)", border: "1px solid var(--ln)", padding: "var(--sp-16)", marginBottom: "var(--sp-16)" }}>
      <h3 style={{ margin: "0 0 var(--sp-16) 0", fontSize: "var(--fs-16)", fontWeight: 600, color: "var(--fg)" }}>
        {t("doc_agent_settings_title")}
      </h3>
      <BasicFields
        enabled={form.enabled}
        prompt={form.promptTemplate}
        onChange={(partial) => setForm((prev) => ({ ...prev, ...partial }))}
      />
      <TimeoutField
        timeout={form.timeout}
        onChange={(partial) => setForm((prev) => ({ ...prev, ...partial }))}
      />
      {msg && <p style={{ fontSize: "var(--fs-13)", color: "var(--gr)", fontWeight: 500, marginBottom: "var(--sp-12)" }}>{msg}</p>}
      <Button
        type="button"
        variant="primary"
        size="sm"
        disabled={saving}
        loading={saving}
        onClick={save}
      >
        {t("doc_agent_save_btn")}
      </Button>
    </div>
  );
};

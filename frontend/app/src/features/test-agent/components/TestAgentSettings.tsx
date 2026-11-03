import React, { useEffect, useState } from "react";
import { t } from "../../../core/i18n";
import { Button } from "../../../ui/components/Button";
import { testAgentApi } from "../api/testAgentApi";
import type { TestAgentConfig } from "../types";

export interface TestAgentSettingsProps {
  projectId: string;
}

interface FormState {
  enabled: boolean;
  suggestedCommand: string;
  maxRuns: number;
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

const labelStyle: React.CSSProperties = {
  display: "block",
  fontSize: "var(--fs-12)",
  fontWeight: 500,
  color: "var(--mu)",
  marginBottom: "var(--sp-4)",
};

const BasicFields: React.FC<{
  enabled: boolean;
  cmd: string;
  onChange: (f: Partial<FormState>) => void;
}> = ({ enabled, cmd, onChange }) => (
  <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)", marginBottom: "var(--sp-12)" }}>
    <label style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)", cursor: "pointer", fontSize: "var(--fs-14)", color: "var(--fg)" }}>
      <input
        type="checkbox"
        checked={enabled}
        onChange={(e) => onChange({ enabled: e.target.checked })}
        style={{ width: "16px", height: "16px", cursor: "pointer" }}
      />
      <span style={{ fontWeight: 500 }}>{t("test_agent_enabled_label")}</span>
    </label>
    <div>
      <label style={labelStyle}>
        {t("test_agent_command_label")}
      </label>
      <input
        type="text"
        style={inputStyle}
        placeholder={t("test_agent_command_placeholder")}
        value={cmd}
        onChange={(e) => onChange({ suggestedCommand: e.target.value })}
      />
    </div>
  </div>
);

const NumericFields: React.FC<{
  maxRuns: number;
  timeout: number;
  onChange: (f: Partial<FormState>) => void;
}> = ({ maxRuns, timeout, onChange }) => (
  <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--sp-12)", marginBottom: "var(--sp-16)" }}>
    <div>
      <label style={labelStyle}>
        {t("test_agent_max_runs_label")}
      </label>
      <input
        type="number"
        min={1}
        max={5}
        style={inputStyle}
        value={maxRuns}
        onChange={(e) => onChange({ maxRuns: Number(e.target.value) })}
      />
    </div>
    <div>
      <label style={labelStyle}>
        {t("test_agent_timeout_label")}
      </label>
      <input
        type="number"
        min={30}
        max={3600}
        style={inputStyle}
        value={timeout}
        onChange={(e) => onChange({ timeout: Number(e.target.value) })}
      />
    </div>
  </div>
);

function useTestAgentForm(projectId: string) {
  const [form, setForm] = useState<FormState>({
    enabled: true,
    suggestedCommand: "",
    maxRuns: 1,
    timeout: 600,
  });
  const [saving, setSaving] = useState(false);
  const [msg, setMsg] = useState<string | null>(null);

  useEffect(() => {
    testAgentApi.getConfig(projectId).then((cfg: TestAgentConfig) => {
      setForm({
        enabled: cfg.enabled,
        suggestedCommand: cfg.suggested_command ?? "",
        maxRuns: cfg.max_test_runs_per_worker,
        timeout: cfg.timeout_seconds,
      });
    }).catch(() => {});
  }, [projectId]);

  const handleSave = async () => {
    setSaving(true);
    setMsg(null);
    try {
      await testAgentApi.updateConfig(projectId, {
        enabled: form.enabled,
        suggested_command: form.suggestedCommand,
        max_test_runs_per_worker: form.maxRuns,
        timeout_seconds: form.timeout,
      });
      setMsg(t("test_agent_save_success"));
    } finally {
      setSaving(false);
    }
  };

  return { form, setForm, saving, msg, handleSave };
}

export const TestAgentSettings: React.FC<TestAgentSettingsProps> = ({ projectId }) => {
  const { form, setForm, saving, msg, handleSave } = useTestAgentForm(projectId);

  return (
    <div style={{ backgroundColor: "var(--sf)", borderRadius: "var(--radius-10)", border: "1px solid var(--ln)", padding: "var(--sp-16)", marginBottom: "var(--sp-16)" }}>
      <h3 style={{ margin: "0 0 var(--sp-16) 0", fontSize: "var(--fs-16)", fontWeight: 600, color: "var(--fg)" }}>
        {t("test_agent_settings_title")}
      </h3>
      <BasicFields enabled={form.enabled} cmd={form.suggestedCommand} onChange={(p) => setForm((prev) => ({ ...prev, ...p }))} />
      <NumericFields maxRuns={form.maxRuns} timeout={form.timeout} onChange={(p) => setForm((prev) => ({ ...prev, ...p }))} />
      {msg && <div style={{ fontSize: "var(--fs-13)", color: "var(--gr)", fontWeight: 500, marginBottom: "var(--sp-12)" }}>{msg}</div>}
      <Button
        type="button"
        variant="primary"
        size="sm"
        disabled={saving}
        loading={saving}
        onClick={handleSave}
      >
        {t("test_agent_save_btn")}
      </Button>
    </div>
  );
};

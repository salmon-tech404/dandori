import React, { useEffect, useState } from "react";
import { t } from "../../../core/i18n";
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

const BasicFields: React.FC<{
  enabled: boolean;
  cmd: string;
  onChange: (f: Partial<FormState>) => void;
}> = ({ enabled, cmd, onChange }) => (
  <div className="space-y-3">
    <label className="flex items-center gap-2">
      <input
        type="checkbox"
        checked={enabled}
        onChange={(e) => onChange({ enabled: e.target.checked })}
      />
      <span>{t("test_agent_enabled_label")}</span>
    </label>
    <div>
      <label className="block text-xs font-medium text-neutral-600 mb-1">
        {t("test_agent_command_label")}
      </label>
      <input
        type="text"
        className="w-full border rounded px-2 py-1 text-xs"
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
  <div className="grid grid-cols-2 gap-2">
    <div>
      <label className="block text-xs font-medium text-neutral-600 mb-1">
        {t("test_agent_max_runs_label")}
      </label>
      <input
        type="number"
        min={1}
        max={5}
        className="w-full border rounded px-2 py-1 text-xs"
        value={maxRuns}
        onChange={(e) => onChange({ maxRuns: Number(e.target.value) })}
      />
    </div>
    <div>
      <label className="block text-xs font-medium text-neutral-600 mb-1">
        {t("test_agent_timeout_label")}
      </label>
      <input
        type="number"
        min={30}
        max={3600}
        className="w-full border rounded px-2 py-1 text-xs"
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
    <div className="p-4 bg-white border rounded space-y-4 text-sm">
      <h3 className="font-semibold text-base">{t("test_agent_settings_title")}</h3>
      <BasicFields enabled={form.enabled} cmd={form.suggestedCommand} onChange={(p) => setForm((prev) => ({ ...prev, ...p }))} />
      <NumericFields maxRuns={form.maxRuns} timeout={form.timeout} onChange={(p) => setForm((prev) => ({ ...prev, ...p }))} />
      {msg && <div className="text-xs text-green-600 font-medium">{msg}</div>}
      <button
        type="button"
        disabled={saving}
        onClick={handleSave}
        className="px-3 py-1.5 bg-blue-600 text-white rounded text-xs hover:bg-blue-700 disabled:opacity-50"
      >
        {saving ? t("test_agent_saving") : t("test_agent_save_btn")}
      </button>
    </div>
  );
};

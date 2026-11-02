import React, { useEffect, useState } from "react";
import { t } from "../../../core/i18n";
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

const BasicFields: React.FC<{
  enabled: boolean;
  prompt: string;
  onChange: (f: Partial<FormState>) => void;
}> = ({ enabled, prompt, onChange }) => (
  <div className="space-y-3">
    <label className="flex items-center gap-2">
      <input
        type="checkbox"
        checked={enabled}
        onChange={(e) => onChange({ enabled: e.target.checked })}
      />
      <span>{t("doc_agent_enabled_label")}</span>
    </label>
    <div>
      <label className="block text-xs font-medium text-neutral-600 mb-1">
        {t("doc_agent_prompt_label")}
      </label>
      <textarea
        rows={3}
        className="w-full border rounded px-2 py-1 text-xs"
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
  <div>
    <label className="block text-xs font-medium text-neutral-600 mb-1">
      {t("doc_agent_timeout_label")}
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
    <div className="p-4 border rounded bg-white space-y-4 max-w-xl">
      <h3 className="text-base font-semibold text-neutral-800">
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
      {msg && <p className="text-xs text-green-600 font-medium">{msg}</p>}
      <button
        type="button"
        disabled={saving}
        onClick={save}
        className="px-4 py-1.5 bg-blue-600 text-white rounded text-xs hover:bg-blue-700 disabled:opacity-50"
      >
        {saving ? t("doc_agent_saving") : t("doc_agent_save_btn")}
      </button>
    </div>
  );
};

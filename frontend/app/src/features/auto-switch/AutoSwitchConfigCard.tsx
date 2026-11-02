import { useState, type FormEvent } from "react";
import { Sliders, Save } from "lucide-react";
import { t } from "../../core/i18n";
import type { AutoSwitchConfig, UpdateAutoSwitchConfigPayload } from "./types";

interface AutoSwitchConfigCardProps {
  config: AutoSwitchConfig;
  onSave: (payload: UpdateAutoSwitchConfigPayload) => Promise<void>;
}

function ConfigHeader() {
  return (
    <div className="flex items-center gap-2 border-b border-slate-200 pb-2 dark:border-slate-800">
      <Sliders className="h-4 w-4 text-blue-500" />
      <div>
        <h4 className="text-xs font-semibold text-slate-800 dark:text-slate-200">
          {t("auto_switch_title")}
        </h4>
        <p className="text-[11px] text-slate-500 dark:text-slate-400">
          {t("auto_switch_desc")}
        </p>
      </div>
    </div>
  );
}

interface CrossProviderProps {
  allowCross: boolean;
  setAllowCross: (v: boolean) => void;
  allowedProviders: string;
  setAllowedProviders: (v: string) => void;
}

function CrossProviderField({
  allowCross,
  setAllowCross,
  allowedProviders,
  setAllowedProviders,
}: CrossProviderProps) {
  return (
    <div className="space-y-2">
      <label className="flex items-center gap-2 text-xs font-medium text-slate-700 dark:text-slate-300">
        <input
          type="checkbox"
          checked={allowCross}
          onChange={(e) => setAllowCross(e.target.checked)}
          className="rounded border-slate-300 text-blue-600 focus:ring-blue-500 dark:border-slate-600 dark:bg-slate-800"
        />
        <span>{t("auto_switch_allow_cross_label")}</span>
      </label>

      {allowCross && (
        <label className="block text-xs font-medium text-slate-700 dark:text-slate-300">
          <span>{t("auto_switch_allowed_providers_label")}</span>
          <input
            type="text"
            value={allowedProviders}
            onChange={(e) => setAllowedProviders(e.target.value)}
            placeholder="claude, gemini, agy"
            className="mt-1 w-full rounded border border-slate-300 bg-white px-2.5 py-1.5 text-xs text-slate-900 focus:border-blue-500 focus:outline-none dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
          />
        </label>
      )}
    </div>
  );
}

interface FormBodyProps {
  maxSwitches: string;
  setMaxSwitches: (v: string) => void;
  allowCross: boolean;
  setAllowCross: (v: boolean) => void;
  allowedProviders: string;
  setAllowedProviders: (v: string) => void;
  isSaving: boolean;
  onSubmit: (e: FormEvent) => void;
}

function SaveButton({ isSaving }: { isSaving: boolean }) {
  return (
    <div className="flex justify-end pt-1">
      <button
        type="submit"
        disabled={isSaving}
        className="flex items-center gap-1.5 rounded bg-blue-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-blue-700 disabled:opacity-50"
      >
        <Save className="h-3 w-3" />
        <span>{t("auto_switch_save_btn")}</span>
      </button>
    </div>
  );
}

function FormBody({
  maxSwitches,
  setMaxSwitches,
  allowCross,
  setAllowCross,
  allowedProviders,
  setAllowedProviders,
  isSaving,
  onSubmit,
}: FormBodyProps) {
  return (
    <form onSubmit={onSubmit} className="space-y-3 pt-2">
      <label className="block text-xs font-medium text-slate-700 dark:text-slate-300">
        <span>{t("auto_switch_max_switches_label")}</span>
        <input
          type="number"
          min="1"
          max="10"
          value={maxSwitches}
          onChange={(e) => setMaxSwitches(e.target.value)}
          className="mt-1 w-full rounded border border-slate-300 bg-white px-2.5 py-1.5 text-xs text-slate-900 focus:border-blue-500 focus:outline-none dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
        />
      </label>

      <CrossProviderField
        allowCross={allowCross}
        setAllowCross={setAllowCross}
        allowedProviders={allowedProviders}
        setAllowedProviders={setAllowedProviders}
      />

      <SaveButton isSaving={isSaving} />
    </form>
  );
}

export function AutoSwitchConfigCard({ config, onSave }: AutoSwitchConfigCardProps) {
  const [maxSwitches, setMaxSwitches] = useState(config.max_switches.toString());
  const [allowCross, setAllowCross] = useState(config.allow_cross_provider);
  const [allowedProviders, setAllowedProviders] = useState(config.allowed_providers.join(", "));
  const [isSaving, setIsSaving] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setIsSaving(true);
    try {
      const providers = allowedProviders
        .split(",")
        .map((p) => p.trim())
        .filter(Boolean);

      await onSave({
        max_switches: parseInt(maxSwitches, 10) || 2,
        allow_cross_provider: allowCross,
        allowed_providers: providers,
      });
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="rounded-lg border border-slate-200 bg-white p-3.5 shadow-xs dark:border-slate-800 dark:bg-slate-900">
      <ConfigHeader />
      <FormBody
        maxSwitches={maxSwitches}
        setMaxSwitches={setMaxSwitches}
        allowCross={allowCross}
        setAllowCross={setAllowCross}
        allowedProviders={allowedProviders}
        setAllowedProviders={setAllowedProviders}
        isSaving={isSaving}
        onSubmit={handleSubmit}
      />
    </div>
  );
}

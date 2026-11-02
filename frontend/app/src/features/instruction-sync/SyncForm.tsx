import { t } from "../../core/i18n";

export interface SyncFormProps {
  provider: string;
  targetFilename: string;
  strategy: string;
  isEnabled: boolean;
  loading: boolean;
  statusMsg: string | null;
  onProviderChange: (p: string) => void;
  onTargetFilenameChange: (f: string) => void;
  onStrategyChange: (s: string) => void;
  onEnabledChange: (e: boolean) => void;
  onPreview: () => void;
  onSave: () => void;
}

function ProviderSelect({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  return (
    <div>
      <label htmlFor="instruction-provider-select" className="block text-sm font-medium text-gray-700 dark:text-gray-300">
        {t("instruction_sync_provider_label")}
      </label>
      <select
        id="instruction-provider-select"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="mt-1 block w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
      >
        <option value="claude">{t("sync_target_claude")}</option>
        <option value="codex">{t("sync_target_codex")}</option>
        <option value="cursor">{t("sync_target_cursor")}</option>
      </select>
    </div>
  );
}

function TargetFileInput({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  return (
    <div>
      <label htmlFor="instruction-target-filename-input" className="block text-sm font-medium text-gray-700 dark:text-gray-300">
        {t("instruction_sync_target_file_label")}
      </label>
      <input
        id="instruction-target-filename-input"
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 text-sm text-gray-900 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
      />
    </div>
  );
}

function StrategySelect({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  return (
    <div>
      <label htmlFor="instruction-strategy-select" className="block text-sm font-medium text-gray-700 dark:text-gray-300">
        {t("instruction_sync_strategy_label")}
      </label>
      <select
        id="instruction-strategy-select"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="mt-1 block w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
      >
        <option value="skip">{t("instruction_sync_strategy_skip")}</option>
        <option value="append">{t("instruction_sync_strategy_append")}</option>
      </select>
    </div>
  );
}

function EnableCheckbox({ checked, onChange }: { checked: boolean; onChange: (c: boolean) => void }) {
  return (
    <div className="flex items-center gap-2 pt-2">
      <input
        id="instruction-sync-enable-checkbox"
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="h-4 w-4 rounded border-gray-300 text-blue-600 dark:border-gray-600"
      />
      <label htmlFor="instruction-sync-enable-checkbox" className="text-sm font-medium text-gray-700 dark:text-gray-300">
        {t("instruction_sync_enabled_label")}
      </label>
    </div>
  );
}

function ActionButtons({ loading, onPreview, onSave }: { loading: boolean; onPreview: () => void; onSave: () => void }) {
  return (
    <div className="flex items-center gap-3 pt-4">
      <button
        type="button"
        onClick={onPreview}
        disabled={loading}
        className="rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600"
      >
        {t("instruction_sync_btn_preview")}
      </button>
      <button
        type="button"
        onClick={onSave}
        disabled={loading}
        className="rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
      >
        {t("instruction_sync_btn_save")}
      </button>
    </div>
  );
}

export function SyncForm(p: SyncFormProps) {
  return (
    <div className="max-w-2xl space-y-4 rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-700 dark:bg-gray-800">
      <ProviderSelect value={p.provider} onChange={p.onProviderChange} />
      <TargetFileInput value={p.targetFilename} onChange={p.onTargetFilenameChange} />
      <StrategySelect value={p.strategy} onChange={p.onStrategyChange} />
      <EnableCheckbox checked={p.isEnabled} onChange={p.onEnabledChange} />
      {p.statusMsg && (
        <p className="text-xs font-medium text-emerald-600 dark:text-emerald-400">
          {p.statusMsg}
        </p>
      )}
      <ActionButtons loading={p.loading} onPreview={p.onPreview} onSave={p.onSave} />
    </div>
  );
}

import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { Input } from "../../ui/components/Input";
import { Select } from "../../ui/components/Select";
import { Checkbox } from "../../ui/components/Checkbox";

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
      <label htmlFor="instruction-provider-select" style={{ display: "block", fontSize: "var(--fs-14)", fontWeight: 500, color: "var(--fg)", marginBottom: 4 }}>
        {t("instruction_sync_provider_label")}
      </label>
      <Select
        id="instruction-provider-select"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      >
        <option value="claude">{t("sync_target_claude")}</option>
        <option value="codex">{t("sync_target_codex")}</option>
        <option value="cursor">{t("sync_target_cursor")}</option>
      </Select>
    </div>
  );
}

function TargetFileInput({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  return (
    <div>
      <label htmlFor="instruction-target-filename-input" style={{ display: "block", fontSize: "var(--fs-14)", fontWeight: 500, color: "var(--fg)", marginBottom: 4 }}>
        {t("instruction_sync_target_file_label")}
      </label>
      <Input
        id="instruction-target-filename-input"
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  );
}

function StrategySelect({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  return (
    <div>
      <label htmlFor="instruction-strategy-select" style={{ display: "block", fontSize: "var(--fs-14)", fontWeight: 500, color: "var(--fg)", marginBottom: 4 }}>
        {t("instruction_sync_strategy_label")}
      </label>
      <Select
        id="instruction-strategy-select"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      >
        <option value="skip">{t("instruction_sync_strategy_skip")}</option>
        <option value="append">{t("instruction_sync_strategy_append")}</option>
      </Select>
    </div>
  );
}

function EnableCheckbox({ checked, onChange }: { checked: boolean; onChange: (c: boolean) => void }) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)", paddingTop: "var(--sp-8)" }}>
      <Checkbox id="instruction-sync-enable-checkbox" label={t("instruction_sync_enabled_label")} checked={checked} onChange={(e) => onChange(e.target.checked)} />
    </div>
  );
}

function ActionButtons({ loading, onPreview, onSave }: { loading: boolean; onPreview: () => void; onSave: () => void }) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-12)", paddingTop: "var(--sp-16)" }}>
      <Button variant="secondary" type="button" onClick={onPreview} disabled={loading}>
        {t("instruction_sync_btn_preview")}
      </Button>
      <Button variant="primary" type="button" onClick={onSave} disabled={loading}>
        {t("instruction_sync_btn_save")}
      </Button>
    </div>
  );
}

export function SyncForm(p: SyncFormProps) {
  return (
    <div style={{ maxWidth: 640, display: "flex", flexDirection: "column", gap: "var(--sp-16)", borderRadius: "var(--radius-8)", border: "1px solid var(--ln)", backgroundColor: "var(--sf)", padding: "var(--sp-24)" }}>
      <ProviderSelect value={p.provider} onChange={p.onProviderChange} />
      <TargetFileInput value={p.targetFilename} onChange={p.onTargetFilenameChange} />
      <StrategySelect value={p.strategy} onChange={p.onStrategyChange} />
      <EnableCheckbox checked={p.isEnabled} onChange={p.onEnabledChange} />
      {p.statusMsg && (
        <p style={{ margin: 0, fontSize: "var(--fs-12)", fontWeight: 500, color: "var(--gr)" }}>
          {p.statusMsg}
        </p>
      )}
      <ActionButtons loading={p.loading} onPreview={p.onPreview} onSave={p.onSave} />
    </div>
  );
}

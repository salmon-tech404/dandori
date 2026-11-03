import { useState, type FormEvent } from "react";
import { Sliders, Save } from "lucide-react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { Input } from "../../ui/components/Input";
import { Checkbox } from "../../ui/components/Checkbox";
import type { AutoSwitchConfig, UpdateAutoSwitchConfigPayload } from "./types";

interface AutoSwitchConfigCardProps {
  config: AutoSwitchConfig;
  onSave: (payload: UpdateAutoSwitchConfigPayload) => Promise<void>;
}

function ConfigHeader() {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)", borderBottom: "1px solid var(--ln)", paddingBottom: "var(--sp-8)" }}>
      <Sliders style={{ width: 16, height: 16, color: "var(--bl)" }} />
      <div>
        <h4 style={{ margin: 0, fontSize: "var(--fs-13)", fontWeight: 600, color: "var(--fg)" }}>
          {t("auto_switch_title")}
        </h4>
        <p style={{ margin: 0, fontSize: "var(--fs-12)", color: "var(--mu)" }}>
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
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
      <label style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)", fontSize: "var(--fs-12)", fontWeight: 500, color: "var(--fg)" }}>
        <Checkbox
          checked={allowCross}
          onChange={(e) => setAllowCross(e.target.checked)}
        />
        <span>{t("auto_switch_allow_cross_label")}</span>
      </label>

      {allowCross && (
        <label style={{ display: "block", fontSize: "var(--fs-12)", fontWeight: 500, color: "var(--fg)" }}>
          <span>{t("auto_switch_allowed_providers_label")}</span>
          <Input
            type="text"
            value={allowedProviders}
            onChange={(e) => setAllowedProviders(e.target.value)}
            placeholder={t("auto_switch_providers_placeholder")}
            style={{ marginTop: 4 }}
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
    <div style={{ display: "flex", justifyContent: "flex-end", paddingTop: 4 }}>
      <Button
        variant="primary"
        size="sm"
        type="submit"
        disabled={isSaving}
        icon={<Save size={14} />}
      >
        {t("auto_switch_save_btn")}
      </Button>
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
    <form onSubmit={onSubmit} style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)", paddingTop: "var(--sp-8)" }}>
      <label style={{ display: "block", fontSize: "var(--fs-12)", fontWeight: 500, color: "var(--fg)" }}>
        <span>{t("auto_switch_max_switches_label")}</span>
        <Input
          type="number"
          min="1"
          max="10"
          value={maxSwitches}
          onChange={(e) => setMaxSwitches(e.target.value)}
          style={{ marginTop: 4 }}
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
    <div style={{ borderRadius: "var(--radius-10)", border: "1px solid var(--ln)", backgroundColor: "var(--sf)", padding: "var(--sp-16)" }}>
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

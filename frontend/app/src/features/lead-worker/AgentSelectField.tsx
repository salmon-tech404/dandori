import { FC } from "react";
import { t } from "../../core/i18n";
import { Select } from "../../ui/components/Select";

interface SelectProps {
  label: string;
  role: "lead" | "worker";
  currentValue: string;
  effectiveValue: string;
  isOverridden: boolean;
  readyProviders: string[];
  onChange: (val: string) => void;
}

function buildAgentOptions(effectiveValue: string, readyProviders: string[]) {
  return [
    { value: "", label: t("default_badge", { provider: effectiveValue }) },
    ...["claude", "gemini", "codex"].map((p) => ({
      value: p,
      label: `${p} ${readyProviders.includes(p) ? "" : "(chưa sẵn sàng)"}`,
    })),
  ];
}

export const AgentSelectField: FC<SelectProps> = ({
  label,
  currentValue,
  effectiveValue,
  isOverridden,
  readyProviders,
  onChange,
}) => {
  const isReady = !currentValue || readyProviders.includes(currentValue);
  const badgeText = isOverridden ? t("override_badge") : t("default_badge", { provider: effectiveValue });
  const options = buildAgentOptions(effectiveValue, readyProviders);

  return (
    <div style={{ marginBottom: "var(--sp-16)" }}>
      <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "var(--sp-4)" }}>
        <label htmlFor={`agent-select-${label}`} style={{ fontWeight: 600 }}>{label}</label>
        <span style={{ fontSize: "0.85em", color: "var(--mu)" }}>{badgeText}</span>
      </div>
      <Select
        id={`agent-select-${label}`}
        value={currentValue}
        onChange={(e: React.ChangeEvent<HTMLSelectElement>) => onChange(e.target.value)}
        options={options}
        style={{ width: "100%", borderColor: isReady ? "var(--ln)" : "var(--rd)" }}
      />
      {!isReady && (
        <div style={{ color: "var(--rd)", fontSize: "0.85em", marginTop: "var(--sp-4)" }}>
          {t("no_ready_accounts_warning")}
        </div>
      )}
    </div>
  );
};

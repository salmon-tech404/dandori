import { FC } from "react";
import { t } from "../../core/i18n";

interface SelectProps {
  label: string;
  role: "lead" | "worker";
  currentValue: string;
  effectiveValue: string;
  isOverridden: boolean;
  readyProviders: string[];
  onChange: (val: string) => void;
}

const ProviderOptions: FC<{ effectiveValue: string; readyProviders: string[] }> = ({
  effectiveValue,
  readyProviders,
}) => (
  <>
    <option value="">{t("default_badge", { provider: effectiveValue })}</option>
    {["claude", "gemini", "codex"].map((p) => (
      <option key={p} value={p}>
        {p} {readyProviders.includes(p) ? "" : "(chưa sẵn sàng)"}
      </option>
    ))}
  </>
);

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

  return (
    <div style={{ marginBottom: "var(--space-md)" }}>
      <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "var(--space-xs)" }}>
        <label htmlFor={`agent-select-${label}`} style={{ fontWeight: 600 }}>{label}</label>
        <span style={{ fontSize: "0.85em", color: "var(--text-secondary)" }}>{badgeText}</span>
      </div>
      <select
        id={`agent-select-${label}`}
        value={currentValue}
        onChange={(e) => onChange(e.target.value)}
        style={{
          width: "100%",
          padding: "var(--space-sm)",
          borderRadius: "var(--radius-sm)",
          borderColor: isReady ? "var(--border)" : "var(--danger)",
        }}
      >
        <ProviderOptions effectiveValue={effectiveValue} readyProviders={readyProviders} />
      </select>
      {!isReady && (
        <div style={{ color: "var(--danger)", fontSize: "0.85em", marginTop: "var(--space-xs)" }}>
          {t("no_ready_accounts_warning")}
        </div>
      )}
    </div>
  );
};

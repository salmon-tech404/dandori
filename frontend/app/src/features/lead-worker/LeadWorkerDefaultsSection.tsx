import { FC, useEffect, useState, ChangeEvent } from "react";
import { t } from "../../core/i18n";
import { RoleDefaultItem } from "./types";
import { useProjectDefaults } from "./useLeadWorker";
import { EmptyState } from "../../ui/components/EmptyState";
import { Button } from "../../ui/components/Button";
import { Card } from "../../ui/components/Card";
import { Select, SelectOption } from "../../ui/components/Select";
import { Chip } from "../../ui/components/Chip";

const PROVIDER_OPTIONS: SelectOption[] = [
  { value: "claude", label: "Claude" },
  { value: "gemini", label: "Gemini" },
  { value: "codex", label: "Codex" },
  { value: "fake", label: "Fake Agent" },
];

interface RoleConfig {
  role: string;
  labelKey: string;
  defaultProv: string;
}

const ROLE_CONFIGS: RoleConfig[] = [
  { role: "pm", labelKey: "role_pm_desc", defaultProv: "claude" },
  { role: "worker", labelKey: "role_worker_desc", defaultProv: "gemini" },
  { role: "test", labelKey: "role_test_desc", defaultProv: "codex" },
  { role: "doc", labelKey: "role_doc_desc", defaultProv: "claude" },
];

function updateRoleProvider(items: RoleDefaultItem[], role: string, newProvider: string): RoleDefaultItem[] {
  const existing = items.find((it) => it.role === role);
  if (!existing) {
    return [...items, { role, provider_order: [newProvider] }];
  }
  const filtered = existing.provider_order.filter((p) => p !== newProvider);
  return items.map((it) =>
    it.role === role ? { ...it, provider_order: [newProvider, ...filtered] } : it
  );
}

function RoleSelectField({
  config,
  currentProvider,
  onChange,
}: {
  config: RoleConfig;
  currentProvider: string;
  onChange: (role: string, provider: string) => void;
}) {
  const label = t(config.labelKey);
  return (
    <Select
      label={label}
      value={currentProvider}
      options={PROVIDER_OPTIONS}
      onChange={(e: ChangeEvent<HTMLSelectElement>) => onChange(config.role, e.target.value)}
    />
  );
}

function RoleGrid({
  defaults,
  onChange,
}: {
  defaults: RoleDefaultItem[];
  onChange: (role: string, provider: string) => void;
}) {
  return (
    <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(220px, 1fr))", gap: "var(--sp-16)" }}>
      {ROLE_CONFIGS.map((cfg) => {
        const item = defaults.find((d) => d.role === cfg.role);
        const currentProvider = item?.provider_order[0] || cfg.defaultProv;
        return (
          <RoleSelectField
            key={cfg.role}
            config={cfg}
            currentProvider={currentProvider}
            onChange={onChange}
          />
        );
      })}
    </div>
  );
}

function LeadWorkerDefaultsForm({ projectId }: { projectId: string }) {
  const { fetchDefaults, saveDefaults, loading, error } = useProjectDefaults(projectId);
  const [defaults, setDefaults] = useState<RoleDefaultItem[]>([]);
  const [savedMsg, setSavedMsg] = useState<string | null>(null);

  useEffect(() => {
    fetchDefaults().then(setDefaults);
  }, [fetchDefaults]);

  const handleProviderChange = (role: string, provider: string) => {
    setDefaults((prev) => updateRoleProvider(prev, role, provider));
  };

  const handleSave = async () => {
    if (await saveDefaults(defaults)) {
      setSavedMsg(t("save_defaults_success"));
      setTimeout(() => setSavedMsg(null), 3000);
    }
  };

  return (
    <Card
      title={t("lead_worker_title")}
      actions={<Chip variant="gray">{t("role_defaults_chip")}</Chip>}
      style={{ marginTop: "var(--sp-16)" }}
    >
      {error && <div style={{ color: "var(--rd)", fontSize: "var(--fs-13)" }}>{error}</div>}
      {savedMsg && <div style={{ color: "var(--gr)", fontSize: "var(--fs-13)" }}>{savedMsg}</div>}
      <RoleGrid defaults={defaults} onChange={handleProviderChange} />
      <div style={{ marginTop: "var(--sp-8)" }}>
        <Button variant="primary" type="button" disabled={loading} loading={loading} onClick={handleSave}>
          {t("btn_save_defaults")}
        </Button>
      </div>
    </Card>
  );
}

export const LeadWorkerDefaultsSection: FC<{ context?: Record<string, unknown> }> = ({ context }) => {
  const projectId = typeof context?.projectId === "string" ? context.projectId : "";
  if (!projectId) return <EmptyState title={t("need_project_title")} description={t("need_project_desc")} />;
  return <LeadWorkerDefaultsForm projectId={projectId} />;
};

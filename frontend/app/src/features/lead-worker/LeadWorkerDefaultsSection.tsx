import { FC, useEffect, useState } from "react";
import { t } from "../../core/i18n";
import { RoleDefaultItem } from "./types";
import { useProjectDefaults } from "./useLeadWorker";

interface BadgeProps {
  prov: string;
  idx: number;
  total: number;
  onMove: (index: number, direction: -1 | 1) => void;
}

const ProviderBadge: FC<BadgeProps> = ({ prov, idx, total, onMove }) => (
  <div style={{ display: "flex", alignItems: "center", gap: "var(--space-xs)", background: "var(--surface)", padding: "var(--space-xs) var(--space-sm)", borderRadius: "var(--radius-sm)", border: "1px solid var(--border)" }}>
    <span>{idx + 1}. {prov}</span>
    <button type="button" aria-label={`Move ${prov} up`} disabled={idx === 0} onClick={() => onMove(idx, -1)} style={{ cursor: idx === 0 ? "not-allowed" : "pointer" }}>↑</button>
    <button type="button" aria-label={`Move ${prov} down`} disabled={idx === total - 1} onClick={() => onMove(idx, 1)} style={{ cursor: idx === total - 1 ? "not-allowed" : "pointer" }}>↓</button>
  </div>
);

const RoleDefaultRow: FC<{ item: RoleDefaultItem; onMove: (role: string, index: number, dir: -1 | 1) => void }> = ({ item, onMove }) => (
  <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "var(--space-md)", border: "1px solid var(--border)", borderRadius: "var(--radius-md)", marginBottom: "var(--space-sm)" }}>
    <div style={{ fontWeight: 600, width: "200px" }}>{t(`role_${item.role}`)}</div>
    <div style={{ display: "flex", gap: "var(--space-sm)", alignItems: "center" }}>
      {item.provider_order.map((prov, idx) => (
        <ProviderBadge key={prov} prov={prov} idx={idx} total={item.provider_order.length} onMove={(i, d) => onMove(item.role, i, d)} />
      ))}
    </div>
  </div>
);

function moveInList(items: RoleDefaultItem[], role: string, index: number, dir: -1 | 1): RoleDefaultItem[] {
  return items.map((item) => {
    if (item.role !== role) return item;
    const newOrder = [...item.provider_order];
    const targetIdx = index + dir;
    if (targetIdx < 0 || targetIdx >= newOrder.length) return item;
    const temp = newOrder[index];
    newOrder[index] = newOrder[targetIdx];
    newOrder[targetIdx] = temp;
    return { ...item, provider_order: newOrder };
  });
}

export const LeadWorkerDefaultsSection: FC<{ context?: Record<string, unknown> }> = ({ context }) => {
  const projectId = (context?.projectId as string) || "default";
  const { fetchDefaults, saveDefaults, loading, error } = useProjectDefaults(projectId);
  const [defaults, setDefaults] = useState<RoleDefaultItem[]>([]);
  const [savedMsg, setSavedMsg] = useState<string | null>(null);

  useEffect(() => { fetchDefaults().then(setDefaults); }, [fetchDefaults]);

  const handleSave = async () => {
    if (await saveDefaults(defaults)) {
      setSavedMsg(t("save_defaults_success"));
      setTimeout(() => setSavedMsg(null), 3000);
    }
  };

  return (
    <div style={{ marginTop: "var(--space-lg)" }}>
      <h3>{t("lead_worker_title")}</h3>
      <p style={{ color: "var(--text-secondary)", marginBottom: "var(--space-md)" }}>{t("lead_worker_desc")}</p>
      {error && <div style={{ color: "var(--danger)", marginBottom: "var(--space-sm)" }}>{error}</div>}
      {savedMsg && <div style={{ color: "var(--success)", marginBottom: "var(--space-sm)" }}>{savedMsg}</div>}
      {defaults.map((item) => (
        <RoleDefaultRow key={item.role} item={item} onMove={(r, i, d) => setDefaults((p) => moveInList(p, r, i, d))} />
      ))}
      <button type="button" disabled={loading} onClick={handleSave} style={{ marginTop: "var(--space-md)", padding: "var(--space-sm) var(--space-lg)" }}>
        {t("btn_save_defaults")}
      </button>
    </div>
  );
};

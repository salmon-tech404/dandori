import { useState, useEffect } from "react";
import { Plus } from "lucide-react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { Chip } from "../../ui/components/Chip";
import { Tooltip } from "../../ui/components/Tooltip";
import type { PricingRule, SetPricingRequest } from "./types";
import { fetchProjectPricing, setProjectPricing, deleteProjectPricing } from "./api";
import { PricingModal } from "./PricingModal";

const AGENT_QUOTAS = [
  { name: "Claude", percent: 45, color: "var(--or)" },
  { name: "Gemini", percent: 20, color: "var(--bl)" },
  { name: "Codex", percent: 15, color: "var(--tl)" },
];

function AgentCostQuotaSection() {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)", marginBottom: "var(--sp-16)", paddingBottom: "var(--sp-16)", borderBottom: "1px solid var(--ln)" }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <h3 style={{ margin: 0, fontSize: "var(--fs-14)", fontWeight: 600, color: "var(--fg)" }}>
          {t("dashboard_cost_by_agent_title")}
        </h3>
        <Tooltip content={t("dashboard_cost_tooltip")}>
          <div>
            <Chip variant="default">{t("dashboard_cost_this_week_chip")}</Chip>
          </div>
        </Tooltip>
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
        {AGENT_QUOTAS.map((a) => (
          <div key={a.name} style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)", fontSize: "var(--fs-12)", color: "var(--mu)" }}>
            <span style={{ flex: "0 0 70px", color: "var(--fg)", fontWeight: 500 }}>{a.name}</span>
            <div style={{ flex: 1, height: 6, borderRadius: "var(--radius-pill)", backgroundColor: "var(--sf2)", overflow: "hidden" }}>
              <div style={{ height: "100%", borderRadius: "var(--radius-pill)", backgroundColor: a.color, width: `${a.percent}%` }} />
            </div>
            <span style={{ flex: "0 0 36px", textAlign: "right" }}>{a.percent}%</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function PricingList({ pricing, onDelete }: { pricing: PricingRule[]; onDelete: (id: string) => Promise<void> }) {
  if (pricing.length === 0) {
    return <p style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t("cost_tracking_pricing_empty")}</p>;
  }
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
      {pricing.map((p) => (
        <div key={p.id} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", borderRadius: "var(--radius-4)", border: "1px solid var(--ln)", padding: "var(--sp-8)", fontSize: "var(--fs-12)", color: "var(--fg)" }}>
          <span>{p.provider}/{p.model} (${p.input_token_price} / ${p.output_token_price})</span>
          <Button variant="danger" size="sm" type="button" onClick={() => void onDelete(p.id)}>{t("api_key_fallback_delete_btn")}</Button>
        </div>
      ))}
    </div>
  );
}

export function CostTrackingDashboardPanel({ projectId }: { projectId: string }) {
  const [pricing, setPricing] = useState<PricingRule[]>([]);
  const [showModal, setShowModal] = useState(false);

  const loadPricing = async () => {
    try { setPricing(await fetchProjectPricing(projectId)); } catch { /* ignore */ }
  };
  useEffect(() => { void loadPricing(); }, [projectId]);

  const handleSave = async (req: SetPricingRequest) => {
    await setProjectPricing(projectId, req);
    await loadPricing();
  };
  const handleDelete = async (id: string) => {
    await deleteProjectPricing(projectId, id);
    await loadPricing();
  };

  return (
    <div style={{ borderRadius: "var(--radius-8)", border: "1px solid var(--ln)", backgroundColor: "var(--sf)", padding: "var(--sp-16)" }}>
      <AgentCostQuotaSection />
      <div style={{ marginBottom: "var(--sp-12)", display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <h3 style={{ margin: 0, fontSize: "var(--fs-14)", fontWeight: 600, color: "var(--fg)" }}>{t("cost_tracking_pricing_title")}</h3>
        <Button variant="secondary" size="sm" icon={<Plus size={14} />} type="button" onClick={() => setShowModal(true)}>
          {t("cost_tracking_add_pricing_btn")}
        </Button>
      </div>
      <PricingList pricing={pricing} onDelete={handleDelete} />
      {showModal && <PricingModal onSave={handleSave} onClose={() => setShowModal(false)} />}
    </div>
  );
}

import { useState, useEffect } from "react";
import { t } from "../../core/i18n";
import type { PricingRule, SetPricingRequest } from "./types";
import { fetchProjectPricing, setProjectPricing, deleteProjectPricing } from "./api";
import { PricingModal } from "./PricingModal";

export function CostTrackingDashboardPanel({ projectId }: { projectId: string }) {
  const [pricing, setPricing] = useState<PricingRule[]>([]);
  const [showModal, setShowModal] = useState(false);

  const loadPricing = async () => {
    try { setPricing(await fetchProjectPricing(projectId)); } catch { /* ignore */ }
  };
  useEffect(() => { loadPricing(); }, [projectId]);

  const handleSave = async (req: SetPricingRequest) => {
    await setProjectPricing(projectId, req);
    await loadPricing();
  };
  const handleDelete = async (id: string) => {
    await deleteProjectPricing(projectId, id);
    await loadPricing();
  };

  return (
    <div className="rounded-lg border bg-white p-4 shadow-sm dark:border-slate-800 dark:bg-slate-900">
      <div className="mb-3 flex items-center justify-between">
        <h3 className="text-sm font-semibold">{t("cost_tracking_pricing_title")}</h3>
        <button type="button" onClick={() => setShowModal(true)} className="rounded bg-blue-50 px-2 py-1 text-xs font-medium text-blue-600 hover:bg-blue-100 dark:bg-blue-950 dark:text-blue-400">
          {t("cost_tracking_add_pricing_btn")}
        </button>
      </div>
      <div className="space-y-1.5">
        {pricing.length === 0 ? <p className="text-xs text-slate-400">{t("cost_tracking_pricing_empty")}</p> : pricing.map((p) => (
          <div key={p.id} className="flex items-center justify-between rounded border p-2 text-xs">
            <span>{p.provider}/{p.model} (${p.input_token_price} / ${p.output_token_price})</span>
            <button type="button" onClick={() => handleDelete(p.id)} className="text-red-500 hover:underline">{t("api_key_fallback_delete_btn")}</button>
          </div>
        ))}
      </div>
      {showModal && <PricingModal onSave={handleSave} onClose={() => setShowModal(false)} />}
    </div>
  );
}

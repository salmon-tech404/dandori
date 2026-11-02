import { useState } from "react";
import { t } from "../../core/i18n";
import type { SetPricingRequest } from "./types";

interface PricingModalProps {
  onSave: (req: SetPricingRequest) => Promise<void>;
  onClose: () => void;
}

export function PricingModal({ onSave, onClose }: PricingModalProps) {
  const [provider, setProvider] = useState("anthropic");
  const [model, setModel] = useState("claude-3-5-sonnet");
  const [inPrice, setInPrice] = useState("3.00");
  const [outPrice, setOutPrice] = useState("15.00");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    await onSave({
      provider,
      model,
      input_token_price: parseFloat(inPrice) || 0,
      output_token_price: parseFloat(outPrice) || 0,
    });
    setLoading(false);
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <form onSubmit={handleSubmit} className="w-full max-w-sm rounded-lg bg-white p-5 shadow dark:bg-slate-800">
        <h3 className="mb-3 text-base font-semibold">{t("cost_tracking_pricing_title")}</h3>
        <input value={provider} onChange={(e) => setProvider(e.target.value)} placeholder="Provider" className="mb-2 w-full rounded border p-1.5 text-xs dark:bg-slate-900" required />
        <input value={model} onChange={(e) => setModel(e.target.value)} placeholder="Model" className="mb-2 w-full rounded border p-1.5 text-xs dark:bg-slate-900" required />
        <input type="number" step="0.01" value={inPrice} onChange={(e) => setInPrice(e.target.value)} placeholder="Input $/1M" className="mb-2 w-full rounded border p-1.5 text-xs dark:bg-slate-900" required />
        <input type="number" step="0.01" value={outPrice} onChange={(e) => setOutPrice(e.target.value)} placeholder="Output $/1M" className="mb-4 w-full rounded border p-1.5 text-xs dark:bg-slate-900" required />
        <div className="flex justify-end gap-2">
          <button type="button" onClick={onClose} className="rounded px-3 py-1 text-xs">{t("cost_tracking_cancel")}</button>
          <button type="submit" disabled={loading} className="rounded bg-blue-600 px-3 py-1 text-xs text-white">{t("cost_tracking_save")}</button>
        </div>
      </form>
    </div>
  );
}

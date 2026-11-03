import { useState } from "react";
import { t } from "../../core/i18n";
import { Modal } from "../../ui/components/Modal";
import { Button } from "../../ui/components/Button";
import { Input } from "../../ui/components/Input";
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
    <Modal
      open={true}
      onClose={onClose}
      title={t("cost_tracking_pricing_title")}
      size="sm"
      footer={
        <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--sp-8)" }}>
          <Button variant="ghost" onClick={onClose}>{t("cost_tracking_cancel")}</Button>
          <Button variant="primary" loading={loading} onClick={handleSubmit}>{t("cost_tracking_save")}</Button>
        </div>
      }
    >
      <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)" }}>
        <Input value={provider} onChange={(e) => setProvider(e.target.value)} placeholder={t("cost_provider_placeholder")} required />
        <Input value={model} onChange={(e) => setModel(e.target.value)} placeholder={t("cost_model_placeholder")} required />
        <Input type="number" step="0.01" value={inPrice} onChange={(e) => setInPrice(e.target.value)} placeholder={t("cost_input_price_placeholder")} required />
        <Input type="number" step="0.01" value={outPrice} onChange={(e) => setOutPrice(e.target.value)} placeholder={t("cost_output_price_placeholder")} required />
      </form>
    </Modal>
  );
}

import { useState } from "react";
import { t } from "../../core/i18n";
import { Modal } from "../../ui/components/Modal";
import { Button } from "../../ui/components/Button";
import { Input } from "../../ui/components/Input";

interface TaskBudgetModalProps {
  currentBudget: number | null;
  onSave: (val: number) => Promise<void>;
  onClose: () => void;
}

export function TaskBudgetModal({ currentBudget, onSave, onClose }: TaskBudgetModalProps) {
  const [budgetVal, setBudgetVal] = useState(currentBudget ? String(currentBudget) : "1.00");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const num = parseFloat(budgetVal);
    if (isNaN(num) || num <= 0) return;
    setLoading(true);
    await onSave(num);
    setLoading(false);
    onClose();
  };

  return (
    <Modal
      open={true}
      onClose={onClose}
      title={t("cost_tracking_budget_modal_title")}
      size="sm"
      footer={
        <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--sp-8)" }}>
          <Button variant="ghost" onClick={onClose}>{t("cost_tracking_cancel")}</Button>
          <Button variant="primary" loading={loading} onClick={handleSubmit}>{t("cost_tracking_save")}</Button>
        </div>
      }
    >
      <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)" }}>
        <Input
          label={t("cost_tracking_budget_input_label")}
          type="number"
          step="0.01"
          min="0.01"
          required
          value={budgetVal}
          onChange={(e) => setBudgetVal(e.target.value)}
        />
      </form>
    </Modal>
  );
}

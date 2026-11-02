import { useState } from "react";
import { t } from "../../core/i18n";

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
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <form onSubmit={handleSubmit} className="w-full max-w-sm rounded-lg bg-white p-5 shadow dark:bg-slate-800">
        <h3 className="mb-3 text-base font-semibold">{t("cost_tracking_budget_modal_title")}</h3>
        <label className="mb-1 block text-xs font-medium text-slate-600 dark:text-slate-300">
          {t("cost_tracking_budget_input_label")}
        </label>
        <input
          type="number"
          step="0.01"
          min="0.01"
          required
          value={budgetVal}
          onChange={(e) => setBudgetVal(e.target.value)}
          className="mb-4 w-full rounded border px-3 py-1.5 text-sm dark:bg-slate-900"
        />
        <div className="flex justify-end gap-2">
          <button type="button" onClick={onClose} className="rounded px-3 py-1 text-sm text-slate-600 hover:bg-slate-100 dark:text-slate-300">
            {t("cost_tracking_cancel")}
          </button>
          <button type="submit" disabled={loading} className="rounded bg-blue-600 px-3 py-1 text-sm text-white hover:bg-blue-700 disabled:opacity-50">
            {t("cost_tracking_save")}
          </button>
        </div>
      </form>
    </div>
  );
}

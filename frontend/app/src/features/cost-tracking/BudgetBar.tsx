import { t } from "../../core/i18n";
import type { TaskCostSummary } from "./types";

interface BudgetBarProps {
  summary: TaskCostSummary;
  onOpenBudgetModal: () => void;
}

export function BudgetBar({ summary, onOpenBudgetModal }: BudgetBarProps) {
  const percent = summary.budget && summary.total_cost !== null
    ? Math.min(Math.round((summary.total_cost / summary.budget) * 100), 100)
    : 0;

  return (
    <div className="rounded-lg border p-3 dark:border-slate-800">
      <div className="mb-2 flex items-center justify-between">
        <span className="text-xs font-medium">{t("cost_tracking_budget_label")}</span>
        <div className="flex items-center gap-2">
          {summary.is_over_budget && (
            <span className="rounded bg-rose-100 px-1.5 py-0.5 text-[10px] font-bold text-rose-700 dark:bg-rose-950 dark:text-rose-300">
              {t("cost_tracking_over_budget")}
            </span>
          )}
          <button type="button" onClick={onOpenBudgetModal} className="text-xs text-blue-600 hover:underline">
            {summary.budget ? `$${summary.budget.toFixed(2)}` : t("cost_tracking_budget_set_btn")}
          </button>
        </div>
      </div>
      {summary.budget && summary.total_cost !== null && (
        <div className="h-2 w-full overflow-hidden rounded-full bg-slate-200 dark:bg-slate-700">
          <div
            className={`h-full ${summary.is_over_budget ? "bg-rose-500" : "bg-emerald-500"}`}
            style={{ width: `${percent}%` }}
          />
        </div>
      )}
    </div>
  );
}

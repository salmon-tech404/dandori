import { t } from "../../core/i18n";
import type { TaskCostSummary } from "./types";

interface CostOverviewCardProps {
  summary: TaskCostSummary;
}

export function CostOverviewCard({ summary }: CostOverviewCardProps) {
  const costDisplay = summary.total_cost !== null
    ? `$${summary.total_cost.toFixed(4)}`
    : t("cost_tracking_no_pricing");

  return (
    <div className="grid grid-cols-2 gap-3 rounded-lg border bg-slate-50 p-4 dark:border-slate-800 dark:bg-slate-900/50">
      <div>
        <span className="text-xs text-slate-500">{t("cost_tracking_total_tokens")}</span>
        <p className="text-lg font-bold text-slate-900 dark:text-white">
          {summary.has_token_data ? summary.total_tokens.toLocaleString() : t("cost_tracking_no_data")}
        </p>
        {summary.has_token_data && (
          <p className="text-[11px] text-slate-400">
            {t("cost_tracking_input_tokens")} {summary.total_input_tokens.toLocaleString()} | {t("cost_tracking_output_tokens")} {summary.total_output_tokens.toLocaleString()}
          </p>
        )}
      </div>
      <div>
        <span className="text-xs text-slate-500">{t("cost_tracking_total_cost")}</span>
        <p className="text-lg font-bold text-slate-900 dark:text-white">{costDisplay}</p>
        {summary.missing_runs_count > 0 && (
          <p className="text-[11px] text-amber-600 dark:text-amber-400">
            {t("cost_tracking_missing_runs_notice", { count: summary.missing_runs_count })}
          </p>
        )}
      </div>
    </div>
  );
}

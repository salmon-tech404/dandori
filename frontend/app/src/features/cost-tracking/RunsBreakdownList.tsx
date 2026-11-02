import { t } from "../../core/i18n";
import type { RunCostView } from "./types";

interface RunsBreakdownListProps {
  runs: RunCostView[];
}

export function RunsBreakdownList({ runs }: RunsBreakdownListProps) {
  return (
    <div className="space-y-2">
      <h4 className="text-xs font-semibold text-slate-700 dark:text-slate-300">{t("cost_tracking_runs_breakdown")}</h4>
      <div className="space-y-1.5 max-h-48 overflow-y-auto">
        {runs.map((r) => (
          <div key={r.run_id} className="flex items-center justify-between rounded border p-2 text-xs dark:border-slate-800">
            <div>
              <p className="font-medium text-slate-800 dark:text-slate-200">{r.run_id}</p>
              <p className="text-[10px] text-slate-400">
                {t("cost_tracking_agent")} {r.agent_id} | {t("cost_tracking_model")} {r.provider}/{r.model}
              </p>
            </div>
            <div className="text-right">
              <p className="font-semibold">
                {r.has_token_data ? `${r.total_tokens?.toLocaleString()} tok` : t("cost_tracking_no_data")}
              </p>
              <p className="text-[10px] text-slate-500">
                {r.calculated_cost !== null && r.calculated_cost !== undefined
                  ? `$${r.calculated_cost.toFixed(4)}`
                  : t("cost_tracking_no_pricing")}
              </p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

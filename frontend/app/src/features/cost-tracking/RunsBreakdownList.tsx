import { t } from "../../core/i18n";
import type { RunCostView } from "./types";

interface RunsBreakdownListProps {
  runs: RunCostView[];
}

export function RunsBreakdownList({ runs }: RunsBreakdownListProps) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
      <h4 style={{ margin: 0, fontSize: "var(--fs-12)", fontWeight: 600, color: "var(--fg)" }}>{t("cost_tracking_runs_breakdown")}</h4>
      <div style={{ display: "flex", flexDirection: "column", gap: 6, maxHeight: 192, overflowY: "auto" }}>
        {runs.map((r) => (
          <div key={r.run_id} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", borderRadius: "var(--radius-4)", border: "1px solid var(--ln)", backgroundColor: "var(--sf)", padding: "var(--sp-8)", fontSize: "var(--fs-12)" }}>
            <div>
              <p style={{ margin: 0, fontWeight: 500, color: "var(--fg)" }}>{r.run_id}</p>
              <p style={{ margin: 0, fontSize: "10px", color: "var(--mu)" }}>
                {t("cost_tracking_agent")} {r.agent_id} | {t("cost_tracking_model")} {r.provider}/{r.model}
              </p>
            </div>
            <div style={{ textAlign: "right" }}>
              <p style={{ margin: 0, fontWeight: 600, color: "var(--fg)" }}>
                {r.has_token_data ? `${r.total_tokens?.toLocaleString()} tok` : t("cost_tracking_no_data")}
              </p>
              <p style={{ margin: 0, fontSize: "10px", color: "var(--mu)" }}>
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

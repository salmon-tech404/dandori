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
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--sp-12)", borderRadius: "var(--radius-8)", border: "1px solid var(--ln)", backgroundColor: "var(--sf)", padding: "var(--sp-16)" }}>
      <div>
        <span style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t("cost_tracking_total_tokens")}</span>
        <p style={{ margin: "4px 0 0", fontSize: "var(--fs-18)", fontWeight: 700, color: "var(--fg)" }}>
          {summary.has_token_data ? summary.total_tokens.toLocaleString() : t("cost_tracking_no_data")}
        </p>
        {summary.has_token_data && (
          <p style={{ margin: "4px 0 0", fontSize: "11px", color: "var(--mu)" }}>
            {t("cost_tracking_input_tokens")} {summary.total_input_tokens.toLocaleString()} | {t("cost_tracking_output_tokens")} {summary.total_output_tokens.toLocaleString()}
          </p>
        )}
      </div>
      <div>
        <span style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t("cost_tracking_total_cost")}</span>
        <p style={{ margin: "4px 0 0", fontSize: "var(--fs-18)", fontWeight: 700, color: "var(--fg)" }}>{costDisplay}</p>
        {summary.missing_runs_count > 0 && (
          <p style={{ margin: "4px 0 0", fontSize: "11px", color: "var(--or)" }}>
            {t("cost_tracking_missing_runs_notice", { count: summary.missing_runs_count })}
          </p>
        )}
      </div>
    </div>
  );
}

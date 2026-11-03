import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
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
    <div style={{ borderRadius: "var(--radius-8)", border: "1px solid var(--ln)", padding: "var(--sp-12)", backgroundColor: "var(--sf)" }}>
      <div style={{ marginBottom: "var(--sp-8)", display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <span style={{ fontSize: "var(--fs-12)", fontWeight: 500, color: "var(--fg)" }}>{t("cost_tracking_budget_label")}</span>
        <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
          {summary.is_over_budget && (
            <span style={{ borderRadius: "var(--radius-4)", backgroundColor: "var(--rdb)", padding: "2px 6px", fontSize: "10px", fontWeight: 700, color: "var(--rd)" }}>
              {t("cost_tracking_over_budget")}
            </span>
          )}
          <Button variant="ghost" size="sm" type="button" onClick={onOpenBudgetModal} style={{ fontSize: "var(--fs-12)", color: "var(--bl)" }}>
            {summary.budget ? `$${summary.budget.toFixed(2)}` : t("cost_tracking_budget_set_btn")}
          </Button>
        </div>
      </div>
      {summary.budget && summary.total_cost !== null && (
        <div style={{ height: 8, width: "100%", overflow: "hidden", borderRadius: "var(--radius-pill)", backgroundColor: "var(--sf2)" }}>
          <div
            style={{
              height: "100%",
              width: `${percent}%`,
              backgroundColor: summary.is_over_budget ? "var(--rd)" : "var(--gr)",
            }}
          />
        </div>
      )}
    </div>
  );
}

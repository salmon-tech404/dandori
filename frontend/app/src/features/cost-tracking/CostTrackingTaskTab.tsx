import { useState, useEffect } from "react";
import { t } from "../../core/i18n";
import type { TaskCostSummary } from "./types";
import { fetchTaskCost, setTaskBudget } from "./api";
import { CostOverviewCard } from "./CostOverviewCard";
import { BudgetBar } from "./BudgetBar";
import { RunsBreakdownList } from "./RunsBreakdownList";
import { TaskBudgetModal } from "./TaskBudgetModal";

export function CostTrackingTaskTab({ taskId }: { taskId: string }) {
  const [summary, setSummary] = useState<TaskCostSummary | null>(null);
  const [showModal, setShowModal] = useState(false);

  const loadData = async () => {
    try {
      const data = await fetchTaskCost(taskId);
      setSummary(data);
    } catch {
      // Bỏ qua lỗi kết nối
    }
  };

  useEffect(() => { void loadData(); }, [taskId]);

  const handleSaveBudget = async (val: number) => {
    await setTaskBudget(taskId, val);
    await loadData();
  };

  if (!summary) return <div style={{ padding: "var(--sp-16)", fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t("api_key_fallback_loading")}</div>;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-16)", padding: "var(--sp-16)" }}>
      <CostOverviewCard summary={summary} />
      <BudgetBar summary={summary} onOpenBudgetModal={() => setShowModal(true)} />
      <RunsBreakdownList runs={summary.runs} />
      <p style={{ margin: 0, fontSize: "10px", color: "var(--mu)", fontStyle: "italic" }}>{t("cost_tracking_source_disclaimer")}</p>
      {showModal && (
        <TaskBudgetModal
          currentBudget={summary.budget}
          onSave={handleSaveBudget}
          onClose={() => setShowModal(false)}
        />
      )}
    </div>
  );
}

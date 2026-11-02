import { CostTrackingTaskTab } from "./CostTrackingTaskTab";
import { CostTrackingDashboardPanel } from "./CostTrackingDashboardPanel";

export * from "./types";
export * from "./api";
export * from "./CostTrackingTaskTab";
export * from "./CostTrackingDashboardPanel";

export const contributions = {
  "cost-tracking-task-tab": CostTrackingTaskTab,
  "cost-tracking-dashboard-panel": CostTrackingDashboardPanel,
};

export default contributions;

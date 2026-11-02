import { ApprovalGateChip } from "./components/ApprovalGateChip";
import { ApprovalGatePanel } from "./components/ApprovalGatePanel";

export const contributions = {
  "approval-gate-chip": ApprovalGateChip,
  "approval-gate-tab": ApprovalGatePanel,
  "approval-gate-kpi": ApprovalGateChip,
};

export * from "./types";
export * from "./api/approvalGateApi";
export { ApprovalGateChip, ApprovalGatePanel };
export { ApprovalQueueList } from "./components/ApprovalQueueList";
export { ApprovalDecisionModal } from "./components/ApprovalDecisionModal";

import { useEffect, useState } from "react";
import { t } from "../../../core/i18n";
import { Button } from "../../../ui/components/Button";
import { approvalGateApi } from "../api/approvalGateApi";
import type { ApprovalRequest } from "../types";
import { ApprovalDecisionModal } from "./ApprovalDecisionModal";
import { ApprovalQueueList } from "./ApprovalQueueList";

interface ApprovalGatePanelProps {
  taskId?: string;
  workspaceId?: string;
  onViewDiff?: (taskId: string) => void;
}

function useApprovalRequests(taskId?: string, workspaceId?: string) {
  const [requests, setRequests] = useState<ApprovalRequest[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadRequests = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await approvalGateApi.listRequests({
        task_id: taskId,
        workspace_id: workspaceId,
      });
      setRequests(data);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void loadRequests();
  }, [taskId, workspaceId]);

  return { requests, loading, error, loadRequests };
}

interface PanelHeaderProps {
  onRefresh: () => void;
}

function PanelHeader({ onRefresh }: PanelHeaderProps) {
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "var(--sp-16)" }}>
      <h4 style={{ margin: 0, fontSize: "var(--fs-14)", fontWeight: 600, color: "var(--fg)" }}>{t("approval_gate_title")}</h4>
      <Button
        variant="ghost"
        size="sm"
        type="button"
        onClick={onRefresh}
        style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}
      >
        {t("approval_gate_btn_refresh")}
      </Button>
    </div>
  );
}

export function ApprovalGatePanel({ taskId, workspaceId, onViewDiff }: ApprovalGatePanelProps) {
  const { requests, loading, error, loadRequests } = useApprovalRequests(taskId, workspaceId);
  const [selectedRequest, setSelectedRequest] = useState<ApprovalRequest | null>(null);

  const handleApprove = async (id: string, reason?: string) => {
    await approvalGateApi.approveRequest(id, reason);
    await loadRequests();
  };

  const handleReject = async (id: string, reason?: string) => {
    await approvalGateApi.rejectRequest(id, reason);
    await loadRequests();
  };

  if (loading) {
    return <div style={{ padding: "var(--sp-16)", fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t("approval_gate_loading")}</div>;
  }
  if (error) {
    return <div style={{ padding: "var(--sp-16)", fontSize: "var(--fs-12)", color: "var(--rd)" }}>{error}</div>;
  }

  return (
    <div style={{ padding: "var(--sp-16)" }} data-testid="approval-gate-panel">
      <PanelHeader onRefresh={() => void loadRequests()} />

      <ApprovalQueueList requests={requests} onSelectRequest={setSelectedRequest} />

      {selectedRequest && (
        <ApprovalDecisionModal
          request={selectedRequest}
          onApprove={handleApprove}
          onReject={handleReject}
          onClose={() => setSelectedRequest(null)}
          onViewDiff={onViewDiff}
        />
      )}
    </div>
  );
}

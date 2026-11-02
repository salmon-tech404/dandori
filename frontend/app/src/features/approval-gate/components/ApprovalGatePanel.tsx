import { useEffect, useState } from "react";
import { t } from "../../../core/i18n";
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
    <div className="flex items-center justify-between mb-4">
      <h4 className="text-sm font-semibold text-zinc-200">{t("approval_gate_title")}</h4>
      <button
        type="button"
        onClick={onRefresh}
        className="text-xs text-zinc-400 hover:text-zinc-200"
      >
        {t("approval_gate_btn_refresh")}
      </button>
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
    return <div className="p-4 text-xs text-zinc-500">{t("approval_gate_loading")}</div>;
  }
  if (error) {
    return <div className="p-4 text-xs text-red-400">{error}</div>;
  }

  return (
    <div className="p-4" data-testid="approval-gate-panel">
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

import { t } from "../../../core/i18n";
import type { ApprovalRequest } from "../types";

interface ApprovalQueueListProps {
  requests: ApprovalRequest[];
  onSelectRequest: (request: ApprovalRequest) => void;
}

function getStatusBadgeClass(status: string) {
  if (status === "pending") return "bg-amber-500/10 text-amber-400 border border-amber-500/20";
  if (status === "approved") return "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20";
  if (status === "rejected") return "bg-red-500/10 text-red-400 border border-red-500/20";
  return "bg-zinc-800 text-zinc-400 border border-zinc-700";
}

interface ItemProps {
  req: ApprovalRequest;
  onSelect: (req: ApprovalRequest) => void;
}

function ApprovalQueueItem({ req, onSelect }: ItemProps) {
  const badgeCls = getStatusBadgeClass(req.status);
  return (
    <div className="py-3 flex items-center justify-between gap-4">
      <div>
        <div className="flex items-center gap-2">
          <span className={`text-xs px-2 py-0.5 rounded font-medium ${badgeCls}`}>
            {t(`approval_gate_status_${req.status}` as Parameters<typeof t>[0])}
          </span>
          <span className="text-sm font-medium text-zinc-200">
            {req.from_status} → {req.target_status}
          </span>
        </div>
        {req.reason && <p className="mt-1 text-xs text-zinc-400 line-clamp-1">{req.reason}</p>}
        <div className="mt-1 text-xs text-zinc-500">
          {t("approval_gate_expires_at")}: {new Date(req.expires_at).toLocaleString()}
        </div>
      </div>
      {req.status === "pending" && (
        <button
          type="button"
          onClick={() => onSelect(req)}
          className="text-xs font-medium px-3 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-200 transition-colors"
          data-testid={`review-request-${req.id}`}
        >
          {t("approval_gate_btn_review")}
        </button>
      )}
    </div>
  );
}

export function ApprovalQueueList({ requests, onSelectRequest }: ApprovalQueueListProps) {
  if (requests.length === 0) {
    return (
      <div className="py-8 text-center text-sm text-zinc-500">
        {t("approval_gate_queue_empty")}
      </div>
    );
  }

  return (
    <div className="divide-y divide-zinc-800" data-testid="approval-queue-list">
      {requests.map((req) => (
        <ApprovalQueueItem key={req.id} req={req} onSelect={onSelectRequest} />
      ))}
    </div>
  );
}

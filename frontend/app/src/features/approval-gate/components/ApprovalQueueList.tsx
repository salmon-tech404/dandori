import { CSSProperties } from "react";
import { t } from "../../../core/i18n";
import { Button } from "../../../ui/components/Button";
import type { ApprovalRequest } from "../types";

interface ApprovalQueueListProps {
  requests: ApprovalRequest[];
  onSelectRequest: (request: ApprovalRequest) => void;
}

function getStatusBadgeStyle(status: string): CSSProperties {
  if (status === "pending") {
    return {
      backgroundColor: "var(--orb)",
      color: "var(--or)",
      border: "1px solid var(--or)",
    };
  }
  if (status === "approved") {
    return {
      backgroundColor: "var(--grb)",
      color: "var(--gr)",
      border: "1px solid var(--gr)",
    };
  }
  if (status === "rejected") {
    return {
      backgroundColor: "var(--rdb)",
      color: "var(--rd)",
      border: "1px solid var(--rd)",
    };
  }
  return {
    backgroundColor: "var(--sf2)",
    color: "var(--mu)",
    border: "1px solid var(--ln)",
  };
}

interface ItemProps {
  req: ApprovalRequest;
  onSelect: (req: ApprovalRequest) => void;
}

function ApprovalQueueItem({ req, onSelect }: ItemProps) {
  const badgeStyle = getStatusBadgeStyle(req.status);
  return (
    <div style={{ padding: "var(--sp-12) 0", display: "flex", alignItems: "center", justifyContent: "space-between", gap: "var(--sp-16)" }}>
      <div>
        <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
          <span
            style={{
              fontSize: "var(--fs-12)",
              padding: "2px 8px",
              borderRadius: "var(--radius-4)",
              fontWeight: 500,
              ...badgeStyle,
            }}
          >
            {t(`approval_gate_status_${req.status}` as Parameters<typeof t>[0])}
          </span>
          <span style={{ fontSize: "var(--fs-14)", fontWeight: 500, color: "var(--fg)" }}>
            {req.from_status} → {req.target_status}
          </span>
        </div>
        {req.reason && <p style={{ margin: "4px 0 0", fontSize: "var(--fs-12)", color: "var(--mu)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{req.reason}</p>}
        <div style={{ marginTop: 4, fontSize: "var(--fs-12)", color: "var(--mu)" }}>
          {t("approval_gate_expires_at")}: {new Date(req.expires_at).toLocaleString()}
        </div>
      </div>
      {req.status === "pending" && (
        <Button
          variant="secondary"
          size="sm"
          type="button"
          onClick={() => onSelect(req)}
          data-testid={`review-request-${req.id}`}
        >
          {t("approval_gate_btn_review")}
        </Button>
      )}
    </div>
  );
}

export function ApprovalQueueList({ requests, onSelectRequest }: ApprovalQueueListProps) {
  if (requests.length === 0) {
    return (
      <div style={{ padding: "var(--sp-32) 0", textAlign: "center", fontSize: "var(--fs-14)", color: "var(--mu)" }}>
        {t("approval_gate_queue_empty")}
      </div>
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)" }} data-testid="approval-queue-list">
      {requests.map((req) => (
        <ApprovalQueueItem key={req.id} req={req} onSelect={onSelectRequest} />
      ))}
    </div>
  );
}

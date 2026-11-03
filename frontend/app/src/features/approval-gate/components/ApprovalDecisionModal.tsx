import { useState } from "react";
import { t } from "../../../core/i18n";
import { Modal } from "../../../ui/components/Modal";
import { Button } from "../../../ui/components/Button";
import { Textarea } from "../../../ui/components/Textarea";
import type { ApprovalRequest } from "../types";

interface ApprovalDecisionModalProps {
  request: ApprovalRequest;
  onApprove: (id: string, reason?: string) => Promise<void>;
  onReject: (id: string, reason?: string) => Promise<void>;
  onClose: () => void;
  onViewDiff?: (taskId: string) => void;
}

function useApprovalAction(
  request: ApprovalRequest,
  onApprove: (id: string, reason?: string) => Promise<void>,
  onReject: (id: string, reason?: string) => Promise<void>,
  onClose: () => void,
) {
  const [reason, setReason] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleAction = async (action: "approve" | "reject") => {
    setSubmitting(true);
    setError(null);
    try {
      const trimmed = reason.trim() || undefined;
      if (action === "approve") {
        await onApprove(request.id, trimmed);
      } else {
        await onReject(request.id, trimmed);
      }
      onClose();
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSubmitting(false);
    }
  };

  return { reason, setReason, submitting, error, handleAction };
}

interface ModalActionsProps {
  submitting: boolean;
  onClose: () => void;
  onReject: () => void;
  onApprove: () => void;
}

function ModalActions({ submitting, onClose, onReject, onApprove }: ModalActionsProps) {
  return (
    <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--sp-8)" }}>
      <Button type="button" variant="ghost" onClick={onClose} disabled={submitting}>
        {t("btn_close")}
      </Button>
      <Button
        type="button"
        variant="danger"
        onClick={onReject}
        disabled={submitting}
        data-testid="reject-button"
      >
        {t("approval_gate_btn_reject")}
      </Button>
      <Button
        type="button"
        variant="primary"
        onClick={onApprove}
        disabled={submitting}
        data-testid="approve-button"
      >
        {t("approval_gate_btn_approve")}
      </Button>
    </div>
  );
}

interface ModalBodyProps {
  request: ApprovalRequest;
  reason: string;
  onReasonChange: (val: string) => void;
  error: string | null;
  onViewDiff?: (taskId: string) => void;
}

function ModalBody({ request, reason, onReasonChange, error, onViewDiff }: ModalBodyProps) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)" }}>
      <p style={{ margin: 0, fontSize: "var(--fs-14)", color: "var(--mu)" }}>
        {request.from_status} → {request.target_status}
      </p>

      {request.reason && (
        <div style={{ padding: "var(--sp-12)", borderRadius: "var(--radius-8)", backgroundColor: "var(--sf2)", border: "1px solid var(--ln)", fontSize: "var(--fs-13)", color: "var(--fg)" }}>
          {request.reason}
        </div>
      )}

      {onViewDiff && (
        <div>
          <Button
            variant="ghost"
            size="sm"
            type="button"
            onClick={() => onViewDiff(request.task_id)}
            style={{ background: "none", border: "none", padding: 0, fontSize: "var(--fs-12)", color: "var(--bl)", textDecoration: "underline", height: "auto", minHeight: "unset" }}
          >
            {t("approval_gate_diff_link")}
          </Button>
        </div>
      )}

      <div>
        <Textarea
          value={reason}
          onChange={(e) => onReasonChange(e.target.value)}
          placeholder={t("approval_gate_reason_placeholder")}
          rows={3}
          data-testid="approval-reason-input"
        />
      </div>

      {error && <p style={{ margin: 0, fontSize: "var(--fs-12)", color: "var(--rd)" }}>{error}</p>}
    </div>
  );
}

export function ApprovalDecisionModal({
  request,
  onApprove,
  onReject,
  onClose,
  onViewDiff,
}: ApprovalDecisionModalProps) {
  const { reason, setReason, submitting, error, handleAction } = useApprovalAction(
    request,
    onApprove,
    onReject,
    onClose,
  );

  return (
    <Modal
      open={true}
      onClose={onClose}
      title={t("approval_gate_title")}
      size="md"
      data-testid="approval-decision-modal"
      footer={
        <ModalActions
          submitting={submitting}
          onClose={onClose}
          onReject={() => void handleAction("reject")}
          onApprove={() => void handleAction("approve")}
        />
      }
    >
      <ModalBody
        request={request}
        reason={reason}
        onReasonChange={setReason}
        error={error}
        onViewDiff={onViewDiff}
      />
    </Modal>
  );
}

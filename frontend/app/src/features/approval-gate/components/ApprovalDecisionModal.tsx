import { useState } from "react";
import { t } from "../../../core/i18n";
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
    <div className="mt-6 flex justify-end gap-3">
      <button
        type="button"
        onClick={onClose}
        disabled={submitting}
        className="rounded-lg px-4 py-2 text-sm text-zinc-400 hover:bg-zinc-800 transition-colors"
      >
        {t("btn_close")}
      </button>
      <button
        type="button"
        onClick={onReject}
        disabled={submitting}
        className="rounded-lg bg-red-600/20 text-red-400 border border-red-500/30 px-4 py-2 text-sm font-medium hover:bg-red-600/30 transition-colors disabled:opacity-50"
        data-testid="reject-button"
      >
        {t("approval_gate_btn_reject")}
      </button>
      <button
        type="button"
        onClick={onApprove}
        disabled={submitting}
        className="rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-500 transition-colors disabled:opacity-50"
        data-testid="approve-button"
      >
        {t("approval_gate_btn_approve")}
      </button>
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
    <>
      <h3 className="text-lg font-semibold text-zinc-100">{t("approval_gate_title")}</h3>
      <p className="mt-1 text-sm text-zinc-400">
        {request.from_status} → {request.target_status}
      </p>

      {request.reason && (
        <div className="mt-3 p-3 rounded bg-zinc-800/50 border border-zinc-800 text-sm text-zinc-300">
          {request.reason}
        </div>
      )}

      {onViewDiff && (
        <div className="mt-4">
          <button
            type="button"
            onClick={() => onViewDiff(request.task_id)}
            className="text-xs text-blue-400 hover:text-blue-300 underline"
          >
            {t("approval_gate_diff_link")}
          </button>
        </div>
      )}

      <div className="mt-4">
        <textarea
          value={reason}
          onChange={(e) => onReasonChange(e.target.value)}
          placeholder={t("approval_gate_reason_placeholder")}
          rows={3}
          className="w-full rounded-lg border border-zinc-700 bg-zinc-800 p-2.5 text-sm text-zinc-200 placeholder-zinc-500 focus:border-blue-500 focus:outline-none"
          data-testid="approval-reason-input"
        />
      </div>

      {error && <p className="mt-2 text-xs text-red-400">{error}</p>}
    </>
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
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
      data-testid="approval-decision-modal"
    >
      <div className="w-full max-w-lg rounded-xl border border-zinc-800 bg-zinc-900 p-6 shadow-xl">
        <ModalBody
          request={request}
          reason={reason}
          onReasonChange={setReason}
          error={error}
          onViewDiff={onViewDiff}
        />
        <ModalActions
          submitting={submitting}
          onClose={onClose}
          onReject={() => void handleAction("reject")}
          onApprove={() => void handleAction("approve")}
        />
      </div>
    </div>
  );
}

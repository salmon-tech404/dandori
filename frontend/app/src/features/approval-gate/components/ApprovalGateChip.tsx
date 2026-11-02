import { t } from "../../../core/i18n";

interface ApprovalGateChipProps {
  pendingCount: number;
  onClick?: () => void;
}

export function ApprovalGateChip({ pendingCount, onClick }: ApprovalGateChipProps) {
  if (pendingCount <= 0) return null;

  return (
    <button
      type="button"
      onClick={onClick}
      className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-amber-500/10 text-amber-500 border border-amber-500/20 hover:bg-amber-500/20 transition-colors"
      data-testid="approval-gate-chip"
    >
      <span className="w-1.5 h-1.5 rounded-full bg-amber-500 animate-pulse" />
      <span>
        {pendingCount} {t("approval_gate_chip_label")}
      </span>
    </button>
  );
}

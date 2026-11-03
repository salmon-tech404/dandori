import { t } from "../../../core/i18n";
import { Button } from "../../../ui/components/Button";

interface ApprovalGateChipProps {
  pendingCount: number;
  onClick?: () => void;
}

export function ApprovalGateChip({ pendingCount, onClick }: ApprovalGateChipProps) {
  if (pendingCount <= 0) return null;

  return (
    <Button
      variant="ghost"
      type="button"
      onClick={onClick}
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: "var(--sp-8)",
        padding: "4px 10px",
        borderRadius: "var(--radius-pill)",
        fontSize: "var(--fs-12)",
        fontWeight: 500,
        backgroundColor: "var(--orb)",
        color: "var(--or)",
        border: "1px solid var(--or)",
        height: "auto",
        minHeight: "unset",
      }}
      data-testid="approval-gate-chip"
    >
      <span
        style={{
          width: 6,
          height: 6,
          borderRadius: "var(--radius-pill)",
          backgroundColor: "var(--or)",
          animation: "dandori-pulse 2s infinite",
        }}
      />
      <span>
        {pendingCount} {t("approval_gate_chip_label")}
      </span>
    </Button>
  );
}

import { FC, CSSProperties } from "react";
import { t } from "../../core/i18n";
import type { TaskStatus } from "../../../../../shared/types";

export interface StatusBadgeProps {
  status: TaskStatus | string;
  label?: string;
  style?: CSSProperties;
}

function resolveStatusConfig(status: string): { color: string; bg: string; defaultKey: string } {
  switch (status) {
    case "running":
    case "in_progress":
      return { color: "var(--bl)", bg: "var(--blb)", defaultKey: "status_running" };
    case "needs_human":
      return { color: "var(--or)", bg: "var(--orb)", defaultKey: "status_needs_human" };
    case "in_review":
      return { color: "var(--tl)", bg: "var(--tlb)", defaultKey: "status_in_review" };
    case "ready_to_merge":
    case "done":
      return { color: "var(--gr)", bg: "var(--grb)", defaultKey: status === "done" ? "status_done" : "status_ready_to_merge" };
    case "failed":
      return { color: "var(--rd)", bg: "var(--rdb)", defaultKey: "status_failed" };
    case "cancelled":
    case "backlog":
    default:
      return { color: "var(--gy)", bg: "var(--gyb)", defaultKey: status === "cancelled" ? "status_cancelled" : "status_backlog" };
  }
}

export const StatusBadge: FC<StatusBadgeProps> = ({ status, label, style }) => {
  const { color, bg, defaultKey } = resolveStatusConfig(status);
  const displayLabel = label || t(defaultKey);

  return (
    <span
      style={{
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        height: "var(--chip-height)",
        padding: "0 8px",
        borderRadius: "var(--radius-pill)",
        backgroundColor: bg,
        color,
        fontSize: "var(--fs-12)",
        fontWeight: 600,
        fontFamily: "var(--font-sans)",
        lineHeight: 1,
        ...style,
      }}
    >
      {displayLabel}
    </span>
  );
};

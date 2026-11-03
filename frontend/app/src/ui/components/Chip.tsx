import { FC, ReactNode, CSSProperties, HTMLAttributes } from "react";
import { t } from "../../core/i18n";

export type ChipVariant = "default" | "blue" | "orange" | "teal" | "green" | "red" | "gray";

export interface ChipProps extends HTMLAttributes<HTMLSpanElement> {
  variant?: ChipVariant;
  size?: "sm" | "md";
  onRemove?: () => void;
  children: ReactNode;
}

const VARIANT_COLORS: Record<ChipVariant, { bg: string; fg: string }> = {
  default: { bg: "var(--gyb)", fg: "var(--gy)" },
  gray: { bg: "var(--gyb)", fg: "var(--gy)" },
  blue: { bg: "var(--blb)", fg: "var(--bl)" },
  orange: { bg: "var(--orb)", fg: "var(--or)" },
  teal: { bg: "var(--tlb)", fg: "var(--tl)" },
  green: { bg: "var(--grb)", fg: "var(--gr)" },
  red: { bg: "var(--rdb)", fg: "var(--rd)" },
};

function RemoveButton({ onRemove }: { onRemove: () => void }) {
  return (
    <button
      type="button"
      onClick={(e) => {
        e.stopPropagation();
        onRemove();
      }}
      aria-label={t("chip_remove")}
      style={{ display: "inline-flex", alignItems: "center", justifyContent: "center", background: "none", border: "none", color: "inherit", cursor: "pointer", padding: 0, marginLeft: "2px" }}
    >
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
    </button>
  );
}

function buildChipStyle(variant: ChipVariant, size: "sm" | "md", custom?: CSSProperties): CSSProperties {
  const { bg, fg } = VARIANT_COLORS[variant] || VARIANT_COLORS.default;
  return {
    display: "inline-flex",
    alignItems: "center",
    gap: "var(--sp-4)",
    height: size === "sm" ? "18px" : "var(--chip-height)",
    padding: size === "sm" ? "0 6px" : "0 8px",
    borderRadius: "var(--radius-pill)",
    fontSize: size === "sm" ? "11px" : "var(--fs-12)",
    fontWeight: 600,
    fontFamily: "var(--font-sans)",
    backgroundColor: bg,
    color: fg,
    whiteSpace: "nowrap",
    userSelect: "none",
    ...custom,
  };
}

export const Chip: FC<ChipProps> = ({ variant = "default", size = "md", onRemove, children, className = "", style, ...props }) => {
  const chipStyle = buildChipStyle(variant, size, style);
  return (
    <span style={chipStyle} className={`dandori-chip ${className}`} {...props}>
      {children}
      {onRemove && <RemoveButton onRemove={onRemove} />}
    </span>
  );
};

export const Badge: FC<ChipProps> = (props) => <Chip {...props} />;

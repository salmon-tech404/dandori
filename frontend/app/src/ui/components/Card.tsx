import { FC, ReactNode, CSSProperties, HTMLAttributes, KeyboardEvent } from "react";

export interface CardProps extends Omit<HTMLAttributes<HTMLDivElement>, "title"> {
  title?: ReactNode;
  subtitle?: ReactNode;
  actions?: ReactNode;
  padding?: "none" | "sm" | "md" | "lg";
  selected?: boolean;
}

const PADDING_MAP = {
  none: "0",
  sm: "var(--sp-8)",
  md: "var(--sp-16)",
  lg: "var(--sp-24)",
};

function CardHeader({ title, subtitle, actions }: { title?: ReactNode; subtitle?: ReactNode; actions?: ReactNode }) {
  if (!title && !subtitle && !actions) return null;
  return (
    <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: "var(--sp-12)" }}>
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-4)" }}>
        {title && <div style={{ fontWeight: 600, fontSize: "var(--fs-16)", color: "var(--fg)" }}>{title}</div>}
        {subtitle && <div style={{ fontSize: "var(--fs-13)", color: "var(--mu)" }}>{subtitle}</div>}
      </div>
      {actions && <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>{actions}</div>}
    </div>
  );
}

function buildCardStyle(selected: boolean, isClickable: boolean, padding: "none" | "sm" | "md" | "lg", custom?: CSSProperties): CSSProperties {
  return {
    backgroundColor: "var(--sf)",
    border: selected ? "2px solid var(--ac)" : "1px solid var(--ln)",
    borderRadius: "var(--radius-10)",
    padding: PADDING_MAP[padding],
    boxShadow: selected ? "0 0 0 1px var(--ac)" : "none",
    cursor: isClickable ? "pointer" : "default",
    display: "flex",
    flexDirection: "column",
    gap: "var(--sp-12)",
    ...custom,
  };
}

export const Card: FC<CardProps> = ({ title, subtitle, actions, children, padding = "md", selected = false, onClick, className = "", style, ...props }) => {
  const isClickable = Boolean(onClick);
  const cardStyle = buildCardStyle(selected, isClickable, padding, style);

  const handleKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (isClickable && (e.key === "Enter" || e.key === " ")) {
      e.preventDefault();
      onClick?.(e as unknown as React.MouseEvent<HTMLDivElement>);
    }
  };

  return (
    <div style={cardStyle} className={`dandori-card ${className}`} onClick={onClick} role={isClickable ? "button" : undefined} tabIndex={isClickable ? 0 : undefined} onKeyDown={handleKeyDown} {...props}>
      <CardHeader title={title} subtitle={subtitle} actions={actions} />
      {children}
    </div>
  );
};

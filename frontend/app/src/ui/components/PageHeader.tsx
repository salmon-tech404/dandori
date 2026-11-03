import { FC, ReactNode, CSSProperties } from "react";

export interface PageHeaderProps {
  title: ReactNode;
  subtitle?: ReactNode;
  badges?: ReactNode;
  actions?: ReactNode;
  breadcrumbs?: ReactNode;
  className?: string;
  style?: CSSProperties;
}

function PageHeaderTitleRow({ title, badges, actions }: { title: ReactNode; badges?: ReactNode; actions?: ReactNode }) {
  return (
    <div style={{ display: "flex", flexWrap: "wrap", alignItems: "center", justifyContent: "space-between", gap: "var(--sp-12)" }}>
      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-12)", minWidth: 0 }}>
        <h1 style={{ margin: 0, fontSize: "var(--fs-18)", fontWeight: 700, color: "var(--fg)", fontFamily: "var(--font-sans)" }}>
          {title}
        </h1>
        {badges && <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>{badges}</div>}
      </div>
      {actions && <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>{actions}</div>}
    </div>
  );
}

export const PageHeader: FC<PageHeaderProps> = ({ title, subtitle, badges, actions, breadcrumbs, className = "", style }) => {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)", paddingBottom: "var(--sp-12)", borderBottom: "1px solid var(--ln)", marginBottom: "var(--sp-16)", ...style }} className={`dandori-page-header ${className}`}>
      {breadcrumbs && <div style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{breadcrumbs}</div>}
      <PageHeaderTitleRow title={title} badges={badges} actions={actions} />
      {subtitle && <div style={{ fontSize: "var(--fs-13)", color: "var(--mu)" }}>{subtitle}</div>}
    </div>
  );
};

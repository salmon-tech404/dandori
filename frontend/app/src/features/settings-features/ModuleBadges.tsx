import { FC } from "react";
import { t } from "../../core/i18n";
import { FeatureModuleItem } from "./types";

export const ModuleBadges: FC<{ item: FeatureModuleItem }> = ({ item }) => {
  const effortText = item.effort === "heavy" ? t("effort_heavy") : item.effort === "medium" ? t("effort_medium") : t("effort_light");
  const isHighRisk = item.risk === "high";

  return (
    <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
      {item.requires_worker_restart && (
        <span style={{ fontSize: "var(--fs-12)", color: "var(--or)", backgroundColor: "var(--orb)", padding: "2px 8px", borderRadius: "var(--radius-pill)", fontWeight: 600 }}>
          {t("worker_restart_badge")}
        </span>
      )}
      {isHighRisk && (
        <span style={{ fontSize: "var(--fs-12)", color: "var(--rd)", backgroundColor: "var(--rdb)", padding: "2px 8px", borderRadius: "var(--radius-pill)", fontWeight: 600 }}>
          {t("risk_high")}
        </span>
      )}
      <span style={{ fontSize: "var(--fs-12)", color: "var(--mu)", backgroundColor: "var(--sf2)", padding: "2px 8px", borderRadius: "var(--radius-pill)", fontWeight: 600 }}>
        {effortText}
      </span>
    </div>
  );
};

import { FC } from "react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";

export interface PacksSectionProps {
  onApplyPack: (packId: string) => void;
  enabledCount: number;
  totalCount: number;
}

export const PacksSection: FC<PacksSectionProps> = ({ onApplyPack, enabledCount, totalCount }) => {
  return (
    <div style={{ backgroundColor: "var(--sf)", borderRadius: "var(--radius-10)", border: "1px solid var(--ln)", padding: "var(--sp-16)", marginBottom: "var(--sp-24)", display: "flex", alignItems: "center", justifyContent: "space-between", flexWrap: "wrap", gap: "var(--sp-16)" }}>
      <div>
        <h3 style={{ margin: "0 0 var(--sp-4) 0", fontSize: "var(--fs-16)", fontWeight: 600 }}>{t("sample_packs_title")}</h3>
        <p style={{ margin: 0, fontSize: "var(--fs-13)", color: "var(--mu)" }}>{t("sample_packs_desc")}</p>
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
        <Button variant="secondary" onClick={() => onApplyPack("solo-light")}>{t("pack_solo")}</Button>
        <Button variant="secondary" onClick={() => onApplyPack("small-team")}>{t("pack_team")}</Button>
        <Button variant="secondary" onClick={() => onApplyPack("full")}>{t("pack_full")}</Button>
        <span style={{ fontSize: "var(--fs-12)", fontWeight: 600, color: "var(--mu)", backgroundColor: "var(--sf2)", padding: "6px 12px", borderRadius: "var(--radius-pill)" }}>
          {t("group_count_badge", { on: enabledCount, total: totalCount })}
        </span>
      </div>
    </div>
  );
};

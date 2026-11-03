import { FC, useState } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { ModuleRow } from "./ModuleRow";
import { FeatureModuleItem } from "./types";

export interface GroupCardProps {
  title: string;
  items: FeatureModuleItem[];
  onToggle: (item: FeatureModuleItem) => void;
  onConfigure: (item: FeatureModuleItem) => void;
}

export const GroupCard: FC<GroupCardProps> = ({ title, items, onToggle, onConfigure }) => {
  const [expanded, setExpanded] = useState(true);
  const enabledCount = items.filter((i) => i.enabled).length;

  return (
    <div style={{ backgroundColor: "var(--sf)", borderRadius: "var(--radius-10)", border: "1px solid var(--ln)", marginBottom: "var(--sp-16)", overflow: "hidden" }}>
      <Button
        type="button"
        variant="ghost"
        onClick={() => setExpanded(!expanded)}
        style={{
          width: "100%",
          padding: "var(--sp-12) var(--sp-16)",
          backgroundColor: "var(--sf2)",
          borderBottom: expanded ? "1px solid var(--ln)" : "none",
          borderRadius: 0,
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
          {expanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
          <span style={{ fontWeight: 600, fontSize: "var(--fs-14)", color: "var(--fg)" }}>{title}</span>
        </div>
        <span style={{ fontSize: "var(--fs-12)", fontWeight: 600, color: "var(--mu)", backgroundColor: "var(--sf)", padding: "2px 8px", borderRadius: "var(--radius-pill)", border: "1px solid var(--ln)" }}>
          {t("group_count_badge", { on: enabledCount, total: items.length })}
        </span>
      </Button>
      {expanded && (
        <div>
          {items.map((item) => (
            <ModuleRow key={item.id} item={item} onToggle={onToggle} onConfigure={onConfigure} />
          ))}
        </div>
      )}
    </div>
  );
};

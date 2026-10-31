import { FC } from "react";
import { Settings as SettingsIcon } from "lucide-react";
import { t } from "../../core/i18n";
import { Tooltip } from "../../ui/components/Tooltip";
import { Switch } from "./Switch";
import { ModuleBadges } from "./ModuleBadges";
import { FeatureModuleItem } from "./types";

export interface ModuleRowProps {
  item: FeatureModuleItem;
  onToggle: (item: FeatureModuleItem) => void;
  onConfigure: (item: FeatureModuleItem) => void;
}

export const ModuleRow: FC<ModuleRowProps> = ({ item, onToggle, onConfigure }) => {
  const isSwitchDisabled = item.core || item.locked_by_parent;
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "var(--sp-12) var(--sp-16)", borderBottom: "1px solid var(--ln)", backgroundColor: "var(--sf)", gap: "var(--sp-16)" }}>
      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-12)", flex: 1, minWidth: 0 }}>
        <Switch checked={item.enabled} disabled={isSwitchDisabled} onChange={() => onToggle(item)} aria-label={item.name} />
        {item.core && <Tooltip content={t("core_module_tooltip")} />}
        <div style={{ display: "flex", flexDirection: "column", gap: "2px", minWidth: 0 }}>
          <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
            <span style={{ fontWeight: 600, fontSize: "var(--fs-14)", color: "var(--fg)" }}>{item.name}</span>
            {item.locked_by_parent && <span style={{ fontSize: "var(--fs-12)", color: "var(--mu)", fontStyle: "italic" }}>({t("locked_by_parent")})</span>}
          </div>
          <span style={{ fontSize: "var(--fs-13)", color: "var(--mu)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{item.description}</span>
          {item.requires.length > 0 && <span style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t("requires_label", { modules: item.requires.join(", ") })}</span>}
        </div>
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-12)" }}>
        <ModuleBadges item={item} />
        <button
          type="button"
          disabled={!item.enabled}
          aria-label={t("configure_module")}
          onClick={() => onConfigure(item)}
          style={{ display: "inline-flex", alignItems: "center", justifyContent: "center", width: "32px", height: "32px", minWidth: "var(--touch-target)", minHeight: "var(--touch-target)", borderRadius: "var(--radius-8)", border: "none", backgroundColor: "transparent", color: item.enabled ? "var(--fg)" : "var(--ln)", cursor: item.enabled ? "pointer" : "not-allowed" }}
        >
          <SettingsIcon size={16} />
        </button>
      </div>
    </div>
  );
};

import { FC } from "react";
import { Search } from "lucide-react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { ScopeType } from "./types";

export interface SettingsHeaderProps {
  scope: ScopeType;
  onScopeChange: (scope: ScopeType) => void;
  search: string;
  onSearchChange: (search: string) => void;
  filter: "all" | "enabled" | "disabled";
  onFilterChange: (filter: "all" | "enabled" | "disabled") => void;
}

export const SettingsHeader: FC<SettingsHeaderProps> = ({ scope, onScopeChange, search, onSearchChange, filter, onFilterChange }) => {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-16)", marginBottom: "var(--sp-24)" }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", flexWrap: "wrap", gap: "var(--sp-16)" }}>
        <div>
          <h2 style={{ margin: "0 0 var(--sp-4) 0", fontSize: "var(--fs-18)", fontWeight: 700 }}>{t("settings_features_title")}</h2>
          <p style={{ margin: 0, fontSize: "var(--fs-13)", color: "var(--mu)" }}>{t("settings_features_desc")}</p>
        </div>
        <div style={{ display: "flex", gap: "var(--sp-8)" }}>
          <Button variant={scope === "system" ? "primary" : "secondary"} onClick={() => onScopeChange("system")}>{t("scope_system")}</Button>
          <Button variant={scope === "workspace" ? "primary" : "secondary"} onClick={() => onScopeChange("workspace")}>{t("scope_workspace")}</Button>
          <Button variant={scope === "project" ? "primary" : "secondary"} onClick={() => onScopeChange("project")}>{t("scope_project")}</Button>
        </div>
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-12)", flexWrap: "wrap" }}>
        <div style={{ position: "relative", flex: 1, minWidth: "260px" }}>
          <Search size={16} style={{ position: "absolute", left: "12px", top: "50%", transform: "translateY(-50%)", color: "var(--mu)" }} />
          <input
            type="text"
            value={search}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder={t("search_features_placeholder")}
            style={{ width: "100%", height: "var(--btn-height)", paddingLeft: "36px", paddingRight: "12px", borderRadius: "var(--radius-8)", border: "1px solid var(--ln)", backgroundColor: "var(--sf)", color: "var(--fg)", fontSize: "var(--fs-14)", outline: "none" }}
          />
        </div>
        <div style={{ display: "flex", gap: "4px" }}>
          <Button variant={filter === "all" ? "primary" : "ghost"} size="sm" onClick={() => onFilterChange("all")}>{t("filter_all")}</Button>
          <Button variant={filter === "enabled" ? "primary" : "ghost"} size="sm" onClick={() => onFilterChange("enabled")}>{t("filter_enabled")}</Button>
          <Button variant={filter === "disabled" ? "primary" : "ghost"} size="sm" onClick={() => onFilterChange("disabled")}>{t("filter_disabled")}</Button>
        </div>
      </div>
    </div>
  );
};

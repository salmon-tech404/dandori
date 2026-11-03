import { RefreshCw } from "lucide-react";
import { t } from "../../core/i18n";

export function AutoSwitchNotice() {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)", borderRadius: "var(--radius-6)", border: "1px solid var(--bl)", backgroundColor: "var(--blb)", padding: "10px", fontSize: "var(--fs-12)", color: "var(--bl)" }}>
      <RefreshCw style={{ width: 16, height: 16, flexShrink: 0, animation: "dandori-spin 3s linear infinite" }} />
      <span>{t("auto_switch_active_notice")}</span>
    </div>
  );
}

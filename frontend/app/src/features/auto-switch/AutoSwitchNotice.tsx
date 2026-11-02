import { RefreshCw } from "lucide-react";
import { t } from "../../core/i18n";

export function AutoSwitchNotice() {
  return (
    <div className="flex items-center gap-2 rounded border border-blue-200 bg-blue-50/70 p-2.5 text-xs text-blue-700 dark:border-blue-900/50 dark:bg-blue-950/40 dark:text-blue-300">
      <RefreshCw className="h-4 w-4 shrink-0 animate-spin-slow" />
      <span>{t("auto_switch_active_notice")}</span>
    </div>
  );
}

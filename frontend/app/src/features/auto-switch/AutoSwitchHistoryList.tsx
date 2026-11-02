import { ArrowRight, History } from "lucide-react";
import { t } from "../../core/i18n";
import type { AutoSwitchHistoryItem } from "./types";

interface AutoSwitchHistoryListProps {
  items: AutoSwitchHistoryItem[];
}

function HistoryItemRow({ item }: { item: AutoSwitchHistoryItem }) {
  return (
    <div className="flex items-center justify-between rounded border border-slate-100 bg-slate-50/50 p-2 text-xs dark:border-slate-800 dark:bg-slate-800/40">
      <div className="flex items-center gap-2">
        <span className="rounded bg-blue-100 px-1.5 py-0.5 text-[10px] font-semibold text-blue-700 dark:bg-blue-950/60 dark:text-blue-300">
          #{item.switch_count}
        </span>
        <div className="flex items-center gap-1.5 text-slate-700 dark:text-slate-300">
          <span className="font-mono text-[11px]">{item.from_account_id}</span>
          <ArrowRight className="h-3 w-3 text-slate-400" />
          <span className="font-mono text-[11px] font-medium text-slate-900 dark:text-slate-100">
            {item.to_account_id}
          </span>
        </div>
      </div>
      <div className="text-[10px] text-slate-400 dark:text-slate-500">
        {item.created_at}
      </div>
    </div>
  );
}

export function AutoSwitchHistoryList({ items }: AutoSwitchHistoryListProps) {
  if (items.length === 0) {
    return (
      <div className="flex items-center gap-1.5 py-2 text-xs text-slate-400 dark:text-slate-500">
        <History className="h-3.5 w-3.5" />
        <span>{t("auto_switch_no_history")}</span>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      <h5 className="flex items-center gap-1.5 text-xs font-semibold text-slate-800 dark:text-slate-200">
        <History className="h-3.5 w-3.5 text-blue-500" />
        <span>{t("auto_switch_history_title")}</span>
      </h5>
      <div className="space-y-1.5">
        {items.map((item) => (
          <HistoryItemRow key={item.id} item={item} />
        ))}
      </div>
    </div>
  );
}

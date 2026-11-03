import { ArrowRight, History } from "lucide-react";
import { t } from "../../core/i18n";
import type { AutoSwitchHistoryItem } from "./types";

interface AutoSwitchHistoryListProps {
  items: AutoSwitchHistoryItem[];
}

function HistoryItemRow({ item }: { item: AutoSwitchHistoryItem }) {
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", borderRadius: "var(--radius-4)", border: "1px solid var(--ln)", backgroundColor: "var(--sf2)", padding: "var(--sp-8)", fontSize: "var(--fs-12)" }}>
      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
        <span style={{ borderRadius: "var(--radius-4)", backgroundColor: "var(--blb)", padding: "2px 6px", fontSize: "10px", fontWeight: 600, color: "var(--bl)" }}>
          #{item.switch_count}
        </span>
        <div style={{ display: "flex", alignItems: "center", gap: 6, color: "var(--fg)" }}>
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "11px" }}>{item.from_account_id}</span>
          <ArrowRight style={{ width: 12, height: 12, color: "var(--mu)" }} />
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "11px", fontWeight: 500, color: "var(--fg)" }}>
            {item.to_account_id}
          </span>
        </div>
      </div>
      <div style={{ fontSize: "10px", color: "var(--mu)" }}>
        {item.created_at}
      </div>
    </div>
  );
}

export function AutoSwitchHistoryList({ items }: AutoSwitchHistoryListProps) {
  if (items.length === 0) {
    return (
      <div style={{ display: "flex", alignItems: "center", gap: 6, padding: "var(--sp-8) 0", fontSize: "var(--fs-12)", color: "var(--mu)" }}>
        <History style={{ width: 14, height: 14 }} />
        <span>{t("auto_switch_no_history")}</span>
      </div>
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
      <h5 style={{ display: "flex", alignItems: "center", gap: 6, fontSize: "var(--fs-12)", fontWeight: 600, color: "var(--fg)", margin: 0 }}>
        <History style={{ width: 14, height: 14, color: "var(--bl)" }} />
        <span>{t("auto_switch_history_title")}</span>
      </h5>
      <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
        {items.map((item) => (
          <HistoryItemRow key={item.id} item={item} />
        ))}
      </div>
    </div>
  );
}

import { t } from "../../core/i18n";
import type { NotificationHistoryRecord } from "./types";

interface Props {
  records: NotificationHistoryRecord[];
}

function HistoryRow({ r }: { r: NotificationHistoryRecord }) {
  const badgeCls =
    r.status === "sent"
      ? "bg-emerald-950 text-emerald-400"
      : r.status === "failed"
        ? "bg-rose-950 text-rose-400"
        : "bg-amber-950 text-amber-400";

  return (
    <tr className="hover:bg-neutral-800/30">
      <td className="px-3 py-2 font-medium text-neutral-200">{r.title}</td>
      <td className="px-3 py-2 uppercase text-neutral-400">{r.event_type}</td>
      <td className="px-3 py-2">
        <span className={`rounded px-1.5 py-0.5 text-[10px] font-semibold ${badgeCls}`}>
          {r.status}
        </span>
      </td>
      <td className="px-3 py-2 text-neutral-400">
        {new Date(r.created_at).toLocaleString()}
      </td>
    </tr>
  );
}

export function NotificationsHistoryTable({ records }: Props) {
  if (records.length === 0) {
    return (
      <div className="py-6 text-center text-sm text-neutral-400">
        {t("notifications_empty_history")}
      </div>
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left text-xs text-neutral-300">
        <thead className="border-b border-neutral-700 bg-neutral-800/50 text-neutral-400">
          <tr>
            <th className="px-3 py-2">{t("notifications_col_title")}</th>
            <th className="px-3 py-2">{t("notifications_col_event")}</th>
            <th className="px-3 py-2">{t("notifications_col_status")}</th>
            <th className="px-3 py-2">{t("notifications_col_time")}</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-neutral-800">
          {records.map((r) => (
            <HistoryRow key={r.id} r={r} />
          ))}
        </tbody>
      </table>
    </div>
  );
}

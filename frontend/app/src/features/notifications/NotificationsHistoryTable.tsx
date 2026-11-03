import { t } from "../../core/i18n";
import type { NotificationHistoryRecord } from "./types";

interface Props {
  records: NotificationHistoryRecord[];
}

function HistoryRow({ r }: { r: NotificationHistoryRecord }) {
  const isSent = r.status === "sent";
  const isFailed = r.status === "failed";
  const bg = isSent ? "var(--grb)" : isFailed ? "var(--rdb)" : "var(--orb)";
  const color = isSent ? "var(--gr)" : isFailed ? "var(--rd)" : "var(--or)";

  return (
    <tr style={{ borderBottom: "1px solid var(--ln)", color: "var(--fg)", fontSize: "var(--fs-13)" }}>
      <td style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 500 }}>{r.title}</td>
      <td style={{ padding: "var(--sp-8) var(--sp-12)", textTransform: "uppercase", fontSize: "11px", color: "var(--mu)", fontWeight: 600 }}>{r.event_type}</td>
      <td style={{ padding: "var(--sp-8) var(--sp-12)" }}>
        <span style={{ padding: "2px 8px", borderRadius: "var(--radius-pill)", fontSize: "var(--fs-12)", fontWeight: 600, backgroundColor: bg, color }}>
          {r.status}
        </span>
      </td>
      <td style={{ padding: "var(--sp-8) var(--sp-12)", color: "var(--mu)" }}>
        {new Date(r.created_at).toLocaleString()}
      </td>
    </tr>
  );
}

export function NotificationsHistoryTable({ records }: Props) {
  if (records.length === 0) {
    return (
      <div style={{ borderRadius: "var(--radius-8)", border: "1px solid var(--ln)", backgroundColor: "var(--sf2)", padding: "var(--sp-24)", textAlign: "center", fontSize: "var(--fs-13)", color: "var(--mu)" }}>
        {t("notifications_empty_history")}
      </div>
    );
  }

  return (
    <div style={{ overflowX: "auto", borderRadius: "var(--radius-8)", border: "1px solid var(--ln)" }}>
      <table style={{ width: "100%", borderCollapse: "collapse", textAlign: "left", fontSize: "var(--fs-13)" }}>
        <thead>
          <tr style={{ backgroundColor: "var(--sf2)", color: "var(--mu)", borderBottom: "1px solid var(--ln)" }}>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("notifications_col_title")}</th>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("notifications_col_event")}</th>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("notifications_col_status")}</th>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("notifications_col_time")}</th>
          </tr>
        </thead>
        <tbody>
          {records.map((r) => (
            <HistoryRow key={r.id} r={r} />
          ))}
        </tbody>
      </table>
    </div>
  );
}

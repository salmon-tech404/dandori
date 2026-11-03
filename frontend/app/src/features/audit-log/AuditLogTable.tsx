import { t } from "../../core/i18n";
import type { AuditLogEntry } from "./types";

interface AuditLogTableProps {
  entries: AuditLogEntry[];
}

function AuditLogRow({ item }: { item: AuditLogEntry }) {
  return (
    <tr style={{ borderBottom: "1px solid var(--ln)", color: "var(--fg)" }}>
      <td style={{ padding: "var(--sp-8) var(--sp-12)", fontFamily: "var(--font-mono)", color: "var(--mu)" }}>#{item.sequence}</td>
      <td style={{ padding: "var(--sp-8) var(--sp-12)", whiteSpace: "nowrap", color: "var(--mu)" }}>
        {new Date(item.created_at).toLocaleString()}
      </td>
      <td style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 500 }}>
        {item.actor_name || item.actor_id}
      </td>
      <td style={{ padding: "var(--sp-8) var(--sp-12)", fontFamily: "var(--font-mono)", color: "var(--bl)" }}>{item.action}</td>
      <td style={{ padding: "var(--sp-8) var(--sp-12)", color: "var(--mu)" }}>
        {item.target_type}:{item.target_id}
      </td>
      <td style={{ maxWidth: "240px", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", padding: "var(--sp-8) var(--sp-12)", fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--mu)" }} title={item.details}>
        {item.details}
      </td>
      <td style={{ padding: "var(--sp-8) var(--sp-12)", fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--mu)" }} title={item.hash}>
        {item.hash.slice(0, 10)}...
      </td>
    </tr>
  );
}

export function AuditLogTable({ entries }: AuditLogTableProps) {
  if (entries.length === 0) {
    return (
      <div style={{ borderRadius: "var(--radius-8)", border: "1px solid var(--ln)", backgroundColor: "var(--sf2)", padding: "var(--sp-24)", textAlign: "center", fontSize: "var(--fs-13)", color: "var(--mu)" }}>
        {t("audit_log_empty")}
      </div>
    );
  }

  return (
    <div style={{ overflowX: "auto", borderRadius: "var(--radius-8)", border: "1px solid var(--ln)" }}>
      <table style={{ width: "100%", borderCollapse: "collapse", textAlign: "left", fontSize: "var(--fs-13)" }}>
        <thead>
          <tr style={{ backgroundColor: "var(--sf2)", color: "var(--mu)", borderBottom: "1px solid var(--ln)" }}>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("audit_log_col_seq")}</th>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("audit_log_col_time")}</th>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("audit_log_col_actor")}</th>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("audit_log_col_action")}</th>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("audit_log_col_target")}</th>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("audit_log_col_details")}</th>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("audit_log_col_hash")}</th>
          </tr>
        </thead>
        <tbody>
          {entries.map((item) => (
            <AuditLogRow key={item.id} item={item} />
          ))}
        </tbody>
      </table>
    </div>
  );
}

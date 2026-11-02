import { t } from "../../core/i18n";
import type { AuditLogEntry } from "./types";

interface AuditLogTableProps {
  entries: AuditLogEntry[];
}

function AuditLogRow({ item }: { item: AuditLogEntry }) {
  return (
    <tr className="hover:bg-neutral-800/40">
      <td className="px-3 py-2 font-mono text-neutral-400">#{item.sequence}</td>
      <td className="px-3 py-2 whitespace-nowrap text-neutral-400">
        {new Date(item.created_at).toLocaleString()}
      </td>
      <td className="px-3 py-2 font-medium text-neutral-200">
        {item.actor_name || item.actor_id}
      </td>
      <td className="px-3 py-2 font-mono text-indigo-300">{item.action}</td>
      <td className="px-3 py-2 text-neutral-400">
        {item.target_type}:{item.target_id}
      </td>
      <td className="max-w-xs truncate px-3 py-2 font-mono text-[11px] text-neutral-400" title={item.details}>
        {item.details}
      </td>
      <td className="px-3 py-2 font-mono text-[11px] text-neutral-500" title={item.hash}>
        {item.hash.slice(0, 10)}...
      </td>
    </tr>
  );
}

export function AuditLogTable({ entries }: AuditLogTableProps) {
  if (entries.length === 0) {
    return (
      <div className="rounded border border-neutral-800 bg-neutral-950 p-6 text-center text-xs text-neutral-400">
        {t("audit_log_empty")}
      </div>
    );
  }

  return (
    <div className="overflow-x-auto rounded border border-neutral-800">
      <table className="w-full text-left text-xs text-neutral-300">
        <thead className="border-b border-neutral-800 bg-neutral-950 text-neutral-400">
          <tr>
            <th className="px-3 py-2">{t("audit_log_col_seq")}</th>
            <th className="px-3 py-2">{t("audit_log_col_time")}</th>
            <th className="px-3 py-2">{t("audit_log_col_actor")}</th>
            <th className="px-3 py-2">{t("audit_log_col_action")}</th>
            <th className="px-3 py-2">{t("audit_log_col_target")}</th>
            <th className="px-3 py-2">{t("audit_log_col_details")}</th>
            <th className="px-3 py-2">{t("audit_log_col_hash")}</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-neutral-800 bg-neutral-900/50">
          {entries.map((item) => (
            <AuditLogRow key={item.id} item={item} />
          ))}
        </tbody>
      </table>
    </div>
  );
}

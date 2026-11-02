/**
 * Bảng lịch sử trích xuất bài học cho auto-lessons (P3-11).
 */

import React from "react";
import { t } from "../../core/i18n";
import type { AutoLessonsRunRecord } from "./types";

interface AutoLessonsHistoryTableProps {
  runs: AutoLessonsRunRecord[];
}

const HistoryRow: React.FC<{ run: AutoLessonsRunRecord }> = ({ run }) => (
  <tr className="border-b text-xs hover:bg-muted/50">
    <td className="p-2 font-mono">{run.task_id}</td>
    <td className="p-2">
      <span className="rounded bg-muted px-2 py-0.5 text-xs font-medium">
        {run.status}
      </span>
    </td>
    <td className="p-2 text-muted-foreground">{run.reason || "-"}</td>
    <td className="p-2 text-muted-foreground">
      {new Date(run.created_at).toLocaleString()}
    </td>
  </tr>
);

export const AutoLessonsHistoryTable: React.FC<AutoLessonsHistoryTableProps> = ({
  runs,
}) => {
  if (runs.length === 0) {
    return (
      <div className="py-6 text-center text-sm text-muted-foreground">
        {t("auto_lessons_empty_history")}
      </div>
    );
  }

  return (
    <div className="overflow-x-auto rounded border">
      <table className="w-full text-left text-sm">
        <thead className="border-b bg-muted/40 text-xs text-muted-foreground">
          <tr>
            <th className="p-2">{t("auto_lessons_col_task")}</th>
            <th className="p-2">{t("auto_lessons_col_status")}</th>
            <th className="p-2">{t("auto_lessons_col_reason")}</th>
            <th className="p-2">{t("auto_lessons_col_time")}</th>
          </tr>
        </thead>
        <tbody>
          {runs.map((r) => (
            <HistoryRow key={r.id} run={r} />
          ))}
        </tbody>
      </table>
    </div>
  );
};

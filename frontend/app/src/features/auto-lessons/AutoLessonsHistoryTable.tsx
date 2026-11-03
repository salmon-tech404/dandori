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
  <tr style={{ borderBottom: "1px solid var(--ln)", color: "var(--fg)", fontSize: "var(--fs-13)" }}>
    <td style={{ padding: "var(--sp-8) var(--sp-12)", fontFamily: "var(--font-mono)" }}>{run.task_id}</td>
    <td style={{ padding: "var(--sp-8) var(--sp-12)" }}>
      <span style={{ padding: "2px 8px", borderRadius: "var(--radius-pill)", fontSize: "var(--fs-12)", fontWeight: 600, backgroundColor: run.status === "completed" ? "var(--grb)" : "var(--sf2)", color: run.status === "completed" ? "var(--gr)" : "var(--fg)" }}>
        {run.status}
      </span>
    </td>
    <td style={{ padding: "var(--sp-8) var(--sp-12)", color: "var(--mu)" }}>{run.reason || "-"}</td>
    <td style={{ padding: "var(--sp-8) var(--sp-12)", color: "var(--mu)" }}>
      {new Date(run.created_at).toLocaleString()}
    </td>
  </tr>
);

export const AutoLessonsHistoryTable: React.FC<AutoLessonsHistoryTableProps> = ({
  runs,
}) => {
  if (runs.length === 0) {
    return (
      <div style={{ borderRadius: "var(--radius-8)", border: "1px solid var(--ln)", backgroundColor: "var(--sf2)", padding: "var(--sp-24)", textAlign: "center", fontSize: "var(--fs-13)", color: "var(--mu)" }}>
        {t("auto_lessons_empty_history")}
      </div>
    );
  }

  return (
    <div style={{ overflowX: "auto", borderRadius: "var(--radius-8)", border: "1px solid var(--ln)" }}>
      <table style={{ width: "100%", borderCollapse: "collapse", textAlign: "left", fontSize: "var(--fs-13)" }}>
        <thead>
          <tr style={{ backgroundColor: "var(--sf2)", color: "var(--mu)", borderBottom: "1px solid var(--ln)" }}>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("auto_lessons_col_task")}</th>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("auto_lessons_col_status")}</th>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("auto_lessons_col_reason")}</th>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("auto_lessons_col_time")}</th>
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

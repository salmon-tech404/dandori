/**
 * Panel chính quản lý auto-lessons (P3-11).
 */

import React from "react";
import { t } from "../../core/i18n";
import { AutoLessonsConfigForm } from "./AutoLessonsConfigForm";
import { AutoLessonsHistoryTable } from "./AutoLessonsHistoryTable";
import { useAutoLessons } from "./useAutoLessons";

interface AutoLessonsPanelProps {
  projectId: string | null;
}

export const AutoLessonsPanel: React.FC<AutoLessonsPanelProps> = ({
  projectId,
}) => {
  const { config, runs, loading, error, saveConfig } =
    useAutoLessons(projectId);

  if (loading && !config) {
    return <div style={{ padding: "var(--sp-16)", fontSize: "var(--fs-13)", color: "var(--mu)" }}>{t("loading")}</div>;
  }

  return (
    <div style={{ backgroundColor: "var(--sf)", borderRadius: "var(--radius-10)", border: "1px solid var(--ln)", padding: "var(--sp-16)", marginBottom: "var(--sp-16)", display: "flex", flexDirection: "column", gap: "var(--sp-16)" }}>
      <div>
        <h2 style={{ margin: "0 0 2px 0", fontSize: "var(--fs-16)", fontWeight: 600, color: "var(--fg)" }}>{t("auto_lessons_title")}</h2>
        <p style={{ margin: 0, fontSize: "var(--fs-13)", color: "var(--mu)" }}>
          {t("auto_lessons_desc")}
        </p>
      </div>

      {error && <div style={{ fontSize: "var(--fs-13)", color: "var(--rd)" }}>{error}</div>}

      {config && (
        <AutoLessonsConfigForm config={config} onSave={saveConfig} />
      )}

      <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
        <h3 style={{ margin: "0 0 var(--sp-4) 0", fontSize: "var(--fs-14)", fontWeight: 600, color: "var(--fg)" }}>
          {t("auto_lessons_history_title")}
        </h3>
        <AutoLessonsHistoryTable runs={runs} />
      </div>
    </div>
  );
};

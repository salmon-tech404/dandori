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
    return <div className="p-4 text-sm text-muted-foreground">Đang tải...</div>;
  }

  return (
    <div className="space-y-6 p-4">
      <div>
        <h2 className="text-lg font-semibold">{t("auto_lessons_title")}</h2>
        <p className="text-xs text-muted-foreground">
          {t("auto_lessons_desc")}
        </p>
      </div>

      {error && <div className="text-sm text-destructive">{error}</div>}

      {config && (
        <AutoLessonsConfigForm config={config} onSave={saveConfig} />
      )}

      <div className="space-y-2">
        <h3 className="text-sm font-medium">
          {t("auto_lessons_history_title")}
        </h3>
        <AutoLessonsHistoryTable runs={runs} />
      </div>
    </div>
  );
};

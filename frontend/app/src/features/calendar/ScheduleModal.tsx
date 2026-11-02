import React, { useState } from "react";
import { t } from "../../core/i18n";
import type { CreateScheduleRequest, ScheduleDto, TaskTemplateDto } from "./types";
import { ScheduleFormFields } from "./ScheduleFormFields";

interface Props {
  initial?: ScheduleDto | null;
  projectId: string;
  onSave: (req: CreateScheduleRequest) => Promise<void>;
  onClose: () => void;
}

export const ScheduleModal: React.FC<Props> = ({ initial, projectId, onSave, onClose }) => {
  const [title, setTitle] = useState(initial?.title || "");
  const [scheduleType, setScheduleType] = useState<"cron" | "interval">(initial?.schedule_type || "cron");
  const [scheduleExpr, setScheduleExpr] = useState(initial?.schedule_expr || "0 * * * *");
  const [timezone, setTimezone] = useState(initial?.timezone || "UTC");
  const [autoDispatch, setAutoDispatch] = useState(initial?.auto_dispatch || false);
  const [missedPolicy] = useState<"skip" | "run_once">(initial?.missed_policy || "skip");
  const [template, setTemplate] = useState<TaskTemplateDto>(initial?.task_template || {
    title: "", description: "", lead_agent: "lead", worker_agent: "worker"
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await onSave({ project_id: projectId, title, schedule_type: scheduleType, schedule_expr: scheduleExpr, timezone, task_template: template, auto_dispatch: autoDispatch, missed_policy: missedPolicy });
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50 p-4">
      <form onSubmit={handleSubmit} className="bg-surface rounded-lg shadow-lg border p-5 w-full max-w-md space-y-4">
        <h3 className="text-base font-semibold text-text">{initial ? t("calendar_edit_schedule") : t("calendar_create_schedule")}</h3>
        <ScheduleFormFields title={title} setTitle={setTitle} scheduleType={scheduleType} setScheduleType={setScheduleType} scheduleExpr={scheduleExpr} setScheduleExpr={setScheduleExpr} timezone={timezone} setTimezone={setTimezone} template={template} setTemplate={setTemplate} />
        <div className="flex items-center space-x-2 text-xs text-text">
          <input type="checkbox" id="autoDispatch" checked={autoDispatch} onChange={(e) => setAutoDispatch(e.target.checked)} />
          <label htmlFor="autoDispatch">{t("calendar_auto_dispatch")}</label>
        </div>
        <div className="flex justify-end space-x-2 pt-2 border-t">
          <button type="button" onClick={onClose} className="px-3 py-1.5 text-xs text-text-subtle hover:bg-surface-subtle rounded">{t("calendar_cancel")}</button>
          <button type="submit" className="px-3 py-1.5 text-xs bg-primary text-white rounded hover:bg-primary-hover font-medium">{t("calendar_save")}</button>
        </div>
      </form>
    </div>
  );
};

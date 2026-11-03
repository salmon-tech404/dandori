import React, { useState } from "react";
import { t } from "../../core/i18n";
import { Modal } from "../../ui/components/Modal";
import { Button } from "../../ui/components/Button";
import { Checkbox } from "../../ui/components/Checkbox";
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
    <Modal
      open={true}
      onClose={onClose}
      title={initial ? t("calendar_edit_schedule") : t("calendar_create_schedule")}
      size="md"
      footer={
        <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--sp-8)" }}>
          <Button variant="ghost" onClick={onClose}>{t("calendar_cancel")}</Button>
          <Button variant="primary" onClick={handleSubmit}>{t("calendar_save")}</Button>
        </div>
      }
    >
      <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: "var(--sp-16)" }}>
        <ScheduleFormFields title={title} setTitle={setTitle} scheduleType={scheduleType} setScheduleType={setScheduleType} scheduleExpr={scheduleExpr} setScheduleExpr={setScheduleExpr} timezone={timezone} setTimezone={setTimezone} template={template} setTemplate={setTemplate} />
        <Checkbox id="autoDispatch" checked={autoDispatch} onChange={(e) => setAutoDispatch(e.target.checked)} label={t("calendar_auto_dispatch")} />
      </form>
    </Modal>
  );
};

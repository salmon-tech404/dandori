import React from "react";
import { t } from "../../core/i18n";
import type { TaskTemplateDto } from "./types";

interface Props {
  title: string;
  setTitle: (v: string) => void;
  scheduleType: "cron" | "interval";
  setScheduleType: (v: "cron" | "interval") => void;
  scheduleExpr: string;
  setScheduleExpr: (v: string) => void;
  timezone: string;
  setTimezone: (v: string) => void;
  template: TaskTemplateDto;
  setTemplate: React.Dispatch<React.SetStateAction<TaskTemplateDto>>;
}

export const ScheduleFormFields: React.FC<Props> = ({
  title, setTitle, scheduleType, setScheduleType, scheduleExpr,
  setScheduleExpr, timezone, setTimezone, template, setTemplate
}) => {
  return (
    <div className="space-y-3">
      <div>
        <label className="block text-xs text-text-subtle mb-1">{t("calendar_title")}</label>
        <input className="w-full text-sm border rounded px-2 py-1 bg-surface" value={title} onChange={(e) => setTitle(e.target.value)} required />
      </div>
      <div className="grid grid-cols-2 gap-2">
        <div>
          <label className="block text-xs text-text-subtle mb-1">{t("calendar_schedule_type")}</label>
          <select className="w-full text-sm border rounded px-2 py-1 bg-surface" value={scheduleType} onChange={(e) => setScheduleType(e.target.value as "cron" | "interval")}>
            <option value="cron">{t("calendar_cron")}</option>
            <option value="interval">{t("calendar_interval")}</option>
          </select>
        </div>
        <div>
          <label className="block text-xs text-text-subtle mb-1">{t("calendar_expression")}</label>
          <input className="w-full text-sm border rounded px-2 py-1 bg-surface" value={scheduleExpr} onChange={(e) => setScheduleExpr(e.target.value)} placeholder={scheduleType === "cron" ? "0 * * * *" : "3600"} required />
        </div>
      </div>
      <div>
        <label className="block text-xs text-text-subtle mb-1">{t("calendar_timezone")}</label>
        <input className="w-full text-sm border rounded px-2 py-1 bg-surface" value={timezone} onChange={(e) => setTimezone(e.target.value)} placeholder="UTC hoặc +07:00" />
      </div>
      <div>
        <label className="block text-xs text-text-subtle mb-1">{t("calendar_task_title")}</label>
        <input className="w-full text-sm border rounded px-2 py-1 bg-surface" value={template.title} onChange={(e) => setTemplate((prev) => ({ ...prev, title: e.target.value }))} required />
      </div>
    </div>
  );
};

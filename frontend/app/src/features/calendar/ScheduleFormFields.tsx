import React from "react";
import { t } from "../../core/i18n";
import { Input } from "../../ui/components/Input";
import { Select } from "../../ui/components/Select";
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
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)" }}>
      <div>
        <label style={{ display: "block", fontSize: "var(--fs-12)", color: "var(--mu)", marginBottom: 4 }}>{t("calendar_title")}</label>
        <Input value={title} onChange={(e) => setTitle(e.target.value)} required />
      </div>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--sp-8)" }}>
        <div>
          <label style={{ display: "block", fontSize: "var(--fs-12)", color: "var(--mu)", marginBottom: 4 }}>{t("calendar_schedule_type")}</label>
          <Select value={scheduleType} onChange={(e) => setScheduleType(e.target.value as "cron" | "interval")}>
            <option value="cron">{t("calendar_cron")}</option>
            <option value="interval">{t("calendar_interval")}</option>
          </Select>
        </div>
        <div>
          <label style={{ display: "block", fontSize: "var(--fs-12)", color: "var(--mu)", marginBottom: 4 }}>{t("calendar_expression")}</label>
          <Input value={scheduleExpr} onChange={(e) => setScheduleExpr(e.target.value)} placeholder={scheduleType === "cron" ? "0 * * * *" : "3600"} required />
        </div>
      </div>
      <div>
        <label style={{ display: "block", fontSize: "var(--fs-12)", color: "var(--mu)", marginBottom: 4 }}>{t("calendar_timezone")}</label>
        <Input value={timezone} onChange={(e) => setTimezone(e.target.value)} placeholder={t("calendar_tz_placeholder")} />
      </div>
      <div>
        <label style={{ display: "block", fontSize: "var(--fs-12)", color: "var(--mu)", marginBottom: 4 }}>{t("calendar_task_title")}</label>
        <Input value={template.title} onChange={(e) => setTemplate((prev) => ({ ...prev, title: e.target.value }))} required />
      </div>
    </div>
  );
};

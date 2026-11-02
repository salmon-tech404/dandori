import React from "react";
import { t } from "../../core/i18n";
import type { ScheduleDto } from "./types";

interface Props {
  schedule: ScheduleDto;
  onFireNow: (id: string) => Promise<void>;
  onTogglePause: (s: ScheduleDto) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
}

export const ScheduleCard: React.FC<Props> = ({ schedule, onFireNow, onTogglePause, onDelete }) => {
  return (
    <div className="bg-surface rounded border p-3 shadow-xs space-y-2">
      <div className="flex items-center justify-between">
        <span className="font-medium text-xs text-text">{schedule.title}</span>
        <span className={`text-[10px] px-1.5 py-0.5 rounded font-mono ${schedule.enabled ? "bg-green-100 text-green-700" : "bg-gray-100 text-gray-600"}`}>
          {schedule.enabled ? t("calendar_active") : t("calendar_paused")}
        </span>
      </div>
      <div className="text-[11px] text-text-subtle font-mono">{schedule.schedule_expr} ({schedule.timezone})</div>
      <div className="text-[11px] text-text-subtle">{t("calendar_next_fire")} {schedule.next_fire ? new Date(schedule.next_fire).toLocaleString() : "—"}</div>
      <div className="flex items-center justify-end space-x-1 pt-1 border-t text-xs">
        <button type="button" onClick={() => onFireNow(schedule.id)} className="px-2 py-0.5 text-[11px] bg-primary text-white rounded hover:bg-primary-hover">{t("calendar_fire_now")}</button>
        <button type="button" onClick={() => onTogglePause(schedule)} className="px-2 py-0.5 text-[11px] border rounded hover:bg-surface-subtle">{schedule.enabled ? t("calendar_pause") : t("calendar_resume")}</button>
        <button type="button" onClick={() => onDelete(schedule.id)} className="px-2 py-0.5 text-[11px] text-red-600 hover:bg-red-50 rounded">{t("calendar_delete_schedule")}</button>
      </div>
    </div>
  );
};

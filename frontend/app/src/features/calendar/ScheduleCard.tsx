import React from "react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import type { ScheduleDto } from "./types";

interface Props {
  schedule: ScheduleDto;
  onFireNow: (id: string) => Promise<void>;
  onTogglePause: (s: ScheduleDto) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
}

export const ScheduleCard: React.FC<Props> = ({ schedule, onFireNow, onTogglePause, onDelete }) => {
  return (
    <div style={{ backgroundColor: "var(--sf)", borderRadius: "var(--radius-6)", border: "1px solid var(--ln)", padding: "var(--sp-12)", display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <span style={{ fontWeight: 500, fontSize: "var(--fs-12)", color: "var(--fg)" }}>{schedule.title}</span>
        <span
          style={{
            fontSize: "10px",
            padding: "2px 6px",
            borderRadius: "var(--radius-4)",
            fontFamily: "var(--font-mono)",
            backgroundColor: schedule.enabled ? "var(--grb)" : "var(--gyb)",
            color: schedule.enabled ? "var(--gr)" : "var(--gy)",
          }}
        >
          {schedule.enabled ? t("calendar_active") : t("calendar_paused")}
        </span>
      </div>
      <div style={{ fontSize: "11px", color: "var(--mu)", fontFamily: "var(--font-mono)" }}>{schedule.schedule_expr} ({schedule.timezone})</div>
      <div style={{ fontSize: "11px", color: "var(--mu)" }}>{t("calendar_next_fire")} {schedule.next_fire ? new Date(schedule.next_fire).toLocaleString() : "—"}</div>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "flex-end", gap: "var(--sp-4)", paddingTop: "var(--sp-4)", borderTop: "1px solid var(--ln)", fontSize: "var(--fs-12)" }}>
        <Button variant="primary" size="sm" type="button" onClick={() => onFireNow(schedule.id)}>{t("calendar_fire_now")}</Button>
        <Button variant="secondary" size="sm" type="button" onClick={() => onTogglePause(schedule)}>{schedule.enabled ? t("calendar_pause") : t("calendar_resume")}</Button>
        <Button variant="danger" size="sm" type="button" onClick={() => onDelete(schedule.id)}>{t("calendar_delete_schedule")}</Button>
      </div>
    </div>
  );
};

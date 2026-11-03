import React, { useEffect, useState, useCallback } from "react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { fetchSchedules, fireScheduleNow, pauseSchedule, resumeSchedule, deleteSchedule } from "./api";
import { ScheduleCard } from "./ScheduleCard";
import type { ScheduleDto } from "./types";

const WEEKDAYS = [
  "calendar_mon",
  "calendar_tue",
  "calendar_wed",
  "calendar_thu",
  "calendar_fri",
  "calendar_sat",
  "calendar_sun",
] as const;

function formatScheduleTime(s: ScheduleDto): string {
  if (s.schedule_type === "cron") {
    const parts = s.schedule_expr.trim().split(/\s+/);
    if (parts.length >= 2) {
      const min = parts[0].padStart(2, "0");
      const hour = parts[1].padStart(2, "0");
      return `${hour}:${min}`;
    }
  }
  return "06:30";
}

function ScheduleEventCard({
  schedule,
  onFire,
  onToggle,
}: {
  schedule: ScheduleDto;
  onFire: (id: string) => Promise<void>;
  onToggle: (s: ScheduleDto) => Promise<void>;
}) {
  return (
    <div
      className="ev"
      style={{
        display: "flex",
        flexDirection: "column",
        gap: "4px",
        padding: "8px 10px",
        borderRadius: "var(--radius-6)",
        backgroundColor: "var(--sf)",
        border: "1px solid var(--ln)",
        fontSize: "var(--fs-12)",
        wordBreak: "break-word",
      }}
    >
      <span style={{ fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--bl)", fontWeight: 600 }}>
        {formatScheduleTime(schedule)}
      </span>
      <span style={{ fontWeight: 600, color: "var(--fg)" }}>{schedule.title}</span>
      <div style={{ display: "flex", gap: "4px", marginTop: "4px" }}>
        <Button variant="ghost" size="sm" type="button" onClick={() => void onFire(schedule.id)} style={{ padding: "2px 6px", height: "auto" }}>
          {t("calendar_fire_now")}
        </Button>
        <Button variant="ghost" size="sm" type="button" onClick={() => void onToggle(schedule)} style={{ padding: "2px 6px", height: "auto" }}>
          {schedule.enabled ? t("calendar_pause") : t("calendar_resume")}
        </Button>
      </div>
    </div>
  );
}

function DayColumn({
  dayKey,
  daySchedules,
  onFire,
  onToggle,
}: {
  dayKey: (typeof WEEKDAYS)[number];
  daySchedules: ScheduleDto[];
  onFire: (id: string) => Promise<void>;
  onToggle: (s: ScheduleDto) => Promise<void>;
}) {
  return (
    <div
      className="day"
      style={{
        display: "flex",
        flexDirection: "column",
        gap: "var(--sp-8)",
        minHeight: "220px",
        padding: "10px",
        borderRadius: "var(--radius-8)",
        backgroundColor: "var(--sf2)",
        minWidth: "130px",
      }}
    >
      <b style={{ fontSize: "var(--fs-13)", fontWeight: 700, color: "var(--fg)" }}>{t(dayKey)}</b>
      {daySchedules.length === 0 ? (
        <span style={{ fontSize: "var(--fs-12)", color: "var(--mu)", marginTop: "auto" }}>—</span>
      ) : (
        daySchedules.map((s) => (
          <ScheduleEventCard key={s.id} schedule={s} onFire={onFire} onToggle={onToggle} />
        ))
      )}
    </div>
  );
}

function useCalendarActions(projectId: string) {
  const [schedules, setSchedules] = useState<ScheduleDto[]>([]);

  const load = useCallback(async () => {
    try {
      setSchedules(await fetchSchedules(projectId));
    } catch {
      setSchedules([]);
    }
  }, [projectId]);

  useEffect(() => {
    void load();
  }, [load]);

  const handleToggle = async (s: ScheduleDto) => {
    if (s.enabled) await pauseSchedule(s.id);
    else await resumeSchedule(s.id);
    await load();
  };
  const handleDelete = async (id: string) => {
    await deleteSchedule(id);
    await load();
  };
  const handleFire = async (id: string) => {
    await fireScheduleNow(id);
    await load();
  };

  return { schedules, handleToggle, handleDelete, handleFire };
}

export const CalendarGrid: React.FC<{ projectId?: string }> = ({ projectId = "default-proj" }) => {
  const { schedules, handleToggle, handleDelete, handleFire } = useCalendarActions(projectId);

  return (
    <div style={{ padding: "var(--sp-16) 0", display: "flex", flexDirection: "column", gap: "var(--sp-16)" }}>
      <div
        className="cal"
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(7, minmax(130px, 1fr))",
          gap: "8px",
          overflowX: "auto",
        }}
      >
        {WEEKDAYS.map((dayKey, idx) => (
          <DayColumn
            key={dayKey}
            dayKey={dayKey}
            daySchedules={schedules.filter((_, sIdx) => sIdx % 7 === idx)}
            onFire={handleFire}
            onToggle={handleToggle}
          />
        ))}
      </div>
      {schedules.length === 0 ? (
        <div style={{ textAlign: "center", padding: "var(--sp-16) 0", fontSize: "var(--fs-12)", color: "var(--mu)" }}>
          {t("calendar_no_schedules")}
        </div>
      ) : (
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))", gap: "var(--sp-12)", marginTop: "var(--sp-12)" }}>
          {schedules.map((s) => (
            <ScheduleCard key={s.id} schedule={s} onFireNow={handleFire} onTogglePause={handleToggle} onDelete={handleDelete} />
          ))}
        </div>
      )}
    </div>
  );
};

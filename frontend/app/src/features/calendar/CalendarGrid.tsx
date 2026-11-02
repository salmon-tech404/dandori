import React, { useEffect, useState, useCallback } from "react";
import { t } from "../../core/i18n";
import { fetchSchedules, fireScheduleNow, pauseSchedule, resumeSchedule, deleteSchedule } from "./api";
import { ScheduleCard } from "./ScheduleCard";
import type { ScheduleDto } from "./types";

const WEEKDAYS = ["calendar_mon", "calendar_tue", "calendar_wed", "calendar_thu", "calendar_fri", "calendar_sat", "calendar_sun"] as const;

export const CalendarGrid: React.FC<{ projectId?: string }> = ({ projectId = "default-proj" }) => {
  const [schedules, setSchedules] = useState<ScheduleDto[]>([]);

  const load = useCallback(async () => {
    try { setSchedules(await fetchSchedules(projectId)); } catch { setSchedules([]); }
  }, [projectId]);

  useEffect(() => { void load(); }, [load]);

  const handleToggle = async (s: ScheduleDto) => {
    if (s.enabled) { await pauseSchedule(s.id); } else { await resumeSchedule(s.id); }
    await load();
  };
  const handleDelete = async (id: string) => { await deleteSchedule(id); await load(); };
  const handleFire = async (id: string) => { await fireScheduleNow(id); await load(); };

  return (
    <div className="p-4 space-y-4">
      <div className="grid grid-cols-7 gap-2 border-b pb-2">
        {WEEKDAYS.map((dayKey) => (
          <div key={dayKey} className="text-center text-xs font-semibold text-text-subtle">{t(dayKey)}</div>
        ))}
      </div>
      {schedules.length === 0 ? (
        <div className="text-center py-8 text-xs text-text-subtle">{t("calendar_no_schedules")}</div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
          {schedules.map((s) => (
            <ScheduleCard key={s.id} schedule={s} onFireNow={handleFire} onTogglePause={handleToggle} onDelete={handleDelete} />
          ))}
        </div>
      )}
    </div>
  );
};

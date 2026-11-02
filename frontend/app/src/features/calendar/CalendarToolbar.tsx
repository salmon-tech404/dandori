import React, { useState } from "react";
import { t } from "../../core/i18n";
import { ScheduleModal } from "./ScheduleModal";
import { createSchedule } from "./api";
import type { CreateScheduleRequest } from "./types";

interface Props {
  projectId?: string;
  onRefresh?: () => void;
}

export const CalendarToolbar: React.FC<Props> = ({ projectId = "default-proj", onRefresh }) => {
  const [modalOpen, setModalOpen] = useState(false);

  const handleSave = async (req: CreateScheduleRequest) => {
    await createSchedule(req);
    onRefresh?.();
  };

  return (
    <div className="flex items-center justify-between py-2 px-4 border-b bg-surface">
      <div className="flex items-center space-x-3">
        <h2 className="text-sm font-semibold text-text">{t("calendar_week_title")}</h2>
      </div>
      <div>
        <button
          type="button"
          onClick={() => setModalOpen(true)}
          className="px-3 py-1.5 text-xs bg-primary text-white rounded hover:bg-primary-hover font-medium cursor-pointer"
        >
          {t("calendar_create_schedule")}
        </button>
      </div>
      {modalOpen && (
        <ScheduleModal projectId={projectId} onSave={handleSave} onClose={() => setModalOpen(false)} />
      )}
    </div>
  );
};

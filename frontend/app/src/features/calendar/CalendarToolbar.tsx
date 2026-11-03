import React, { useState } from "react";
import { Plus } from "lucide-react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { Chip } from "../../ui/components/Chip";
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
    <div style={{ display: "flex", flexWrap: "wrap", alignItems: "center", justifyContent: "space-between", gap: "var(--sp-12)", padding: "var(--sp-12) var(--sp-16)", borderBottom: "1px solid var(--ln)", backgroundColor: "var(--sf)", borderRadius: "var(--radius-8)" }}>
      <div style={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: "var(--sp-12)" }}>
        <Button
          variant="primary"
          size="sm"
          icon={<Plus size={14} />}
          type="button"
          onClick={() => setModalOpen(true)}
        >
          {t("calendar_create_schedule")}
        </Button>
        <Chip variant="default">{t("calendar_week_chip")}</Chip>
        <Chip variant="orange">{t("calendar_quiet_chip")}</Chip>
      </div>
      {modalOpen && (
        <ScheduleModal projectId={projectId} onSave={handleSave} onClose={() => setModalOpen(false)} />
      )}
    </div>
  );
};

import React, { useState } from "react";
import { t } from "../../core/i18n";
import type { CreateQuietWindowRequest, QuietWindow } from "./types";

interface Props {
  initial?: QuietWindow | null;
  workspaceId: string;
  onSave: (req: CreateQuietWindowRequest) => void;
  onClose: () => void;
}

interface FormState {
  name: string;
  startTime: string;
  endTime: string;
  days: string;
  tz: string;
  blockManual: boolean;
}

const BasicInputs: React.FC<{
  name: string;
  startTime: string;
  endTime: string;
  onChange: (patch: Partial<FormState>) => void;
}> = ({ name, startTime, endTime, onChange }) => (
  <>
    <div>
      <label className="block mb-1 font-medium">{t("quiet_hours_window_name")}</label>
      <input
        type="text"
        required
        value={name}
        onChange={(e) => onChange({ name: e.target.value })}
        className="w-full border rounded px-2.5 py-1.5 bg-transparent"
        placeholder="VD: Khung giờ cấm ban ngày"
      />
    </div>
    <div className="grid grid-cols-2 gap-2">
      <div>
        <label className="block mb-1 font-medium">{t("quiet_hours_start_time")}</label>
        <input
          type="text"
          required
          value={startTime}
          onChange={(e) => onChange({ startTime: e.target.value })}
          className="w-full border rounded px-2.5 py-1.5 bg-transparent"
          placeholder="09:00"
        />
      </div>
      <div>
        <label className="block mb-1 font-medium">{t("quiet_hours_end_time")}</label>
        <input
          type="text"
          required
          value={endTime}
          onChange={(e) => onChange({ endTime: e.target.value })}
          className="w-full border rounded px-2.5 py-1.5 bg-transparent"
          placeholder="18:00"
        />
      </div>
    </div>
  </>
);

const ScheduleInputs: React.FC<{
  days: string;
  blockManual: boolean;
  onChange: (patch: Partial<FormState>) => void;
}> = ({ days, blockManual, onChange }) => (
  <>
    <div>
      <label className="block mb-1 font-medium">{t("quiet_hours_days")}</label>
      <input
        type="text"
        required
        value={days}
        onChange={(e) => onChange({ days: e.target.value })}
        className="w-full border rounded px-2.5 py-1.5 bg-transparent"
        placeholder="1,2,3,4,5"
      />
    </div>
    <div className="flex items-center gap-2 pt-1">
      <input
        type="checkbox"
        id="blockManual"
        checked={blockManual}
        onChange={(e) => onChange({ blockManual: e.target.checked })}
        className="rounded"
      />
      <label htmlFor="blockManual" className="cursor-pointer">{t("quiet_hours_block_manual")}</label>
    </div>
  </>
);

const ModalActions: React.FC<{ onClose: () => void }> = ({ onClose }) => (
  <div className="flex justify-end gap-2 pt-3 border-t">
    <button type="button" onClick={onClose} className="px-3 py-1.5 border rounded">
      {t("quiet_hours_cancel")}
    </button>
    <button type="submit" className="px-3 py-1.5 bg-blue-600 text-white rounded font-medium">
      {t("quiet_hours_save")}
    </button>
  </div>
);

function getInitialForm(initial?: QuietWindow | null): FormState {
  return {
    name: initial?.name || "",
    startTime: initial?.start_time || "09:00",
    endTime: initial?.end_time || "18:00",
    days: initial?.days_of_week || "1,2,3,4,5",
    tz: initial?.timezone || "UTC",
    blockManual: initial?.block_manual_runs || false,
  };
}

export const QuietWindowFormModal: React.FC<Props> = ({
  initial,
  workspaceId,
  onSave,
  onClose,
}) => {
  const [form, setForm] = useState<FormState>(() => getInitialForm(initial));

  const onSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave({
      workspace_id: workspaceId,
      name: form.name,
      start_time: form.startTime,
      end_time: form.endTime,
      days_of_week: form.days,
      timezone: form.tz,
      block_manual_runs: form.blockManual,
    });
  };

  const title = initial ? t("quiet_hours_edit_window") : t("quiet_hours_add_window");
  return (
    <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50 p-4">
      <div className="bg-white dark:bg-slate-900 border rounded-lg p-5 max-w-md w-full shadow-lg">
        <h3 className="text-base font-semibold mb-3">{title}</h3>
        <form onSubmit={onSubmit} className="space-y-3 text-sm">
          <BasicInputs name={form.name} startTime={form.startTime} endTime={form.endTime} onChange={(p) => setForm((s) => ({ ...s, ...p }))} />
          <ScheduleInputs days={form.days} blockManual={form.blockManual} onChange={(p) => setForm((s) => ({ ...s, ...p }))} />
          <ModalActions onClose={onClose} />
        </form>
      </div>
    </div>
  );
};

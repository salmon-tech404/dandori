import React, { useState } from "react";
import { t } from "../../core/i18n";
import { Modal } from "../../ui/components/Modal";
import { Button } from "../../ui/components/Button";
import { Input } from "../../ui/components/Input";
import { Checkbox } from "../../ui/components/Checkbox";
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
      <label style={{ display: "block", marginBottom: 4, fontWeight: 500, fontSize: "var(--fs-12)", color: "var(--fg)" }}>{t("quiet_hours_window_name")}</label>
      <Input type="text"
        required
        value={name}
        onChange={(e) => onChange({ name: e.target.value })}
        placeholder={t("quiet_hours_name_placeholder")}
      />
    </div>
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--sp-8)" }}>
      <div>
        <label style={{ display: "block", marginBottom: 4, fontWeight: 500, fontSize: "var(--fs-12)", color: "var(--fg)" }}>{t("quiet_hours_start_time")}</label>
        <Input type="text"
          required
          value={startTime}
          onChange={(e) => onChange({ startTime: e.target.value })}
          placeholder="09:00"
        />
      </div>
      <div>
        <label style={{ display: "block", marginBottom: 4, fontWeight: 500, fontSize: "var(--fs-12)", color: "var(--fg)" }}>{t("quiet_hours_end_time")}</label>
        <Input type="text"
          required
          value={endTime}
          onChange={(e) => onChange({ endTime: e.target.value })}
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
      <label style={{ display: "block", marginBottom: 4, fontWeight: 500, fontSize: "var(--fs-12)", color: "var(--fg)" }}>{t("quiet_hours_days")}</label>
      <Input type="text"
        required
        value={days}
        onChange={(e) => onChange({ days: e.target.value })}
        placeholder="1,2,3,4,5"
      />
    </div>
    <Checkbox id="blockManual" checked={blockManual} onChange={(e) => onChange({ blockManual: e.target.checked })} label={t("quiet_hours_block_manual")} />
  </>
);

const ModalActions: React.FC<{ onClose: () => void }> = ({ onClose }) => (
  <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--sp-8)", paddingTop: "var(--sp-12)", borderTop: "1px solid var(--ln)" }}>
    <Button variant="secondary" type="button" onClick={onClose}>{t("quiet_hours_cancel")}</Button>
    <Button variant="primary" type="submit">{t("quiet_hours_save")}</Button>
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
    <Modal open onClose={onClose} title={title} size="md">
      <form onSubmit={onSubmit} style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)", fontSize: "var(--fs-14)" }}>
        <BasicInputs name={form.name} startTime={form.startTime} endTime={form.endTime} onChange={(p) => setForm((s) => ({ ...s, ...p }))} />
        <ScheduleInputs days={form.days} blockManual={form.blockManual} onChange={(p) => setForm((s) => ({ ...s, ...p }))} />
        <ModalActions onClose={onClose} />
      </form>
    </Modal>
  );
};

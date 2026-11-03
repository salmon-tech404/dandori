import React, { useState } from "react";
import { Plus } from "lucide-react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { Checkbox } from "../../ui/components/Checkbox";
import { QuietWindowFormModal } from "./QuietWindowFormModal";
import type { QuietWindow } from "./types";
import { useQuietHours } from "./useQuietHours";

interface Props {
  workspaceId?: string;
}

const WindowRow: React.FC<{
  item: QuietWindow;
  onToggle: (id: string) => void;
  onDelete: (id: string) => void;
}> = ({ item, onToggle, onDelete }) => (
  <tr style={{ borderBottom: "1px solid var(--ln)" }}>
    <td style={{ padding: "10px 12px", fontWeight: 500, color: "var(--fg)" }}>{item.name}</td>
    <td style={{ padding: "10px 12px", fontFamily: "var(--font-mono)", fontSize: "var(--fs-12)", color: "var(--fg)" }}>{item.start_time} - {item.end_time}</td>
    <td style={{ padding: "10px 12px", fontSize: "var(--fs-12)", color: "var(--mu)" }}>{item.days_of_week}</td>
    <td style={{ padding: "10px 12px", fontSize: "var(--fs-12)", color: "var(--mu)" }}>{item.timezone}</td>
    <td style={{ padding: "10px 12px", textAlign: "center" }}>
      <Checkbox checked={item.enabled} onChange={() => onToggle(item.id)} aria-label={`${t("quiet_hours_col_enabled")} ${item.name}`} />
    </td>
    <td style={{ padding: "10px 12px", textAlign: "right" }}>
      <Button variant="danger" size="sm" type="button" onClick={() => onDelete(item.id)}>{t("quiet_hours_delete_window")}</Button>
    </td>
  </tr>
);

const WindowsTable: React.FC<{
  windows: QuietWindow[];
  onToggle: (id: string) => void;
  onDelete: (id: string) => void;
}> = ({ windows, onToggle, onDelete }) => (
  <table style={{ width: "100%", fontSize: "var(--fs-14)", textAlign: "left", border: "1px solid var(--ln)", borderRadius: "var(--radius-6)", borderCollapse: "collapse" }}>
    <thead style={{ backgroundColor: "var(--sf2)", borderBottom: "1px solid var(--ln)", fontSize: "var(--fs-12)", fontWeight: 600, color: "var(--mu)" }}>
      <tr>
        <th style={{ padding: "8px 12px" }}>{t("quiet_hours_col_name")}</th>
        <th style={{ padding: "8px 12px" }}>{t("quiet_hours_col_time")}</th>
        <th style={{ padding: "8px 12px" }}>{t("quiet_hours_col_days")}</th>
        <th style={{ padding: "8px 12px" }}>{t("quiet_hours_col_timezone")}</th>
        <th style={{ padding: "8px 12px", textAlign: "center" }}>{t("quiet_hours_col_enabled")}</th>
        <th style={{ padding: "8px 12px", textAlign: "right" }}>{t("quiet_hours_col_actions")}</th>
      </tr>
    </thead>
    <tbody>
      {windows.map((w) => (
        <WindowRow key={w.id} item={w} onToggle={onToggle} onDelete={onDelete} />
      ))}
    </tbody>
  </table>
);

export const QuietHoursSettingsPanel: React.FC<Props> = ({
  workspaceId = "default",
}) => {
  const { windows, loading, handleCreate, handleToggle, handleDelete } =
    useQuietHours(workspaceId);
  const [modalOpen, setModalOpen] = useState(false);

  return (
    <div style={{ padding: "var(--sp-16)", display: "flex", flexDirection: "column", gap: "var(--sp-16)" }} data-testid="quiet-hours-settings-panel">
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <h2 style={{ margin: 0, fontSize: "var(--fs-18)", fontWeight: 600, color: "var(--fg)" }}>{t("quiet_hours_title")}</h2>
        <Button variant="primary" size="sm" icon={<Plus size={14} />} type="button" onClick={() => setModalOpen(true)}>{t("quiet_hours_add_window")}</Button>
      </div>
      {loading ? (
        <div style={{ fontSize: "var(--fs-14)", color: "var(--mu)" }}>{t("quiet_hours_loading")}</div>
      ) : windows.length === 0 ? (
        <div style={{ fontSize: "var(--fs-14)", color: "var(--mu)", padding: "var(--sp-16) 0", border: "1px dashed var(--ln)", borderRadius: "var(--radius-6)", textAlign: "center" }}>
          {t("quiet_hours_no_windows")}
        </div>
      ) : (
        <WindowsTable windows={windows} onToggle={handleToggle} onDelete={handleDelete} />
      )}
      {modalOpen && (
        <QuietWindowFormModal
          workspaceId={workspaceId}
          onSave={async (req) => {
            await handleCreate(req);
            setModalOpen(false);
          }}
          onClose={() => setModalOpen(false)}
        />
      )}
    </div>
  );
};

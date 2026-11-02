import React, { useState } from "react";
import { t } from "../../core/i18n";
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
  <tr className="border-b hover:bg-slate-50/50 dark:hover:bg-slate-800/50">
    <td className="py-2.5 px-3 font-medium">{item.name}</td>
    <td className="py-2.5 px-3 font-mono text-xs">{item.start_time} - {item.end_time}</td>
    <td className="py-2.5 px-3 text-xs">{item.days_of_week}</td>
    <td className="py-2.5 px-3 text-xs">{item.timezone}</td>
    <td className="py-2.5 px-3 text-center">
      <input
        type="checkbox"
        checked={item.enabled}
        onChange={() => onToggle(item.id)}
        aria-label={`${t("quiet_hours_col_enabled")} ${item.name}`}
      />
    </td>
    <td className="py-2.5 px-3 text-right">
      <button
        type="button"
        onClick={() => onDelete(item.id)}
        className="text-red-500 hover:text-red-700 text-xs"
      >
        {t("quiet_hours_delete_window")}
      </button>
    </td>
  </tr>
);

const WindowsTable: React.FC<{
  windows: QuietWindow[];
  onToggle: (id: string) => void;
  onDelete: (id: string) => void;
}> = ({ windows, onToggle, onDelete }) => (
  <table className="w-full text-sm text-left border rounded">
    <thead className="bg-slate-50 dark:bg-slate-800 border-b text-xs font-semibold">
      <tr>
        <th className="py-2 px-3">{t("quiet_hours_col_name")}</th>
        <th className="py-2 px-3">{t("quiet_hours_col_time")}</th>
        <th className="py-2 px-3">{t("quiet_hours_col_days")}</th>
        <th className="py-2 px-3">{t("quiet_hours_col_timezone")}</th>
        <th className="py-2 px-3 text-center">{t("quiet_hours_col_enabled")}</th>
        <th className="py-2 px-3 text-right">{t("quiet_hours_col_actions")}</th>
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
    <div className="p-4 space-y-4" data-testid="quiet-hours-settings-panel">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">{t("quiet_hours_title")}</h2>
        <button
          type="button"
          onClick={() => setModalOpen(true)}
          className="px-3 py-1.5 bg-blue-600 text-white text-xs rounded hover:bg-blue-700 font-medium"
        >
          {t("quiet_hours_add_window")}
        </button>
      </div>
      {loading ? (
        <div className="text-sm text-slate-500">{t("quiet_hours_loading")}</div>
      ) : windows.length === 0 ? (
        <div className="text-sm text-slate-500 py-4 border border-dashed rounded text-center">
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

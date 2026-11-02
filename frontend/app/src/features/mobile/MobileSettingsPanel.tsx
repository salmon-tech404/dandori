import { useCallback, useEffect, useState } from "react";
import { t } from "../../core/i18n";
import {
  createDevice,
  deleteDevice,
  fetchDevices,
  fetchMobileConfig,
  revokeDevice,
  updateMobileConfig,
} from "./api";
import type { CreateDeviceResponse, PublicMobileConfig, PublicMobileDevice } from "./types";

function DeviceActionsCell({
  d,
  onRevoke,
  onDelete,
}: {
  d: PublicMobileDevice;
  onRevoke: (id: string) => void;
  onDelete: (id: string) => void;
}) {
  return (
    <td className="flex gap-2 px-3 py-2">
      {!d.is_revoked && (
        <button
          onClick={() => onRevoke(d.id)}
          className="rounded bg-amber-950 px-2 py-1 text-xs text-amber-300 hover:bg-amber-900"
        >
          {t("mobile_revoke_btn")}
        </button>
      )}
      <button
        onClick={() => onDelete(d.id)}
        className="rounded bg-rose-950 px-2 py-1 text-xs text-rose-300 hover:bg-rose-900"
      >
        {t("mobile_delete_btn")}
      </button>
    </td>
  );
}

function DeviceRow({
  d,
  onRevoke,
  onDelete,
}: {
  d: PublicMobileDevice;
  onRevoke: (id: string) => void;
  onDelete: (id: string) => void;
}) {
  return (
    <tr className="hover:bg-neutral-800/30">
      <td className="px-3 py-2 font-medium text-neutral-200">{d.name}</td>
      <td className="px-3 py-2">
        <span
          className={`rounded px-1.5 py-0.5 text-[10px] font-semibold ${
            d.is_revoked ? "bg-rose-950 text-rose-400" : "bg-emerald-950 text-emerald-400"
          }`}
        >
          {d.is_revoked ? t("mobile_status_revoked") : t("mobile_status_active")}
        </span>
      </td>
      <td className="px-3 py-2 text-neutral-400">{d.read_only ? t("mobile_yes") : t("mobile_no")}</td>
      <td className="px-3 py-2 text-neutral-400">
        {d.last_used_at ? new Date(d.last_used_at).toLocaleDateString() : t("mobile_never")}
      </td>
      <DeviceActionsCell d={d} onRevoke={onRevoke} onDelete={onDelete} />
    </tr>
  );
}

function ModalActions({ onClose }: { onClose: () => void }) {
  return (
    <div className="flex justify-end gap-2">
      <button
        type="button"
        onClick={onClose}
        className="rounded px-3 py-1.5 text-xs text-neutral-400 hover:bg-neutral-800"
      >
        {t("mobile_cancel_btn")}
      </button>
      <button
        type="submit"
        className="rounded bg-indigo-600 px-3 py-1.5 text-xs text-white hover:bg-indigo-500"
      >
        {t("mobile_save_btn")}
      </button>
    </div>
  );
}

function makeDeviceDraft(name: string, ro: boolean): CreateDeviceResponse {
  return {
    device: {
      id: "",
      workspace_id: "",
      name: name.trim(),
      created_by: "",
      read_only: ro,
      is_revoked: false,
      expires_at: "",
      last_used_at: null,
      created_at: "",
    },
    token: "",
  };
}

function CreateDeviceModal({
  onClose,
  onCreated,
}: {
  onClose: () => void;
  onCreated: (res: CreateDeviceResponse) => void;
}) {
  const [name, setName] = useState("");
  const [ro, setRo] = useState(false);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (name.trim()) {
      onCreated(makeDeviceDraft(name, ro));
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4">
      <form onSubmit={handleSubmit} className="w-full max-w-sm rounded-lg bg-neutral-900 p-6 shadow-xl">
        <h3 className="mb-4 text-base font-semibold text-neutral-100">{t("mobile_add_device_title")}</h3>
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder={t("mobile_device_name_placeholder")}
          className="mb-3 w-full rounded border border-neutral-700 bg-neutral-800 px-3 py-2 text-sm text-neutral-100"
          required
        />
        <label className="mb-4 flex items-center gap-2 text-xs text-neutral-300">
          <input type="checkbox" checked={ro} onChange={(e) => setRo(e.target.checked)} />
          {t("mobile_read_only_opt")}
        </label>
        <ModalActions onClose={onClose} />
      </form>
    </div>
  );
}

function TokenDisplayModal({ res, onClose }: { res: CreateDeviceResponse; onClose: () => void }) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4">
      <div className="w-full max-w-md rounded-lg bg-neutral-900 p-6 shadow-xl">
        <h3 className="mb-2 text-base font-semibold text-emerald-400">{t("mobile_pair_success_title")}</h3>
        <p className="mb-4 text-xs text-neutral-300">{t("mobile_pair_success_desc")}</p>
        <div className="mb-4 rounded bg-neutral-950 p-3 font-mono text-xs text-amber-300 break-all">
          {res.token}
        </div>
        <div className="flex justify-end">
          <button
            onClick={onClose}
            className="rounded bg-indigo-600 px-4 py-2 text-xs text-white hover:bg-indigo-500"
          >
            {t("mobile_done_btn")}
          </button>
        </div>
      </div>
    </div>
  );
}

function useMobileDevices(ws: string) {
  const [devices, setDevices] = useState<PublicMobileDevice[]>([]);
  const [cfg, setCfg] = useState<PublicMobileConfig | null>(null);

  const reload = useCallback(async () => {
    const [d, c] = await Promise.all([fetchDevices(ws), fetchMobileConfig(ws)]);
    setDevices(d);
    setCfg(c);
  }, [ws]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const toggleRo = async () => {
    if (!cfg) return;
    const updated = await updateMobileConfig(ws, { read_only: !cfg.read_only });
    setCfg(updated);
  };

  return { devices, cfg, reload, toggleRo };
}

function DevicesTable({
  devices,
  ws,
  onReload,
}: {
  devices: PublicMobileDevice[];
  ws: string;
  onReload: () => void;
}) {
  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left text-xs text-neutral-300">
        <thead className="border-b border-neutral-800 bg-neutral-950 text-neutral-400">
          <tr>
            <th className="px-3 py-2">{t("mobile_col_device")}</th>
            <th className="px-3 py-2">{t("mobile_col_status")}</th>
            <th className="px-3 py-2">{t("mobile_col_read_only")}</th>
            <th className="px-3 py-2">{t("mobile_col_last_used")}</th>
            <th className="px-3 py-2">{t("mobile_col_actions")}</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-neutral-800">
          {devices.map((d) => (
            <DeviceRow
              key={d.id}
              d={d}
              onRevoke={async (id) => { await revokeDevice(ws, id); onReload(); }}
              onDelete={async (id) => { await deleteDevice(ws, id); onReload(); }}
            />
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function MobileSettingsPanel({ workspaceId }: { workspaceId?: string }) {
  const ws = workspaceId || "default";
  const { devices, cfg, reload, toggleRo } = useMobileDevices(ws);
  const [modalOpen, setModalOpen] = useState(false);
  const [newPair, setNewPair] = useState<CreateDeviceResponse | null>(null);

  const handleCreate = async (draft: CreateDeviceResponse) => {
    setModalOpen(false);
    const created = await createDevice(ws, {
      name: draft.device.name,
      read_only: draft.device.read_only,
    });
    setNewPair(created);
    void reload();
  };

  return (
    <div className="rounded-lg border border-neutral-800 bg-neutral-900 p-6">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h2 className="text-lg font-bold text-neutral-100">{t("mobile_settings_title")}</h2>
          <p className="text-xs text-neutral-400">{t("mobile_settings_desc")}</p>
        </div>
        <button
          onClick={() => setModalOpen(true)}
          className="rounded bg-indigo-600 px-3 py-1.5 text-xs text-white hover:bg-indigo-500"
        >
          {t("mobile_add_device_btn")}
        </button>
      </div>

      <div className="mb-6 flex items-center gap-3 rounded bg-neutral-950 p-3">
        <input type="checkbox" id="ro-toggle" checked={cfg?.read_only || false} onChange={toggleRo} />
        <label htmlFor="ro-toggle" className="text-xs text-neutral-300">
          {t("mobile_workspace_read_only")}
        </label>
      </div>

      <DevicesTable devices={devices} ws={ws} onReload={reload} />
      {modalOpen && <CreateDeviceModal onClose={() => setModalOpen(false)} onCreated={handleCreate} />}
      {newPair && <TokenDisplayModal res={newPair} onClose={() => setNewPair(null)} />}
    </div>
  );
}

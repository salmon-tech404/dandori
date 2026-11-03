import { useCallback, useEffect, useState } from "react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { Modal } from "../../ui/components/Modal";
import { Input } from "../../ui/components/Input";
import { Checkbox } from "../../ui/components/Checkbox";
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
    <td style={{ padding: "var(--sp-8) var(--sp-12)", display: "flex", gap: "var(--sp-8)" }}>
      {!d.is_revoked && (
        <Button
          variant="secondary"
          size="sm"
          onClick={() => onRevoke(d.id)}
          style={{ color: "var(--or)" }}
        >
          {t("mobile_revoke_btn")}
        </Button>
      )}
      <Button
        variant="ghost"
        size="sm"
        onClick={() => onDelete(d.id)}
        style={{ color: "var(--rd)" }}
      >
        {t("mobile_delete_btn")}
      </Button>
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
    <tr style={{ borderBottom: "1px solid var(--ln)", color: "var(--fg)" }}>
      <td style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 500 }}>{d.name}</td>
      <td style={{ padding: "var(--sp-8) var(--sp-12)" }}>
        <span
          style={{
            padding: "2px 8px",
            borderRadius: "var(--radius-pill)",
            fontSize: "var(--fs-12)",
            fontWeight: 600,
            backgroundColor: d.is_revoked ? "var(--rdb)" : "var(--grb)",
            color: d.is_revoked ? "var(--rd)" : "var(--gr)",
          }}
        >
          {d.is_revoked ? t("mobile_status_revoked") : t("mobile_status_active")}
        </span>
      </td>
      <td style={{ padding: "var(--sp-8) var(--sp-12)", color: "var(--mu)" }}>{d.read_only ? t("mobile_yes") : t("mobile_no")}</td>
      <td style={{ padding: "var(--sp-8) var(--sp-12)", color: "var(--mu)" }}>
        {d.last_used_at ? new Date(d.last_used_at).toLocaleDateString() : t("mobile_never")}
      </td>
      <DeviceActionsCell d={d} onRevoke={onRevoke} onDelete={onDelete} />
    </tr>
  );
}

function ModalActions({ onClose }: { onClose: () => void }) {
  return (
    <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--sp-8)", marginTop: "var(--sp-16)" }}>
      <Button type="button" variant="secondary" size="sm" onClick={onClose}>
        {t("mobile_cancel_btn")}
      </Button>
      <Button type="submit" variant="primary" size="sm">
        {t("mobile_save_btn")}
      </Button>
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
    <Modal open onClose={onClose} title={t("mobile_add_device_title")} size="sm">
      <form onSubmit={handleSubmit}>
        <Input type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder={t("mobile_device_name_placeholder")}
          style={{ width: "100%", height: "var(--btn-height)", padding: "0 12px", fontSize: "var(--fs-13)", fontFamily: "var(--font-sans)", color: "var(--fg)", backgroundColor: "var(--sf2)", border: "1px solid var(--ln)", borderRadius: "var(--radius-8)", outline: "none", boxSizing: "border-box", marginBottom: "var(--sp-12)" }}
          required
        />
        <div style={{ marginBottom: "var(--sp-16)" }}><Checkbox checked={ro} onChange={(e) => setRo(e.target.checked)} label={t("mobile_read_only_opt")} /></div>
        <ModalActions onClose={onClose} />
      </form>
    </Modal>
  );
}

function TokenDisplayModal({ res, onClose }: { res: CreateDeviceResponse; onClose: () => void }) {
  const footer = (
    <Button variant="primary" size="sm" onClick={onClose}>
      {t("mobile_done_btn")}
    </Button>
  );

  return (
    <Modal open onClose={onClose} title={t("mobile_pair_success_title")} footer={footer} size="sm">
      <p style={{ margin: "0 0 var(--sp-16) 0", fontSize: "var(--fs-13)", color: "var(--mu)" }}>{t("mobile_pair_success_desc")}</p>
      <div style={{ padding: "var(--sp-12)", borderRadius: "var(--radius-8)", backgroundColor: "var(--sf2)", fontFamily: "var(--font-mono)", fontSize: "var(--fs-12)", color: "var(--fg)", wordBreak: "break-all" }}>
        {res.token}
      </div>
    </Modal>
  );
}

function useMobileDevices(ws: string) {
  const [devices, setDevices] = useState<PublicMobileDevice[]>([]);
  const [cfg, setCfg] = useState<PublicMobileConfig | null>(null);

  const reload = useCallback(async () => {
    const [d, c] = await Promise.all([
      fetchDevices(ws).catch(() => [] as PublicMobileDevice[]),
      fetchMobileConfig(ws).catch(() => null),
    ]);
    setDevices(Array.isArray(d) ? d : []);
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
    <div style={{ overflowX: "auto" }}>
      <table style={{ width: "100%", borderCollapse: "collapse", textAlign: "left", fontSize: "var(--fs-13)" }}>
        <thead>
          <tr style={{ backgroundColor: "var(--sf2)", color: "var(--mu)", borderBottom: "1px solid var(--ln)" }}>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("mobile_col_device")}</th>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("mobile_col_status")}</th>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("mobile_col_read_only")}</th>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("mobile_col_last_used")}</th>
            <th style={{ padding: "var(--sp-8) var(--sp-12)", fontWeight: 600 }}>{t("mobile_col_actions")}</th>
          </tr>
        </thead>
        <tbody>
          {Array.isArray(devices) && devices.map((d) => (
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

function MobilePanelHeader({ onOpenModal }: { onOpenModal: () => void }) {
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "var(--sp-12)" }}>
      <div>
        <h2 style={{ margin: "0 0 2px 0", fontSize: "var(--fs-16)", fontWeight: 600, color: "var(--fg)" }}>{t("mobile_settings_title")}</h2>
        <p style={{ margin: 0, fontSize: "var(--fs-13)", color: "var(--mu)" }}>{t("mobile_settings_desc")}</p>
      </div>
      <Button variant="primary" size="sm" onClick={onOpenModal}>
        {t("mobile_add_device_btn")}
      </Button>
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
    <div style={{ backgroundColor: "var(--sf)", borderRadius: "var(--radius-10)", border: "1px solid var(--ln)", padding: "var(--sp-16)", marginBottom: "var(--sp-16)" }}>
      <MobilePanelHeader onOpenModal={() => setModalOpen(true)} />

      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)", padding: "var(--sp-12)", borderRadius: "var(--radius-8)", backgroundColor: "var(--sf2)", marginBottom: "var(--sp-16)" }}>
        <Checkbox id="ro-toggle" checked={cfg?.read_only || false} onChange={toggleRo} label={t("mobile_workspace_read_only")} />
      </div>

      <DevicesTable devices={devices} ws={ws} onReload={reload} />
      {modalOpen && <CreateDeviceModal onClose={() => setModalOpen(false)} onCreated={handleCreate} />}
      {newPair && <TokenDisplayModal res={newPair} onClose={() => setNewPair(null)} />}
    </div>
  );
}

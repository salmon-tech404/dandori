import React, { useEffect, useState } from "react";
import { AlertCircle, CheckCircle2, Play, Plus, Trash2 } from "lucide-react";
import { t } from "../../../core/i18n";
import { integrationsApi } from "../api/integrationsApi";
import type { CreateConnectionRequest, PublicIntegrationConnection } from "../types";

export interface IntegrationsSettingsProps {
  workspaceId: string;
}

const ConnectionItem: React.FC<{
  conn: PublicIntegrationConnection;
  onDelete: (id: string) => void;
  onTest: (id: string) => void;
}> = ({ conn, onDelete, onTest }) => (
  <div className="p-3 border rounded flex items-center justify-between text-xs bg-neutral-50">
    <div>
      <div className="font-semibold text-neutral-800 flex items-center gap-2">
        <span>{conn.name}</span>
        <span className="uppercase text-[10px] px-1.5 py-0.5 rounded bg-blue-100 text-blue-700">
          {conn.provider}
        </span>
      </div>
      <div className="text-neutral-500 mt-0.5">{conn.base_url}</div>
    </div>
    <div className="flex items-center gap-2">
      <button
        type="button"
        onClick={() => onTest(conn.id)}
        className="px-2 py-1 bg-white border rounded text-neutral-700 hover:bg-neutral-100 flex items-center gap-1"
      >
        <Play className="w-3 h-3 text-green-600" />
        <span>{t("integrations_test_btn")}</span>
      </button>
      <button
        type="button"
        onClick={() => onDelete(conn.id)}
        className="p-1 text-red-600 hover:bg-red-50 rounded"
      >
        <Trash2 className="w-3.5 h-3.5" />
      </button>
    </div>
  </div>
);

const ProviderSelect: React.FC<{
  provider: string;
  onChange: (p: string) => void;
}> = ({ provider, onChange }) => (
  <div>
    <label className="block font-medium text-neutral-700 mb-0.5">
      {t("integrations_provider_label")}
    </label>
    <select
      value={provider}
      onChange={(e) => onChange(e.target.value)}
      className="w-full border rounded px-2 py-1"
    >
      <option value="github">GitHub</option>
      <option value="gitlab">GitLab</option>
      <option value="linear">Linear</option>
    </select>
  </div>
);

const FormInputs: React.FC<{
  form: CreateConnectionRequest;
  onChange: (f: Partial<CreateConnectionRequest>) => void;
}> = ({ form, onChange }) => (
  <div className="space-y-2 text-xs">
    <ProviderSelect
      provider={form.provider}
      onChange={(provider) => onChange({ provider })}
    />
    <div>
      <label className="block font-medium text-neutral-700 mb-0.5">
        {t("integrations_name_label")}
      </label>
      <input
        type="text"
        className="w-full border rounded px-2 py-1"
        placeholder={t("integrations_name_placeholder")}
        value={form.name}
        onChange={(e) => onChange({ name: e.target.value })}
      />
    </div>
    <div>
      <label className="block font-medium text-neutral-700 mb-0.5">
        {t("integrations_token_label")}
      </label>
      <input
        type="password"
        className="w-full border rounded px-2 py-1"
        placeholder={t("integrations_token_placeholder")}
        value={form.token}
        onChange={(e) => onChange({ token: e.target.value })}
      />
    </div>
  </div>
);

const MessageAlert: React.FC<{ msg: { text: string; ok: boolean } | null }> = ({ msg }) => {
  if (!msg) return null;
  const cls = msg.ok ? "bg-green-50 text-green-700" : "bg-red-50 text-red-700";
  return (
    <div className={`p-2 rounded text-xs flex items-center gap-1.5 ${cls}`}>
      {msg.ok ? <CheckCircle2 className="w-4 h-4" /> : <AlertCircle className="w-4 h-4" />}
      <span>{msg.text}</span>
    </div>
  );
};

const NewConnectionCard: React.FC<{
  show: boolean;
  form: CreateConnectionRequest;
  onChange: (p: Partial<CreateConnectionRequest>) => void;
  onCreate: () => void;
}> = ({ show, form, onChange, onCreate }) => {
  if (!show) return null;
  return (
    <div className="p-3 border rounded bg-neutral-50 space-y-3">
      <FormInputs form={form} onChange={onChange} />
      <button
        type="button"
        onClick={onCreate}
        disabled={!form.name || !form.token}
        className="px-3 py-1 bg-green-600 text-white rounded text-xs hover:bg-green-700 disabled:opacity-50"
      >
        {t("integrations_save_btn")}
      </button>
    </div>
  );
};

const ConnectionList: React.FC<{
  conns: PublicIntegrationConnection[];
  onDelete: (id: string) => void;
  onTest: (id: string) => void;
}> = ({ conns, onDelete, onTest }) => {
  if (conns.length === 0) {
    return <p className="text-xs text-neutral-500">{t("integrations_no_connections")}</p>;
  }
  return (
    <div className="space-y-2">
      {conns.map((c) => (
        <ConnectionItem key={c.id} conn={c} onDelete={onDelete} onTest={onTest} />
      ))}
    </div>
  );
};

function useIntegrationsData(workspaceId: string) {
  const [conns, setConns] = useState<PublicIntegrationConnection[]>([]);
  const [loading, setLoading] = useState(true);
  const [msg, setMsg] = useState<{ text: string; ok: boolean } | null>(null);

  const reload = () => {
    integrationsApi
      .listConnections(workspaceId)
      .then((data) => setConns(data))
      .catch(() => {})
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    reload();
  }, [workspaceId]);

  return { conns, loading, msg, setMsg, reload };
}

function useIntegrationsActions(
  workspaceId: string,
  reload: () => void,
  setMsg: (m: { text: string; ok: boolean }) => void,
) {
  const [form, setForm] = useState<CreateConnectionRequest>({
    provider: "github",
    name: "",
    token: "",
  });

  const handleCreate = async () => {
    try {
      await integrationsApi.createConnection(workspaceId, form);
      setForm({ provider: "github", name: "", token: "" });
      setMsg({ text: t("integrations_test_success"), ok: true });
      reload();
    } catch (e: unknown) {
      setMsg({ text: e instanceof Error ? e.message : t("integrations_test_failed"), ok: false });
    }
  };

  const handleTest = async (id: string) => {
    try {
      await integrationsApi.testConnection(id);
      setMsg({ text: t("integrations_test_success"), ok: true });
    } catch (e: unknown) {
      setMsg({ text: e instanceof Error ? e.message : t("integrations_test_failed"), ok: false });
    }
  };

  const handleDelete = async (id: string) => {
    await integrationsApi.deleteConnection(id);
    reload();
  };

  return { form, setForm, handleCreate, handleTest, handleDelete };
}

export const IntegrationsSettings: React.FC<IntegrationsSettingsProps> = ({ workspaceId }) => {
  const { conns, loading, msg, setMsg, reload } = useIntegrationsData(workspaceId);
  const [showAdd, setShowAdd] = useState(false);
  const actions = useIntegrationsActions(workspaceId, reload, setMsg);

  if (loading) return null;

  return (
    <div className="p-4 border rounded bg-white space-y-4 max-w-xl">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-base font-semibold text-neutral-800">{t("integrations_title")}</h3>
          <p className="text-xs text-neutral-500">{t("integrations_desc")}</p>
        </div>
        <button
          type="button"
          onClick={() => setShowAdd(!showAdd)}
          className="px-2.5 py-1.5 bg-blue-600 text-white rounded text-xs hover:bg-blue-700 flex items-center gap-1"
        >
          <Plus className="w-3.5 h-3.5" />
          <span>{t("integrations_add_btn")}</span>
        </button>
      </div>
      <MessageAlert msg={msg} />
      <NewConnectionCard
        show={showAdd}
        form={actions.form}
        onChange={(p) => actions.setForm((prev) => ({ ...prev, ...p }))}
        onCreate={actions.handleCreate}
      />
      <ConnectionList conns={conns} onDelete={actions.handleDelete} onTest={actions.handleTest} />
    </div>
  );
};

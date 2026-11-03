import React, { useEffect, useState } from "react";
import { AlertCircle, CheckCircle2, Play, Plus, Trash2 } from "lucide-react";
import { t } from "../../../core/i18n";
import { Button } from "../../../ui/components/Button";
import { integrationsApi } from "../api/integrationsApi";
import type { CreateConnectionRequest, PublicIntegrationConnection } from "../types";

export interface IntegrationsSettingsProps {
  workspaceId: string;
}

const inputStyle: React.CSSProperties = {
  width: "100%",
  height: "var(--btn-height)",
  padding: "0 12px",
  fontSize: "var(--fs-13)",
  fontFamily: "var(--font-sans)",
  color: "var(--fg)",
  backgroundColor: "var(--sf)",
  border: "1px solid var(--ln)",
  borderRadius: "var(--radius-8)",
  outline: "none",
  boxSizing: "border-box",
};

const labelStyle: React.CSSProperties = {
  display: "block",
  fontSize: "var(--fs-12)",
  fontWeight: 500,
  color: "var(--mu)",
  marginBottom: "var(--sp-4)",
};

const ConnectionButtons: React.FC<{
  id: string;
  onDelete: (id: string) => void;
  onTest: (id: string) => void;
}> = ({ id, onDelete, onTest }) => (
  <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
    <Button
      type="button"
      variant="secondary"
      size="sm"
      onClick={() => onTest(id)}
      icon={<Play size={12} style={{ color: "var(--gr)" }} />}
    >
      {t("integrations_test_btn")}
    </Button>
    <Button
      type="button"
      variant="ghost"
      size="icon"
      onClick={() => onDelete(id)}
      style={{ color: "var(--rd)" }}
    >
      <Trash2 size={14} />
    </Button>
  </div>
);

const ConnectionItem: React.FC<{
  conn: PublicIntegrationConnection;
  onDelete: (id: string) => void;
  onTest: (id: string) => void;
}> = ({ conn, onDelete, onTest }) => (
  <div
    style={{
      padding: "var(--sp-12)",
      borderRadius: "var(--radius-8)",
      border: "1px solid var(--ln)",
      backgroundColor: "var(--sf2)",
      display: "flex",
      alignItems: "center",
      justifyContent: "space-between",
      fontSize: "var(--fs-13)",
    }}
  >
    <div>
      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)", fontWeight: 600, color: "var(--fg)" }}>
        <span>{conn.name}</span>
        <span
          style={{
            textTransform: "uppercase",
            fontSize: "10px",
            padding: "2px 6px",
            borderRadius: "var(--radius-4)",
            backgroundColor: "var(--blb)",
            color: "var(--bl)",
            fontWeight: 700,
          }}
        >
          {conn.provider}
        </span>
      </div>
      <div style={{ fontSize: "var(--fs-12)", color: "var(--mu)", marginTop: "2px" }}>{conn.base_url}</div>
    </div>
    <ConnectionButtons id={conn.id} onDelete={onDelete} onTest={onTest} />
  </div>
);

const ProviderSelect: React.FC<{
  provider: string;
  onChange: (p: string) => void;
}> = ({ provider, onChange }) => (
  <div>
    <label style={labelStyle}>
      {t("integrations_provider_label")}
    </label>
    <select
      value={provider}
      onChange={(e) => onChange(e.target.value)}
      style={inputStyle}
    >
      <option value="github">{t("integration_provider_github")}</option>
      <option value="gitlab">{t("integration_provider_gitlab")}</option>
      <option value="linear">{t("integration_provider_linear")}</option>
    </select>
  </div>
);

const FormInputs: React.FC<{
  form: CreateConnectionRequest;
  onChange: (f: Partial<CreateConnectionRequest>) => void;
}> = ({ form, onChange }) => (
  <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
    <ProviderSelect
      provider={form.provider}
      onChange={(provider) => onChange({ provider })}
    />
    <div>
      <label style={labelStyle}>
        {t("integrations_name_label")}
      </label>
      <input
        type="text"
        style={inputStyle}
        placeholder={t("integrations_name_placeholder")}
        value={form.name}
        onChange={(e) => onChange({ name: e.target.value })}
      />
    </div>
    <div>
      <label style={labelStyle}>
        {t("integrations_token_label")}
      </label>
      <input
        type="password"
        style={inputStyle}
        placeholder={t("integrations_token_placeholder")}
        value={form.token}
        onChange={(e) => onChange({ token: e.target.value })}
      />
    </div>
  </div>
);

const MessageAlert: React.FC<{ msg: { text: string; ok: boolean } | null }> = ({ msg }) => {
  if (!msg) return null;
  const bg = msg.ok ? "var(--grb)" : "var(--rdb)";
  const color = msg.ok ? "var(--gr)" : "var(--rd)";
  return (
    <div
      style={{
        padding: "var(--sp-8) var(--sp-12)",
        borderRadius: "var(--radius-8)",
        fontSize: "var(--fs-13)",
        display: "flex",
        alignItems: "center",
        gap: "var(--sp-8)",
        backgroundColor: bg,
        color,
        marginBottom: "var(--sp-12)",
      }}
    >
      {msg.ok ? <CheckCircle2 size={16} /> : <AlertCircle size={16} />}
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
    <div
      style={{
        padding: "var(--sp-12)",
        borderRadius: "var(--radius-8)",
        border: "1px solid var(--ln)",
        backgroundColor: "var(--sf2)",
        display: "flex",
        flexDirection: "column",
        gap: "var(--sp-12)",
        marginBottom: "var(--sp-12)",
      }}
    >
      <FormInputs form={form} onChange={onChange} />
      <div>
        <Button
          type="button"
          variant="primary"
          size="sm"
          onClick={onCreate}
          disabled={!form.name || !form.token}
        >
          {t("integrations_save_btn")}
        </Button>
      </div>
    </div>
  );
};

const ConnectionList: React.FC<{
  conns: PublicIntegrationConnection[];
  onDelete: (id: string) => void;
  onTest: (id: string) => void;
}> = ({ conns, onDelete, onTest }) => {
  if (conns.length === 0) {
    return <p style={{ fontSize: "var(--fs-13)", color: "var(--mu)", margin: 0 }}>{t("integrations_no_connections")}</p>;
  }
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
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
  setMsg: (m: { text: string; ok: boolean } | null) => void
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
      setMsg({ text: t("integrations_save_success"), ok: true });
      reload();
    } catch {
      setMsg({ text: t("integrations_save_error"), ok: false });
    }
  };

  const handleTest = async (id: string) => {
    try {
      const ok = await integrationsApi.testConnection(id);
      if (ok) {
        setMsg({ text: t("integrations_test_success", { status: "200" }), ok: true });
      } else {
        setMsg({ text: t("integrations_test_error", { err: "failed" }), ok: false });
      }
    } catch (err: unknown) {
      const errMsg = err instanceof Error ? err.message : "network error";
      setMsg({ text: t("integrations_test_error", { err: errMsg }), ok: false });
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
    <div style={{ backgroundColor: "var(--sf)", borderRadius: "var(--radius-10)", border: "1px solid var(--ln)", padding: "var(--sp-16)", marginBottom: "var(--sp-16)" }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "var(--sp-12)" }}>
        <div>
          <h3 style={{ margin: "0 0 2px 0", fontSize: "var(--fs-16)", fontWeight: 600, color: "var(--fg)" }}>{t("integrations_title")}</h3>
          <p style={{ margin: 0, fontSize: "var(--fs-13)", color: "var(--mu)" }}>{t("integrations_desc")}</p>
        </div>
        <Button
          type="button"
          variant="primary"
          size="sm"
          onClick={() => setShowAdd(!showAdd)}
          icon={<Plus size={14} />}
        >
          {t("integrations_add_btn")}
        </Button>
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

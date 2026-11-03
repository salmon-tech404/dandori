import { useCallback, useEffect, useState } from "react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { Input } from "../../ui/components/Input";
import { fetchMobileApprovals, fetchMobileStatus, submitMobileDecision } from "./api";
import type { MobileApprovalItem, MobileStatusResponse } from "./types";

const TOKEN_KEY = "dandori_mobile_token";

function TokenInputForm({ onSave }: { onSave: (t: string) => void }) {
  const [val, setVal] = useState("");

  const handleSub = (e: React.FormEvent) => {
    e.preventDefault();
    if (val.trim()) onSave(val.trim());
  };

  return (
    <form onSubmit={handleSub} style={{ display: "flex", flexDirection: "column", gap: "var(--sp-16)", padding: "var(--sp-16)" }}>
      <h2 style={{ margin: 0, fontSize: "var(--fs-18)", fontWeight: 600, color: "var(--fg)" }}>{t("mobile_pair_title")}</h2>
      <p style={{ margin: 0, fontSize: "var(--fs-14)", color: "var(--mu)" }}>{t("mobile_pair_desc")}</p>
      <Input
        type="password"
        value={val}
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => setVal(e.target.value)}
        placeholder={t("mobile_token_placeholder")}
        className="min-h-[44px]"
      />
      <Button variant="primary" type="submit" className="min-h-[44px]">
        {t("mobile_connect_btn")}
      </Button>
    </form>
  );
}

function MobileTitle({ status }: { status: MobileStatusResponse | null }) {
  return (
    <div>
      <h1 style={{ margin: 0, fontSize: "var(--fs-16)", fontWeight: 700, color: "var(--fg)" }}>{t("mobile_app_title")}</h1>
      <p style={{ margin: "2px 0 0", fontSize: "var(--fs-12)", color: "var(--mu)" }}>
        {status?.device_name || t("mobile_connected_device")}
        {status?.is_read_only && (
          <span style={{ marginLeft: 8, borderRadius: "var(--radius-4)", backgroundColor: "var(--orb)", padding: "2px 6px", fontSize: "10px", color: "var(--or)" }}>
            {t("mobile_read_only_badge")}
          </span>
        )}
      </p>
    </div>
  );
}

function MobileHeaderActions({ onRefresh, onLogout }: { onRefresh: () => void; onLogout: () => void }) {
  return (
    <div style={{ display: "flex", gap: "var(--sp-8)" }}>
      <Button variant="secondary" size="sm" type="button" onClick={onRefresh} className="min-h-[44px]">
        {t("mobile_refresh_btn")}
      </Button>
      <Button variant="danger" size="sm" type="button" onClick={onLogout} className="min-h-[44px]">
        {t("mobile_disconnect_btn")}
      </Button>
    </div>
  );
}

function MobileHeader({
  status,
  onRefresh,
  onLogout,
}: {
  status: MobileStatusResponse | null;
  onRefresh: () => void;
  onLogout: () => void;
}) {
  return (
    <header style={{ display: "flex", alignItems: "center", justifyContent: "space-between", borderBottom: "1px solid var(--ln)", backgroundColor: "var(--sf)", padding: "var(--sp-12) var(--sp-16)" }}>
      <MobileTitle status={status} />
      <MobileHeaderActions onRefresh={onRefresh} onLogout={onLogout} />
    </header>
  );
}

function ApprovalCardActions({
  itemId,
  onAction,
}: {
  itemId: string;
  onAction: (id: string, act: "approve" | "reject") => void;
}) {
  return (
    <div style={{ display: "flex", gap: "var(--sp-12)" }}>
      <Button
        variant="primary"
        type="button"
        onClick={() => onAction(itemId, "approve")}
        className="min-h-[44px]"
        style={{ flex: 1 }}
      >
        {t("mobile_approve_btn")}
      </Button>
      <Button
        variant="danger"
        type="button"
        onClick={() => onAction(itemId, "reject")}
        className="min-h-[44px]"
        style={{ flex: 1 }}
      >
        {t("mobile_reject_btn")}
      </Button>
    </div>
  );
}

function ApprovalCard({
  item,
  readOnly,
  onAction,
}: {
  item: MobileApprovalItem;
  readOnly: boolean;
  onAction: (id: string, act: "approve" | "reject") => void;
}) {
  return (
    <div style={{ borderRadius: "var(--radius-8)", border: "1px solid var(--ln)", backgroundColor: "var(--sf)", padding: "var(--sp-16)" }}>
      <div style={{ marginBottom: "var(--sp-8)", display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: "var(--sp-8)" }}>
        <h3 style={{ margin: 0, fontWeight: 600, color: "var(--fg)" }}>{item.task_title}</h3>
        <span style={{ borderRadius: "var(--radius-4)", backgroundColor: "var(--orb)", padding: "2px 8px", fontSize: "var(--fs-12)", fontWeight: 600, color: "var(--or)" }}>
          {item.status}
        </span>
      </div>
      <p style={{ margin: "0 0 var(--sp-16)", fontSize: "var(--fs-12)", color: "var(--mu)" }}>{item.reason}</p>
      {!readOnly && <ApprovalCardActions itemId={item.id} onAction={onAction} />}
    </div>
  );
}

function ApprovalList({
  items,
  readOnly,
  onAction,
}: {
  items: MobileApprovalItem[];
  readOnly: boolean;
  onAction: (id: string, act: "approve" | "reject") => void;
}) {
  if (items.length === 0) {
    return (
      <div style={{ padding: "var(--sp-32)", textAlign: "center", fontSize: "var(--fs-14)", color: "var(--mu)" }}>
        {t("mobile_empty_approvals")}
      </div>
    );
  }
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)", padding: "var(--sp-16)" }}>
      {items.map((it) => (
        <ApprovalCard key={it.id} item={it} readOnly={readOnly} onAction={onAction} />
      ))}
    </div>
  );
}

function useMobileData(token: string) {
  const [status, setStatus] = useState<MobileStatusResponse | null>(null);
  const [items, setItems] = useState<MobileApprovalItem[]>([]);
  const [msg, setMsg] = useState("");

  const loadData = useCallback(async (tok: string) => {
    try {
      const [st, apprs] = await Promise.all([fetchMobileStatus(tok), fetchMobileApprovals(tok)]);
      setStatus(st);
      setItems(apprs);
      setMsg("");
    } catch (e) {
      setMsg((e as Error).message);
    }
  }, []);

  useEffect(() => {
    if (token) void loadData(token);
  }, [token, loadData]);

  const handleDecision = async (id: string, act: "approve" | "reject") => {
    if (!token) return;
    try {
      await submitMobileDecision(token, id, { action: act });
      void loadData(token);
    } catch (e) {
      setMsg((e as Error).message);
    }
  };

  return { status, items, msg, reload: () => void loadData(token), handleDecision };
}

export function MobileScreen() {
  const [token, setToken] = useState(() => localStorage.getItem(TOKEN_KEY) || "");
  const { status, items, msg, reload, handleDecision } = useMobileData(token);

  if (!token) {
    return (
      <div style={{ maxWidth: 448, margin: "0 auto", padding: "var(--sp-16)" }}>
        <TokenInputForm onSave={(t) => { localStorage.setItem(TOKEN_KEY, t); setToken(t); }} />
      </div>
    );
  }

  return (
    <div style={{ maxWidth: 448, minHeight: "100vh", margin: "0 auto", backgroundColor: "var(--bg)", color: "var(--fg)" }}>
      <MobileHeader
        status={status}
        onRefresh={reload}
        onLogout={() => { localStorage.removeItem(TOKEN_KEY); setToken(""); }}
      />
      {msg && <div style={{ margin: "var(--sp-16)", borderRadius: "var(--radius-4)", backgroundColor: "var(--rdb)", padding: "var(--sp-12)", fontSize: "var(--fs-12)", color: "var(--rd)" }}>{msg}</div>}
      <ApprovalList items={items} readOnly={Boolean(status?.is_read_only)} onAction={handleDecision} />
    </div>
  );
}

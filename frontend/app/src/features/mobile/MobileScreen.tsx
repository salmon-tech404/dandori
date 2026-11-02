import { useCallback, useEffect, useState } from "react";
import { t } from "../../core/i18n";
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
    <form onSubmit={handleSub} className="flex flex-col gap-4 p-4">
      <h2 className="text-lg font-semibold text-neutral-100">{t("mobile_pair_title")}</h2>
      <p className="text-sm text-neutral-400">{t("mobile_pair_desc")}</p>
      <input
        type="password"
        value={val}
        onChange={(e) => setVal(e.target.value)}
        placeholder={t("mobile_token_placeholder")}
        className="min-h-[44px] rounded border border-neutral-700 bg-neutral-900 px-3 py-2 text-sm text-neutral-100"
      />
      <button
        type="submit"
        className="min-h-[44px] rounded bg-indigo-600 px-4 py-2 font-medium text-white hover:bg-indigo-500"
      >
        {t("mobile_connect_btn")}
      </button>
    </form>
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
    <header className="flex items-center justify-between border-b border-neutral-800 bg-neutral-900/80 px-4 py-3">
      <div>
        <h1 className="text-base font-bold text-neutral-100">{t("mobile_app_title")}</h1>
        <p className="text-xs text-neutral-400">
          {status?.device_name || t("mobile_connected_device")}
          {status?.is_read_only && (
            <span className="ml-2 rounded bg-amber-950 px-1.5 py-0.5 text-[10px] text-amber-400">
              {t("mobile_read_only_badge")}
            </span>
          )}
        </p>
      </div>
      <div className="flex gap-2">
        <button
          onClick={onRefresh}
          className="min-h-[44px] min-w-[44px] rounded bg-neutral-800 px-3 text-xs text-neutral-200 hover:bg-neutral-700"
        >
          {t("mobile_refresh_btn")}
        </button>
        <button
          onClick={onLogout}
          className="min-h-[44px] min-w-[44px] rounded bg-rose-950 px-3 text-xs text-rose-300 hover:bg-rose-900"
        >
          {t("mobile_disconnect_btn")}
        </button>
      </div>
    </header>
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
    <div className="rounded-lg border border-neutral-800 bg-neutral-900 p-4 shadow-sm">
      <div className="mb-2 flex items-start justify-between gap-2">
        <h3 className="font-semibold text-neutral-100">{item.task_title}</h3>
        <span className="rounded bg-amber-950 px-2 py-0.5 text-xs font-semibold text-amber-400">
          {item.status}
        </span>
      </div>
      <p className="mb-4 text-xs text-neutral-400">{item.reason}</p>
      {!readOnly && (
        <div className="flex gap-3">
          <button
            onClick={() => onAction(item.id, "approve")}
            className="flex-1 min-h-[44px] rounded bg-emerald-600 px-4 py-2 font-medium text-white hover:bg-emerald-500"
          >
            {t("mobile_approve_btn")}
          </button>
          <button
            onClick={() => onAction(item.id, "reject")}
            className="flex-1 min-h-[44px] rounded bg-rose-600 px-4 py-2 font-medium text-white hover:bg-rose-500"
          >
            {t("mobile_reject_btn")}
          </button>
        </div>
      )}
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
      <div className="p-8 text-center text-sm text-neutral-400">
        {t("mobile_empty_approvals")}
      </div>
    );
  }
  return (
    <div className="flex flex-col gap-3 p-4">
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
      <div className="mx-auto max-w-md p-4">
        <TokenInputForm onSave={(t) => { localStorage.setItem(TOKEN_KEY, t); setToken(t); }} />
      </div>
    );
  }

  return (
    <div className="mx-auto min-h-screen max-w-md bg-neutral-950 text-neutral-100">
      <MobileHeader
        status={status}
        onRefresh={reload}
        onLogout={() => { localStorage.removeItem(TOKEN_KEY); setToken(""); }}
      />
      {msg && <div className="m-4 rounded bg-rose-950 p-3 text-xs text-rose-300">{msg}</div>}
      <ApprovalList items={items} readOnly={Boolean(status?.is_read_only)} onAction={handleDecision} />
    </div>
  );
}

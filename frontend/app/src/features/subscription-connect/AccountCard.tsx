import type { ReactElement } from "react";
import type { AgentAccount } from "./types";
import { t } from "../../core/i18n";

interface AccountCardProps {
  acc: AgentAccount;
  onShowInstructions: (id: string) => void;
  onRecheck: (id: string) => void;
  onDelete: (id: string) => void;
}

function StatusBadge({ status }: { status: string }) {
  if (status === "ready") return <span className="chip chip-green">{t("account_status_ready")}</span>;
  if (status === "not_installed") return <span className="chip chip-gray">{t("account_status_not_installed")}</span>;
  if (status === "not_logged_in" || status === "session_expired") return <span className="chip chip-orange">{t("account_status_not_logged_in")}</span>;
  return <span className="chip chip-gray">{t("account_status_unknown")}</span>;
}

function ProviderAvatar({ provider }: { provider: string }) {
  const p = provider.toLowerCase();
  const cls = p === "claude" ? "provider-circle-claude" : p === "codex" ? "provider-circle-codex" : "provider-circle-gemini";
  return <div className={`provider-circle ${cls}`}>{p === "claude" || p === "codex" ? "C" : "G"}</div>;
}

function CardActions({ id, onShowInstructions, onRecheck, onDelete }: { id: string; onShowInstructions: (id: string) => void; onRecheck: (id: string) => void; onDelete: (id: string) => void }) {
  return (
    <div style={{ display: "flex", gap: "8px", flexWrap: "wrap", marginTop: "auto", paddingTop: "8px", borderTop: "1px solid var(--ln)" }}>
      <button type="button" className="btn btn-secondary" style={{ minHeight: "44px", fontSize: "12px" }} onClick={() => onShowInstructions(id)}>{t("btn_login_instructions")}</button>
      <button type="button" className="btn btn-secondary" style={{ minHeight: "44px", fontSize: "12px" }} onClick={() => onRecheck(id)}>{t("btn_recheck_status")}</button>
      <button type="button" className="btn btn-secondary" style={{ minHeight: "44px", fontSize: "12px", color: "var(--rd)" }} onClick={() => onDelete(id)}>{t("btn_delete_account")}</button>
    </div>
  );
}

export function AccountCard({ acc, onShowInstructions, onRecheck, onDelete }: AccountCardProps): ReactElement {
  return (
    <div style={{ backgroundColor: "var(--sf)", border: "1px solid var(--ln)", borderRadius: "10px", padding: "16px", display: "flex", flexDirection: "column", gap: "12px" }}>
      <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
        <ProviderAvatar provider={acc.provider} />
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ fontWeight: 600, fontSize: "15px", color: "var(--fg)" }}>{acc.label}</div>
          <div style={{ fontFamily: "var(--font-mono, monospace)", fontSize: "12px", color: "var(--mu)" }}>{acc.id.slice(0, 8)} • {acc.provider.toUpperCase()}</div>
        </div>
        <StatusBadge status={acc.status} />
      </div>
      <div style={{ fontSize: "12px", color: "var(--mu)" }}><span>{t("account_priority")}: {acc.priority}</span></div>
      <CardActions id={acc.id} onShowInstructions={onShowInstructions} onRecheck={onRecheck} onDelete={onDelete} />
    </div>
  );
}

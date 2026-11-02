import { useState, type ReactElement } from "react";
import { Zap } from "lucide-react";
import type { ProviderGroup, AccountDetail } from "./types";
import { t } from "../../core/i18n";

function AccountRunningBadge({ title }: { title: string }): ReactElement {
  return (
    <div
      className="running-task-badge"
      style={{
        marginTop: "6px",
        display: "inline-flex",
        alignItems: "center",
        gap: "6px",
        padding: "2px 8px",
        borderRadius: "4px",
        backgroundColor: "var(--bl-soft, #eef4ff)",
        color: "var(--bl, #0052cc)",
        fontSize: "12px",
        fontWeight: 500,
      }}
    >
      <Zap size={14} />
      <span>{t("multi_account_active_task")}: {title}</span>
    </div>
  );
}

function AccountInfoCol({ acc }: { acc: AccountDetail }): ReactElement {
  return (
    <div style={{ minWidth: "180px", flex: 1 }}>
      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <span style={{ fontWeight: 600, fontSize: "14px", color: "var(--fg)" }}>{acc.label}</span>
        <span className={`chip ${acc.status === "ready" ? "chip-green" : "chip-gray"}`} style={{ fontSize: "11px" }}>
          {acc.status}
        </span>
      </div>
      <div style={{ fontSize: "11px", color: "var(--mu)", fontFamily: "monospace", marginTop: "2px" }}>
        {acc.id.slice(0, 8)} • {acc.home_dir}
      </div>
      {acc.running_task && <AccountRunningBadge title={acc.running_task.task_title} />}
    </div>
  );
}

interface AccountActionsProps {
  acc: AccountDetail;
  priority: number;
  setPriority: (p: number) => void;
  onUpdate: (id: string, payload: { priority?: number; enabled?: boolean; label?: string }) => Promise<void>;
  onShowRelogin: (acc: AccountDetail) => void;
}

function AccountActionsCol({ acc, priority, setPriority, onUpdate, onShowRelogin }: AccountActionsProps): ReactElement {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
      <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
        <label htmlFor={`priority-${acc.id}`} style={{ fontSize: "12px", color: "var(--mu)" }}>
          {t("account_priority")}:
        </label>
        <input
          id={`priority-${acc.id}`}
          type="number"
          value={priority}
          onChange={(e) => setPriority(Number(e.target.value))}
          style={{ width: "60px", height: "32px", padding: "0 6px", borderRadius: "4px", border: "1px solid var(--ln)" }}
        />
        {priority !== acc.priority && (
          <button
            type="button"
            className="btn btn-secondary"
            style={{ height: "32px", fontSize: "12px", padding: "0 8px" }}
            onClick={() => onUpdate(acc.id, { priority })}
          >
            {t("multi_account_save_priority")}
          </button>
        )}
      </div>

      <button type="button" className="btn btn-secondary" style={{ height: "32px", fontSize: "12px" }} onClick={() => onShowRelogin(acc)}>
        {t("multi_account_relogin_hint")}
      </button>

      <button
        type="button"
        className={`btn ${acc.enabled ? "btn-secondary" : "btn-primary"}`}
        style={{ height: "32px", fontSize: "12px" }}
        onClick={() => onUpdate(acc.id, { enabled: !acc.enabled })}
      >
        {acc.enabled ? t("multi_account_disable_label") : t("multi_account_enable_label")}
      </button>
    </div>
  );
}

interface AccountRowProps {
  acc: AccountDetail;
  onUpdate: (id: string, payload: { priority?: number; enabled?: boolean; label?: string }) => Promise<void>;
  onShowRelogin: (acc: AccountDetail) => void;
}

function AccountRowItem({ acc, onUpdate, onShowRelogin }: AccountRowProps): ReactElement {
  const [priority, setPriority] = useState<number>(acc.priority);

  return (
    <div
      className="account-row-item"
      style={{
        display: "flex",
        flexWrap: "wrap",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "10px 14px",
        border: "1px solid var(--ln)",
        borderRadius: "8px",
        backgroundColor: acc.enabled ? "var(--bg)" : "var(--sf-muted, #f8f9fa)",
        opacity: acc.enabled ? 1 : 0.7,
        gap: "12px",
      }}
    >
      <AccountInfoCol acc={acc} />
      <AccountActionsCol
        acc={acc}
        priority={priority}
        setPriority={setPriority}
        onUpdate={onUpdate}
        onShowRelogin={onShowRelogin}
      />
    </div>
  );
}

function GroupHeader({ group }: { group: ProviderGroup }): ReactElement {
  return (
    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "12px" }}>
      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <h3 style={{ fontSize: "16px", margin: 0, color: "var(--fg)" }}>{group.provider.toUpperCase()}</h3>
        <span style={{ fontSize: "12px", color: "var(--mu)" }}>({group.accounts.length} tài khoản)</span>
      </div>
      {!group.supports_home_isolation && (
        <span className="chip chip-orange" style={{ fontSize: "11px" }}>
          {t("multi_account_no_home_isolation")}
        </span>
      )}
    </div>
  );
}

interface AccountGroupCardProps {
  group: ProviderGroup;
  onUpdate: (id: string, payload: { priority?: number; enabled?: boolean; label?: string }) => Promise<void>;
  onShowRelogin: (acc: AccountDetail) => void;
}

export function AccountGroupCard({ group, onUpdate, onShowRelogin }: AccountGroupCardProps): ReactElement {
  return (
    <div
      className="provider-group-card"
      style={{
        backgroundColor: "var(--sf)",
        border: "1px solid var(--ln)",
        borderRadius: "10px",
        padding: "16px",
        marginBottom: "16px",
      }}
    >
      <GroupHeader group={group} />
      <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
        {group.accounts.map((acc) => (
          <AccountRowItem
            key={acc.id}
            acc={acc}
            onUpdate={onUpdate}
            onShowRelogin={onShowRelogin}
          />
        ))}
      </div>
    </div>
  );
}

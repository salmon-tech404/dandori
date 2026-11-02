import { useState, useEffect, type ReactElement } from "react";
import type { ProviderGroup, AccountDetail } from "./types";
import { fetchGroupedAccounts, updateAccount } from "./api";
import { AccountGroupCard } from "./AccountGroupCard";
import { t } from "../../core/i18n";

function ReloginModal({ account, onClose }: { account: AccountDetail; onClose: () => void }): ReactElement {
  const loginCmd = account.provider === "claude"
    ? `$env:USERPROFILE="${account.home_dir}"; claude auth login`
    : `$env:USERPROFILE="${account.home_dir}"; ${account.provider} login`;

  return (
    <div className="modal-backdrop">
      <div className="modal-content" style={{ maxWidth: "500px" }}>
        <h3 style={{ margin: "0 0 12px 0", color: "var(--fg)" }}>
          {t("dialog_login_title", { label: account.label })}
        </h3>
        <p style={{ fontSize: "13px", color: "var(--mu)", marginBottom: "12px" }}>
          {t("dialog_login_desc")}
        </p>
        <div style={{ backgroundColor: "var(--bg)", border: "1px solid var(--ln)", borderRadius: "6px", padding: "12px", fontFamily: "monospace", fontSize: "12px", overflowX: "auto" }}>
          <code>{loginCmd}</code>
        </div>
        <div style={{ display: "flex", justifyContent: "flex-end", marginTop: "16px" }}>
          <button type="button" className="btn btn-primary" style={{ minHeight: "44px" }} onClick={onClose}>
            {t("btn_close")}
          </button>
        </div>
      </div>
    </div>
  );
}

function ManagerHeader(): ReactElement {
  return (
    <div style={{ marginBottom: "16px" }}>
      <h2 style={{ fontSize: "18px", margin: "0 0 4px 0", color: "var(--fg)" }}>
        {t("multi_account_title")}
      </h2>
      <p style={{ fontSize: "13px", margin: 0, color: "var(--mu)" }}>
        {t("multi_account_desc")}
      </p>
    </div>
  );
}

function useMultiAccountData() {
  const [groups, setGroups] = useState<ProviderGroup[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadData = async () => {
    setLoading(true);
    setError(null);
    try {
      setGroups(await fetchGroupedAccounts());
    } catch (e) {
      setError(e instanceof Error ? e.message : "Lỗi không xác định");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void loadData();
  }, []);

  const handleUpdate = async (id: string, payload: { priority?: number; enabled?: boolean; label?: string }) => {
    try {
      await updateAccount(id, payload);
      await loadData();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Cập nhật tài khoản thất bại");
    }
  };

  return { groups, loading, error, handleUpdate };
}

export function MultiAccountManager(): ReactElement {
  const { groups, loading, error, handleUpdate } = useMultiAccountData();
  const [selectedRelogin, setSelectedRelogin] = useState<AccountDetail | null>(null);

  return (
    <div className="multi-account-manager" style={{ padding: "16px 0" }}>
      <ManagerHeader />
      {loading && <p style={{ color: "var(--mu)" }}>{t("loading")}</p>}
      {error && <p style={{ color: "var(--rd)" }}>{error}</p>}
      {!loading && !error && groups.length === 0 && (
        <div style={{ border: "1px dashed var(--ln)", borderRadius: "8px", padding: "32px", textAlign: "center", color: "var(--mu)" }}>
          {t("no_accounts_connected")}
        </div>
      )}
      {!loading && !error && groups.map((g) => (
        <AccountGroupCard
          key={g.provider}
          group={g}
          onUpdate={handleUpdate}
          onShowRelogin={(acc) => setSelectedRelogin(acc)}
        />
      ))}
      {selectedRelogin && <ReloginModal account={selectedRelogin} onClose={() => setSelectedRelogin(null)} />}
    </div>
  );
}

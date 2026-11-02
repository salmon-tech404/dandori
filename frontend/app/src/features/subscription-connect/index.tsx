import type { ReactElement } from "react";
import { AccountCard } from "./AccountCard";
import { AddAccountModal } from "./AddAccountModal";
import { LoginInstructionsModal } from "./LoginInstructionsModal";
import { useSubscriptionAccounts } from "./useSubscriptionAccounts";
import { t } from "../../core/i18n";

export function SubscriptionAccounts(): ReactElement {
  const {
    accounts, loading, error, showAddModal, activeInstructions,
    setShowAddModal, setActiveInstructions,
    handleAdd, handleRecheck, handleDelete, handleShowInstructions,
  } = useSubscriptionAccounts();

  return (
    <div className="subscription-accounts-container" style={{ padding: "16px 0" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: "16px" }}>
        <div>
          <h2 style={{ fontSize: "18px", margin: "0 0 4px 0", color: "var(--fg)" }}>{t("subscription_connect_title")}</h2>
          <p style={{ fontSize: "13px", margin: 0, color: "var(--mu)" }}>{t("subscription_connect_desc")}</p>
        </div>
        <button type="button" className="btn btn-primary" style={{ minHeight: "44px" }} onClick={() => setShowAddModal(true)}>
          {t("btn_add_account")}
        </button>
      </div>
      {loading && <p style={{ color: "var(--mu)" }}>{t("loading")}</p>}
      {error && <p style={{ color: "var(--rd)" }}>{error}</p>}
      {!loading && !error && accounts.length === 0 && (
        <div style={{ border: "1px dashed var(--ln)", borderRadius: "8px", padding: "32px", textAlign: "center", color: "var(--mu)" }}>
          {t("no_accounts_connected")}
        </div>
      )}
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(320px, 1fr))", gap: "16px" }}>
        {accounts.map((acc) => (
          <AccountCard key={acc.id} acc={acc} onShowInstructions={handleShowInstructions} onRecheck={handleRecheck} onDelete={handleDelete} />
        ))}
      </div>
      {showAddModal && <AddAccountModal onClose={() => setShowAddModal(false)} onSubmit={handleAdd} />}
      {activeInstructions && <LoginInstructionsModal instructions={activeInstructions} onClose={() => setActiveInstructions(null)} />}
    </div>
  );
}

export const contributions = {
  "subscription-accounts": SubscriptionAccounts,
};

export default contributions;

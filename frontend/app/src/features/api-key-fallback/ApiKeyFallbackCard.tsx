import { useState, useEffect } from "react";
import { Plus } from "lucide-react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { Card } from "../../ui/components/Card";
import { Chip } from "../../ui/components/Chip";
import type { ApiKeyAccount, SpendingLimit } from "./types";
import {
  fetchApiKeyAccounts,
  createApiKeyAccount,
  toggleApiKeyAccount,
  deleteApiKeyAccount,
} from "./api";
import { AddApiKeyModal } from "./AddApiKeyModal";

type CreateApiKeyPayload = Parameters<typeof createApiKeyAccount>[0];

export function ApiKeySpendingBar({ limit }: { limit: SpendingLimit }) {
  const pct = limit.limit_value > 0 ? (limit.used_value / limit.limit_value) * 100 : 0;
  const isAlert = pct >= limit.alert_threshold;
  const isCap = pct >= 100;
  const fillClass = isCap ? "bg-danger" : isAlert ? "bg-warning" : "bg-primary";
  const barColor = isCap ? "var(--rd)" : isAlert ? "var(--or)" : "var(--bl)";

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-4)", minWidth: "180px" }}>
      <div style={{ display: "flex", justifyContent: "space-between", fontSize: "var(--fs-12)", color: "var(--mu)" }}>
        <span>{t("api_key_used_label")} {limit.used_value.toFixed(2)} / {limit.limit_value.toFixed(2)} ({limit.period})</span>
        <span style={{ fontWeight: 600, color: isCap ? "var(--rd)" : isAlert ? "var(--or)" : "inherit" }}>{pct.toFixed(0)}%</span>
      </div>
      <div style={{ width: "100%", height: "6px", borderRadius: "999px", backgroundColor: "var(--sf2)", overflow: "hidden" }}>
        <div
          className={`progress-bar-fill ${fillClass}`}
          style={{ width: `${Math.min(pct, 100)}%`, height: "100%", borderRadius: "999px", backgroundColor: barColor }}
          data-testid="spending-progress-bar"
        />
      </div>
    </div>
  );
}

export function ApiKeyItem({
  acc,
  onToggle,
  onDelete,
}: {
  acc: ApiKeyAccount;
  onToggle: (id: string) => void;
  onDelete: (id: string) => void;
}) {
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: "var(--sp-12)", padding: "var(--sp-12)", backgroundColor: "var(--sf2)", borderRadius: "var(--radius-8)", border: "1px solid var(--ln)", flexWrap: "wrap" }}>
      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)", flexWrap: "wrap", flex: 1, minWidth: "240px" }}>
        <Chip variant="blue">{acc.provider}</Chip>
        <span style={{ fontWeight: 600, fontSize: "var(--fs-13)", color: "var(--fg)" }}>{acc.label}</span>
        <span style={{ fontFamily: "var(--font-mono, monospace)", fontSize: "var(--fs-12)", color: "var(--mu)" }} data-testid="masked-key">{acc.masked_key}</span>
        <Chip variant={acc.enabled ? "green" : "gray"}>
          {acc.enabled ? t("api_key_fallback_status_enabled") : t("api_key_fallback_status_disabled")}
        </Chip>
      </div>
      {acc.spending_limit && <ApiKeySpendingBar limit={acc.spending_limit} />}
      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
        <Button variant="secondary" size="sm" onClick={() => onToggle(acc.id)}>
          {acc.enabled ? t("api_key_fallback_toggle_off") : t("api_key_fallback_toggle_on")}
        </Button>
        <Button variant="danger" size="sm" onClick={() => onDelete(acc.id)}>
          {t("api_key_fallback_delete_btn")}
        </Button>
      </div>
    </div>
  );
}

function useApiKeyAccounts() {
  const [accounts, setAccounts] = useState<ApiKeyAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadData = () => {
    setLoading(true);
    fetchApiKeyAccounts()
      .then((data) => setAccounts(data))
      .catch((err: unknown) => setError(err instanceof Error ? err.message : "Lỗi tải"))
      .finally(() => setLoading(false));
  };

  useEffect(() => { loadData(); }, []);

  const handleToggle = (id: string) => {
    toggleApiKeyAccount(id).then(loadData).catch((err: unknown) => {
      setError(err instanceof Error ? err.message : "Không thể đổi trạng thái");
    });
  };

  const handleDelete = (id: string) => {
    if (!window.confirm("Bạn có chắc chắn muốn xóa khóa API này?")) return;
    deleteApiKeyAccount(id).then(loadData).catch((err: unknown) => {
      setError(err instanceof Error ? err.message : "Không thể xóa");
    });
  };

  return { accounts, loading, error, loadData, handleToggle, handleDelete };
}

function ApiKeyListContent({
  accounts,
  loading,
  onToggle,
  onDelete,
}: {
  accounts: ApiKeyAccount[];
  loading: boolean;
  onToggle: (id: string) => void;
  onDelete: (id: string) => void;
}) {
  if (loading) {
    return <div style={{ color: "var(--mu)", fontSize: "var(--fs-13)" }}>{t("api_key_fallback_loading")}</div>;
  }
  if (accounts.length === 0) {
    return <div style={{ color: "var(--mu)", fontSize: "var(--fs-13)" }}>{t("api_key_fallback_empty")}</div>;
  }
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
      {accounts.map((acc) => (
        <ApiKeyItem key={acc.id} acc={acc} onToggle={onToggle} onDelete={onDelete} />
      ))}
    </div>
  );
}

export function ApiKeyFallbackCard() {
  const { accounts, loading, error, loadData, handleToggle, handleDelete } = useApiKeyAccounts();
  const [isModalOpen, setIsModalOpen] = useState(false);

  const handleModalSubmit = async (data: CreateApiKeyPayload) => {
    await createApiKeyAccount(data);
    loadData();
  };

  return (
    <Card
      title={
        <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
          <span>{t("api_key_fallback_title")}</span>
          <Chip variant="default">{t("api_key_fallback_token_chip")}</Chip>
        </div>
      }
      subtitle={t("api_key_fallback_desc")}
      actions={
        <Button variant="primary" size="sm" icon={<Plus size={16} />} onClick={() => setIsModalOpen(true)}>
          {t("api_key_fallback_add_btn")}
        </Button>
      }
      style={{ marginTop: "var(--sp-16)" }}
    >
      {error && <div style={{ color: "var(--rd)", fontSize: "var(--fs-13)" }} role="alert">{error}</div>}
      <ApiKeyListContent accounts={accounts} loading={loading} onToggle={handleToggle} onDelete={handleDelete} />
      <AddApiKeyModal isOpen={isModalOpen} onClose={() => setIsModalOpen(false)} onSubmit={handleModalSubmit} />
    </Card>
  );
}

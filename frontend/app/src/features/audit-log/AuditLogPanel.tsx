import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { Input } from "../../ui/components/Input";
import { AuditLogTable } from "./AuditLogTable";
import type { VerifyResult } from "./types";
import { useAuditLog } from "./useAuditLog";

function HeaderActions({
  onVerify,
  verifying,
  onExport,
}: {
  onVerify: () => void;
  verifying: boolean;
  onExport: () => void;
}) {
  return (
    <div style={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: "var(--sp-8)" }}>
      <Button
        variant="secondary"
        size="sm"
        onClick={onVerify}
        disabled={verifying}
        loading={verifying}
        style={{ color: "var(--gr)" }}
      >
        {t("audit_log_verify_btn")}
      </Button>
      <Button
        variant="primary"
        size="sm"
        onClick={onExport}
      >
        {t("audit_log_export_btn")}
      </Button>
    </div>
  );
}

function VerifyStatusBanner({ result }: { result: VerifyResult | null }) {
  if (!result) return null;
  const isOk = result.is_valid;
  return (
    <div
      style={{
        marginBottom: "var(--sp-16)",
        borderRadius: "var(--radius-8)",
        padding: "var(--sp-12)",
        fontSize: "var(--fs-13)",
        fontWeight: 500,
        border: `1px solid ${isOk ? "var(--gr)" : "var(--rd)"}`,
        backgroundColor: isOk ? "var(--grb)" : "var(--rdb)",
        color: isOk ? "var(--gr)" : "var(--rd)",
      }}
    >
      {isOk
        ? t("audit_log_chain_valid", { count: result.records_checked })
        : t("audit_log_chain_invalid", {
            seq: result.broken_at_sequence ?? 0,
          })}
    </div>
  );
}

function FilterBar({
  filter,
  onChange,
  loading,
}: {
  filter: string;
  onChange: (v: string) => void;
  loading: boolean;
}) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-12)", marginBottom: "var(--sp-16)" }}>
      <label style={{ fontSize: "var(--fs-13)", color: "var(--mu)" }}>{t("audit_log_filter_action")}</label>
      <Input
        type="text"
        value={filter}
        onChange={(e) => onChange(e.target.value)}
        placeholder={t("audit_log_filter_placeholder")}
        style={{
          height: "var(--btn-height)",
          padding: "0 12px",
          fontSize: "var(--fs-13)",
          fontFamily: "var(--font-sans)",
          color: "var(--fg)",
          backgroundColor: "var(--sf2)",
          border: "1px solid var(--ln)",
          borderRadius: "var(--radius-8)",
          outline: "none",
        }}
      />
      {loading && <span style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t("loading")}</span>}
    </div>
  );
}

export function AuditLogPanel({ workspaceId }: { workspaceId?: string }) {
  const ws = workspaceId || "default";
  const {
    entries,
    actionFilter,
    setActionFilter,
    loading,
    verifying,
    verifyResult,
    verify,
    exportLogs,
  } = useAuditLog(ws);

  return (
    <div style={{ backgroundColor: "var(--sf)", borderRadius: "var(--radius-10)", border: "1px solid var(--ln)", padding: "var(--sp-16)", marginBottom: "var(--sp-16)" }}>
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)", marginBottom: "var(--sp-16)" }}>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", flexWrap: "wrap", gap: "var(--sp-12)" }}>
          <div>
            <h2 style={{ margin: "0 0 2px 0", fontSize: "var(--fs-16)", fontWeight: 600, color: "var(--fg)" }}>{t("audit_chain_title")}</h2>
            <p style={{ margin: 0, fontSize: "var(--fs-13)", color: "var(--mu)" }}>{t("audit_log_desc")}</p>
          </div>
          <HeaderActions
            onVerify={() => void verify()}
            verifying={verifying}
            onExport={() => void exportLogs()}
          />
        </div>
      </div>

      <VerifyStatusBanner result={verifyResult} />

      <div style={{ fontSize: "var(--fs-12)", color: "var(--or)", backgroundColor: "var(--orb)", border: "1px solid var(--or)", padding: "var(--sp-12)", borderRadius: "var(--radius-8)", marginBottom: "var(--sp-16)", lineHeight: 1.5 }}>
        {t("audit_log_limitation_notice")}
      </div>

      <FilterBar filter={actionFilter} onChange={setActionFilter} loading={loading} />

      <AuditLogTable entries={entries} />
    </div>
  );
}

export default AuditLogPanel;

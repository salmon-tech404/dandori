import { t } from "../../core/i18n";
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
    <div className="flex flex-wrap items-center gap-2">
      <button
        onClick={onVerify}
        disabled={verifying}
        className="rounded bg-emerald-700 px-3 py-1.5 text-xs font-semibold text-white hover:bg-emerald-600 disabled:opacity-50"
      >
        {verifying ? t("audit_log_verifying") : t("audit_log_verify_btn")}
      </button>
      <button
        onClick={onExport}
        className="rounded bg-indigo-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-indigo-500"
      >
        {t("audit_log_export_btn")}
      </button>
    </div>
  );
}

function VerifyStatusBanner({ result }: { result: VerifyResult | null }) {
  if (!result) return null;
  return (
    <div
      className={`mb-6 rounded p-3 text-xs font-medium ${
        result.is_valid
          ? "border border-emerald-800 bg-emerald-950/40 text-emerald-300"
          : "border border-rose-800 bg-rose-950/40 text-rose-300"
      }`}
    >
      {result.is_valid
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
    <div className="mb-4 flex items-center gap-3">
      <label className="text-xs text-neutral-400">{t("audit_log_filter_action")}</label>
      <input
        type="text"
        value={filter}
        onChange={(e) => onChange(e.target.value)}
        placeholder={t("audit_log_filter_placeholder")}
        className="rounded border border-neutral-700 bg-neutral-800 px-3 py-1.5 text-xs text-neutral-100 placeholder-neutral-500"
      />
      {loading && <span className="text-xs text-neutral-500">{t("loading")}</span>}
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
    <div className="rounded-lg border border-neutral-800 bg-neutral-900 p-6">
      <div className="mb-4 flex flex-col gap-2 md:flex-row md:items-center md:justify-between">
        <div>
          <h2 className="text-lg font-bold text-neutral-100">{t("audit_chain_title")}</h2>
          <p className="text-xs text-neutral-400">{t("audit_log_desc")}</p>
        </div>
        <HeaderActions
          onVerify={() => void verify()}
          verifying={verifying}
          onExport={() => void exportLogs()}
        />
      </div>
      <div className="mb-6 rounded border border-amber-900/60 bg-amber-950/30 p-3 text-xs text-amber-300/90">
        {t("audit_log_limitation_notice")}
      </div>
      <VerifyStatusBanner result={verifyResult} />
      <FilterBar filter={actionFilter} onChange={setActionFilter} loading={loading} />
      <AuditLogTable entries={entries} />
    </div>
  );
}

export default AuditLogPanel;

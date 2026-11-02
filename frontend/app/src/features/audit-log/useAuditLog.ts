import { useCallback, useEffect, useState } from "react";
import { exportAuditLogs, fetchAuditLogs, verifyAuditChain } from "./api";
import type { AuditLogEntry, VerifyResult } from "./types";

function triggerDownload(data: unknown, filename: string) {
  const blob = new Blob([JSON.stringify(data, null, 2)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

export function useAuditVerification(ws: string) {
  const [verifying, setVerifying] = useState<boolean>(false);
  const [verifyResult, setVerifyResult] = useState<VerifyResult | null>(null);

  const verify = async () => {
    setVerifying(true);
    try {
      setVerifyResult(await verifyAuditChain(ws));
    } catch {
      setVerifyResult({ is_valid: false, records_checked: 0, reason: "Lỗi kết nối" });
    } finally {
      setVerifying(false);
    }
  };

  const exportLogs = async () => {
    try {
      const data = await exportAuditLogs(ws);
      triggerDownload(data, `audit-log-${ws}-${new Date().toISOString().slice(0, 10)}.json`);
    } catch {
      // Bỏ qua lỗi
    }
  };

  return { verifying, verifyResult, verify, exportLogs };
}

export function useAuditLog(ws: string) {
  const [entries, setEntries] = useState<AuditLogEntry[]>([]);
  const [actionFilter, setActionFilter] = useState<string>("");
  const [loading, setLoading] = useState<boolean>(false);
  const verifier = useAuditVerification(ws);

  const reload = useCallback(async () => {
    setLoading(true);
    try {
      const data = await fetchAuditLogs(ws, {
        action: actionFilter ? actionFilter : undefined,
      });
      setEntries(data);
    } catch {
      setEntries([]);
    } finally {
      setLoading(false);
    }
  }, [ws, actionFilter]);

  useEffect(() => {
    void reload();
  }, [reload]);

  return {
    entries,
    actionFilter,
    setActionFilter,
    loading,
    ...verifier,
  };
}

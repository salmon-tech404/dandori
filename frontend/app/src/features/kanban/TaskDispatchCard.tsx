import { useState } from "react";
import { Link } from "react-router-dom";
import { Play, AlertCircle } from "lucide-react";
import { t } from "../../core/i18n";
import { getSafeErrorMessage } from "../../core/api";
import type { Task } from "../../../../../shared/types";

interface Props {
  task: Task;
  onTaskUpdated?: () => void;
}

function useTaskDispatch(taskId: string, onTaskUpdated?: () => void) {
  const [loading, setLoading] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [isNoAccount, setIsNoAccount] = useState(false);

  const handleDispatch = async () => {
    setLoading(true);
    setErrorMsg(null);
    setIsNoAccount(false);
    try {
      const res = await fetch(`/api/v1/tasks/${taskId}/dispatch`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok) {
        if (data?.error?.code === "no_available_account") {
          setIsNoAccount(true);
        } else {
          setErrorMsg(getSafeErrorMessage(data).userMessage);
        }
        return;
      }
      onTaskUpdated?.();
    } catch (err) {
      setErrorMsg(getSafeErrorMessage(err).userMessage);
    } finally {
      setLoading(false);
    }
  };

  return { loading, errorMsg, isNoAccount, handleDispatch };
}

const dispatchBtnStyle: React.CSSProperties = {
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  gap: "8px",
  padding: "8px 16px",
  background: "var(--bl)",
  color: "white",
  border: "none",
  borderRadius: "6px",
  fontWeight: 600,
  fontSize: "13px",
};

export function TaskDispatchCard({ task, onTaskUpdated }: Props) {
  const { loading, errorMsg, isNoAccount, handleDispatch } = useTaskDispatch(task.id, onTaskUpdated);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
      {isNoAccount && <NoAccountAlert />}
      {errorMsg && <div role="alert" style={{ color: "var(--rd)", fontSize: "12px" }}>{errorMsg}</div>}
      <button
        type="button"
        data-testid="btn-dispatch-task"
        disabled={loading}
        onClick={handleDispatch}
        style={{ ...dispatchBtnStyle, cursor: loading ? "not-allowed" : "pointer" }}
      >
        <Play size={14} />
        <span>{loading ? t("btn_dispatching") : t("btn_dispatch_task")}</span>
      </button>
    </div>
  );
}

function NoAccountAlert() {
  return (
    <div
      data-testid="no-account-alert"
      style={{
        padding: "10px 12px",
        background: "var(--amb, #fffbeb)",
        border: "1px solid var(--am, #f59e0b)",
        borderRadius: "6px",
        fontSize: "12px",
        color: "var(--fg)",
        display: "flex",
        flexDirection: "column",
        gap: "6px",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "6px", fontWeight: 600 }}>
        <AlertCircle size={14} color="var(--am, #f59e0b)" />
        <span>{t("dispatch_no_account_notice")}</span>
      </div>
      <Link
        to="/agents"
        style={{ color: "var(--bl)", textDecoration: "underline", fontWeight: 600, width: "fit-content" }}
      >
        {t("go_to_agents")}
      </Link>
    </div>
  );
}

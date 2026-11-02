import { FC, useEffect, useState } from "react";
import { t } from "../../core/i18n";
import { TaskAssignment } from "./types";
import { useTaskAssignment } from "./useLeadWorker";
import { AgentSelectField } from "./AgentSelectField";

interface FieldGroupProps {
  data: TaskAssignment;
  onUpdate: (field: "lead" | "worker", val: string) => void;
}

const FieldGroup: FC<FieldGroupProps> = ({ data, onUpdate }) => (
  <>
    <AgentSelectField
      label={t("lead_agent_label")}
      role="lead"
      currentValue={data.task_lead_agent}
      effectiveValue={data.effective_lead_agent}
      isOverridden={data.is_lead_overridden}
      readyProviders={data.ready_providers}
      onChange={(val) => onUpdate("lead", val)}
    />
    <AgentSelectField
      label={t("worker_agent_label")}
      role="worker"
      currentValue={data.task_worker_agent}
      effectiveValue={data.effective_worker_agent}
      isOverridden={data.is_worker_overridden}
      readyProviders={data.ready_providers}
      onChange={(val) => onUpdate("worker", val)}
    />
  </>
);

export const TaskAssignmentSection: FC<{ context?: Record<string, unknown> }> = ({ context }) => {
  const taskId = (context?.taskId as string) || (context?.id as string) || "";
  const taskTitle = (context?.title as string) || "";
  const taskDesc = (context?.description as string) || "";
  const { fetchAssignment, assignAgent, dispatch, loading, error, setError } = useTaskAssignment(taskId);
  const [data, setData] = useState<TaskAssignment | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);

  useEffect(() => { if (taskId) fetchAssignment().then(setData); }, [taskId, fetchAssignment]);
  if (!taskId) return null;
  if (!data) return <div>{t("empty_task_assignment")}</div>;

  const handleUpdate = async (field: "lead" | "worker", val: string) => {
    const lead = field === "lead" ? val : data.task_lead_agent;
    const worker = field === "worker" ? val : data.task_worker_agent;
    if (await assignAgent(taskTitle, taskDesc, lead, worker)) {
      const refreshed = await fetchAssignment();
      if (refreshed) setData(refreshed);
    }
  };

  const handleDispatch = async () => {
    setError(null);
    setSuccessMsg(null);
    if (await dispatch()) {
      setSuccessMsg(t("dispatch_success"));
      setTimeout(() => setSuccessMsg(null), 3000);
    }
  };

  return (
    <div style={{ borderTop: "1px solid var(--border)", paddingTop: "var(--space-md)", marginTop: "var(--space-md)" }}>
      <h4>{t("task_assignment_title")}</h4>
      {error && <div role="alert" style={{ color: "var(--danger)", padding: "var(--space-sm)", background: "var(--surface)", border: "1px solid var(--danger)", borderRadius: "var(--radius-sm)", marginBottom: "var(--space-sm)" }}>{error}</div>}
      {successMsg && <div style={{ color: "var(--success)", marginBottom: "var(--space-sm)" }}>{successMsg}</div>}
      <FieldGroup data={data} onUpdate={handleUpdate} />
      <button type="button" disabled={loading} onClick={handleDispatch} style={{ width: "100%", padding: "var(--space-sm)", background: "var(--primary)", color: "var(--surface)", border: "none", borderRadius: "var(--radius-sm)", cursor: loading ? "not-allowed" : "pointer" }}>
        {t("btn_dispatch")}
      </button>
    </div>
  );
};

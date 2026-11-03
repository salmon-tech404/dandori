import { FC, useEffect, useState } from "react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
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

function useAssignmentController(taskId: string, taskTitle: string, taskDesc: string) {
  const { fetchAssignment, assignAgent, dispatch, loading, error, setError } = useTaskAssignment(taskId);
  const [data, setData] = useState<TaskAssignment | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);

  useEffect(() => {
    if (taskId) fetchAssignment().then(setData);
  }, [taskId, fetchAssignment]);

  const handleUpdate = async (field: "lead" | "worker", val: string) => {
    if (!data) return;
    const lead = field === "lead" ? val : (data.task_lead_agent || "");
    const worker = field === "worker" ? val : (data.task_worker_agent || "");
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

  return { data, loading, error, successMsg, handleUpdate, handleDispatch };
}

export const TaskAssignmentSection: FC<{ context?: Record<string, unknown> }> = ({ context }) => {
  const taskId = (context?.taskId as string) || (context?.id as string) || "";
  const taskTitle = (context?.title as string) || "";
  const taskDesc = (context?.description as string) || "";
  const ctrl = useAssignmentController(taskId, taskTitle, taskDesc);

  if (!taskId) return null;
  if (!ctrl.data) return <div>{t("empty_task_assignment")}</div>;

  return (
    <div style={{ borderTop: "1px solid var(--ln)", paddingTop: "var(--sp-16)", marginTop: "var(--sp-16)" }}>
      <h4>{t("task_assignment_title")}</h4>
      {ctrl.error && <div role="alert" style={{ color: "var(--rd)", padding: "var(--sp-8)", background: "var(--sf)", border: "1px solid var(--rd)", borderRadius: "var(--radius-6)", marginBottom: "var(--sp-8)" }}>{ctrl.error}</div>}
      {ctrl.successMsg && <div style={{ color: "var(--gr)", marginBottom: "var(--sp-8)" }}>{ctrl.successMsg}</div>}
      <FieldGroup data={ctrl.data} onUpdate={ctrl.handleUpdate} />
      <Button
        variant="primary"
        type="button"
        disabled={ctrl.loading}
        loading={ctrl.loading}
        onClick={ctrl.handleDispatch}
        style={{ width: "100%" }}
      >
        {t("btn_dispatch")}
      </Button>
    </div>
  );
};

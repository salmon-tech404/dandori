import { FC, FormEvent, useState, ReactNode } from "react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { Input } from "../../ui/components/Input";
import { getSafeErrorMessage } from "../../core/api";

export interface CreateTaskModalProps {
  isOpen: boolean;
  onClose: () => void;
  projectId: string;
  onCreated?: () => void;
}

function ModalBackdrop({ children }: { children: ReactNode }) {
  return (
    <div
      data-testid="create-task-modal"
      style={{
        position: "fixed",
        inset: 0,
        backgroundColor: "rgba(0,0,0,0.5)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 50,
      }}
    >
      <div
        style={{
          width: "100%",
          maxWidth: "480px",
          backgroundColor: "var(--sf)",
          borderRadius: "var(--radius-10)",
          border: "1px solid var(--ln)",
          padding: "var(--sp-24)",
        }}
      >
        {children}
      </div>
    </div>
  );
}

function ModalErrorBanner({ error }: { error: string | null }) {
  if (!error) return null;
  return (
    <div
      role="alert"
      data-testid="create-task-error"
      style={{
        padding: "var(--sp-8)",
        marginBottom: "var(--sp-12)",
        color: "var(--rd)",
        backgroundColor: "var(--rdb)",
        borderRadius: "var(--radius-6)",
        fontSize: "var(--fs-13)",
      }}
    >
      {error}
    </div>
  );
}

function TaskTitleField({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  return (
    <div>
      <label
        htmlFor="create-task-title-input"
        style={{ display: "block", marginBottom: "var(--sp-4)", fontSize: "var(--fs-13)", fontWeight: 500 }}
      >
        {t("task_title_label")}
      </label>
      <Input
        id="create-task-title-input"
        data-testid="create-task-title-input"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={t("task_title_placeholder")}
        required
      />
    </div>
  );
}

const descTextareaStyle = {
  width: "100%",
  padding: "var(--sp-8)",
  borderRadius: "var(--radius-6)",
  border: "1px solid var(--ln)",
  backgroundColor: "var(--sf2)",
  color: "var(--fg)",
  fontSize: "var(--fs-14)",
  fontFamily: "inherit",
  resize: "vertical" as const,
  minHeight: "80px",
  boxSizing: "border-box" as const,
};

function TaskDescField({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  return (
    <div>
      <label
        htmlFor="create-task-desc-input"
        style={{ display: "block", marginBottom: "var(--sp-4)", fontSize: "var(--fs-13)", fontWeight: 500 }}
      >
        {t("task_desc_label")}
      </label>
      <textarea
        id="create-task-desc-input"
        data-testid="create-task-desc-input"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={t("task_desc_placeholder")}
        style={descTextareaStyle}
      />
    </div>
  );
}

function TaskFormActions({
  loading,
  onClose,
}: {
  loading: boolean;
  onClose: () => void;
}) {
  return (
    <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--sp-8)" }}>
      <Button variant="ghost" type="button" onClick={onClose} disabled={loading}>
        {t("btn_cancel")}
      </Button>
      <Button
        variant="primary"
        type="submit"
        data-testid="btn-submit-create-task"
        disabled={loading}
      >
        {loading ? t("btn_creating_task") : t("btn_submit_create_task")}
      </Button>
    </div>
  );
}

async function sendCreateTaskRequest(projectId: string, title: string, description: string) {
  const res = await fetch(`/api/v1/projects/${projectId}/tasks`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ title: title.trim(), description: description.trim() }),
  });
  if (!res.ok) {
    const data = await res.json().catch(() => ({}));
    throw new Error(data.error?.message || `Tạo task thất bại: ${res.status}`);
  }
}

function useCreateTaskForm(projectId: string, onClose: () => void, onCreated?: () => void) {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!title.trim()) {
      setError(t("task_title_required"));
      return;
    }
    setLoading(true);
    setError(null);
    try {
      await sendCreateTaskRequest(projectId, title, description);
      onCreated?.();
      onClose();
    } catch (err) {
      setError(getSafeErrorMessage(err).userMessage);
    } finally {
      setLoading(false);
    }
  };

  return { title, setTitle, description, setDescription, loading, error, handleSubmit };
}

export const CreateTaskModal: FC<CreateTaskModalProps> = ({
  isOpen,
  onClose,
  projectId,
  onCreated,
}) => {
  const form = useCreateTaskForm(projectId, onClose, onCreated);

  if (!isOpen) return null;

  return (
    <ModalBackdrop>
      <h3 style={{ margin: "0 0 var(--sp-16) 0", fontSize: "var(--fs-18)", fontWeight: 600 }}>
        {t("create_task_modal_title")}
      </h3>
      <ModalErrorBanner error={form.error} />
      <form onSubmit={form.handleSubmit} noValidate>
        <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)", marginBottom: "var(--sp-16)" }}>
          <TaskTitleField value={form.title} onChange={form.setTitle} />
          <TaskDescField value={form.description} onChange={form.setDescription} />
        </div>
        <TaskFormActions loading={form.loading} onClose={onClose} />
      </form>
    </ModalBackdrop>
  );
};

import React, { useState } from "react";
import { t } from "../../core/i18n";
import { Modal } from "../../ui/components/Modal";
import { Button } from "../../ui/components/Button";
import { Checkbox } from "../../ui/components/Checkbox";
import { useWorktree, executeCleanup } from "./useWorktree";
import type { WorktreeStatus } from "./types";

interface Props {
  context?: {
    taskId?: string;
  };
}

const emptyBoxStyle: React.CSSProperties = {
  padding: "16px",
  background: "var(--sf)",
  border: "1px dashed var(--ln)",
  borderRadius: "8px",
  color: "var(--mu)",
  fontSize: "13px",
};

const cardStyle: React.CSSProperties = {
  display: "flex",
  flexDirection: "column",
  gap: "10px",
  padding: "14px",
  background: "var(--sf)",
  border: "1px solid var(--ln)",
  borderRadius: "8px",
};

const codeStyle: React.CSSProperties = {
  padding: "2px 6px",
  background: "var(--sf2)",
  borderRadius: "4px",
  fontSize: "12px",
  color: "var(--fg)",
};

const dirtyChipStyle: React.CSSProperties = {
  padding: "6px 10px",
  background: "var(--orb)",
  color: "var(--or)",
  borderRadius: "6px",
  fontSize: "12px",
};

function WorktreeEmptyView() {
  return <div style={emptyBoxStyle}>{t("worktree_empty")}</div>;
}

function WorktreeDetailsView({
  status,
  onOpenCleanup,
}: {
  status: WorktreeStatus;
  onOpenCleanup: () => void;
}) {
  return (
    <div style={cardStyle}>
      <div>
        <span style={{ fontSize: "12px", color: "var(--mu)" }}>{t("worktree_branch_label")}</span>
        <div style={{ marginTop: "2px" }}>
          <code style={codeStyle}>{status.branch || "—"}</code>
        </div>
      </div>
      <div>
        <span style={{ fontSize: "12px", color: "var(--mu)" }}>{t("worktree_path_label")}</span>
        <div style={{ fontSize: "12px", color: "var(--mu)", wordBreak: "break-all", marginTop: "2px" }}>
          {status.workdir_path || "—"}
        </div>
      </div>
      {status.is_dirty && (
        <div style={dirtyChipStyle}>
          {t("worktree_dirty_warning", { count: status.dirty_files.length })}
        </div>
      )}
      <div style={{ marginTop: "6px" }}>
        <Button type="button" variant="danger" size="sm" onClick={onOpenCleanup}>
          {t("worktree_btn_cleanup")}
        </Button>
      </div>
    </div>
  );
}

function DialogButtons({
  onCancel,
  onConfirm,
}: {
  onCancel: () => void;
  onConfirm: () => void;
}) {
  return (
    <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px" }}>
      <Button type="button" variant="secondary" size="sm" onClick={onCancel}>
        {t("worktree_btn_cancel")}
      </Button>
      <Button type="button" variant="danger" size="sm" onClick={onConfirm}>
        {t("worktree_btn_confirm")}
      </Button>
    </div>
  );
}

function ForceCheckbox({ force, onChange }: { force: boolean; onChange: (v: boolean) => void }) {
  return (
    <label style={{ display: "flex", alignItems: "center", gap: "8px", fontSize: "12px", color: "var(--or)", margin: "10px 0" }}>
      <Checkbox checked={force} onChange={(e) => onChange(e.target.checked)} />
      {t("worktree_force_checkbox")}
    </label>
  );
}

function CleanupDialog({
  isDirty,
  force,
  onForceChange,
  onConfirm,
  onCancel,
}: {
  isDirty: boolean;
  force: boolean;
  onForceChange: (checked: boolean) => void;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  const footer = <DialogButtons onCancel={onCancel} onConfirm={onConfirm} />;

  return (
    <Modal open onClose={onCancel} title={t("worktree_cleanup_confirm_title")} footer={footer} size="sm">
      <p style={{ fontSize: "13px", color: "var(--mu)", margin: "0 0 10px 0" }}>
        {t("worktree_cleanup_confirm_msg")}
      </p>
      {isDirty && <ForceCheckbox force={force} onChange={onForceChange} />}
    </Modal>
  );
}

function SectionContent({
  status,
  loading,
  error,
  onOpenCleanup,
}: {
  status: WorktreeStatus | null;
  loading: boolean;
  error: string | null;
  onOpenCleanup: () => void;
}) {
  if (loading) {
    return <div style={{ fontSize: "13px", color: "var(--mu)" }}>{t("loading")}</div>;
  }
  if (error) {
    return <div style={{ fontSize: "13px", color: "var(--rd)" }}>{error}</div>;
  }
  if (!status?.exists) {
    return <WorktreeEmptyView />;
  }
  return <WorktreeDetailsView status={status} onOpenCleanup={onOpenCleanup} />;
}

export function WorktreeSection({ context }: Props) {
  const taskId = context?.taskId;
  const { status, loading, error, refresh } = useWorktree(taskId);
  const [showConfirm, setShowConfirm] = useState(false);
  const [force, setForce] = useState(false);
  const [actionMsg, setActionMsg] = useState<string | null>(null);

  const onConfirm = async () => {
    if (!taskId) return;
    try {
      await executeCleanup(taskId, force);
      setShowConfirm(false);
      setActionMsg(t("worktree_cleanup_success"));
      await refresh();
    } catch (err) {
      setActionMsg(err instanceof Error ? err.message : String(err));
    }
  };

  return (
    <div style={{ marginBottom: "16px" }}>
      <div style={{ fontSize: "13px", fontWeight: 600, color: "var(--fg)", marginBottom: "8px" }}>
        {t("worktree_title")}
      </div>
      {actionMsg && <div style={{ fontSize: "12px", color: "var(--gr)", marginBottom: "8px" }}>{actionMsg}</div>}
      <SectionContent status={status} loading={loading} error={error} onOpenCleanup={() => { setForce(false); setShowConfirm(true); }} />
      {showConfirm && <CleanupDialog isDirty={status?.is_dirty ?? false} force={force} onForceChange={setForce} onConfirm={onConfirm} onCancel={() => setShowConfirm(false)} />}
    </div>
  );
}

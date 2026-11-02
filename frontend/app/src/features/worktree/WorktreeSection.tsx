import React, { useState } from "react";
import { t } from "../../core/i18n";
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
  background: "var(--sf-subtle)",
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

const btnCleanupStyle: React.CSSProperties = {
  padding: "6px 12px",
  background: "var(--sf)",
  border: "1px solid var(--rd)",
  color: "var(--rd)",
  borderRadius: "6px",
  fontSize: "12px",
  cursor: "pointer",
};

const modalOverlayStyle: React.CSSProperties = {
  position: "fixed",
  top: 0,
  left: 0,
  right: 0,
  bottom: 0,
  background: "rgba(0,0,0,0.4)",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  zIndex: 1000,
};

const modalBoxStyle: React.CSSProperties = {
  background: "var(--sf)",
  border: "1px solid var(--ln)",
  borderRadius: "8px",
  padding: "20px",
  width: "360px",
  maxWidth: "90%",
};

const btnCancelStyle: React.CSSProperties = {
  padding: "6px 12px",
  border: "1px solid var(--ln)",
  background: "var(--sf)",
  color: "var(--fg)",
  borderRadius: "6px",
  fontSize: "12px",
  cursor: "pointer",
};

const btnConfirmStyle: React.CSSProperties = {
  padding: "6px 12px",
  background: "var(--rd)",
  color: "var(--acf)",
  border: "none",
  borderRadius: "6px",
  fontSize: "12px",
  cursor: "pointer",
};

function WorktreeEmptyView() {
  return <div style={emptyBoxStyle}>{t("worktree_empty")}</div>;
}

function BranchDisplay({ branch }: { branch: string | null }) {
  return (
    <div>
      <span style={{ fontSize: "12px", color: "var(--mu)" }}>{t("worktree_branch_label")}</span>
      <div style={{ marginTop: "2px" }}>
        <code style={codeStyle}>{branch || "—"}</code>
      </div>
    </div>
  );
}

function PathDisplay({ path }: { path: string | null }) {
  return (
    <div>
      <span style={{ fontSize: "12px", color: "var(--mu)" }}>{t("worktree_path_label")}</span>
      <div style={{ fontSize: "12px", color: "var(--mu)", wordBreak: "break-all", marginTop: "2px" }}>
        {path || "—"}
      </div>
    </div>
  );
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
      <BranchDisplay branch={status.branch} />
      <PathDisplay path={status.workdir_path} />
      {status.is_dirty && (
        <div style={dirtyChipStyle}>
          {t("worktree_dirty_warning", { count: status.dirty_files.length })}
        </div>
      )}
      <div style={{ marginTop: "6px" }}>
        <button type="button" onClick={onOpenCleanup} style={btnCleanupStyle}>
          {t("worktree_btn_cleanup")}
        </button>
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
    <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px", marginTop: "16px" }}>
      <button type="button" onClick={onCancel} style={btnCancelStyle}>
        {t("worktree_btn_cancel")}
      </button>
      <button type="button" onClick={onConfirm} style={btnConfirmStyle}>
        {t("worktree_btn_confirm")}
      </button>
    </div>
  );
}

function ForceCheckbox({ force, onChange }: { force: boolean; onChange: (v: boolean) => void }) {
  return (
    <label style={{ display: "flex", alignItems: "center", gap: "8px", fontSize: "12px", color: "var(--or)", margin: "10px 0" }}>
      <input type="checkbox" checked={force} onChange={(e) => onChange(e.target.checked)} />
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
  return (
    <div style={modalOverlayStyle}>
      <div style={modalBoxStyle}>
        <div style={{ fontSize: "14px", fontWeight: 600, color: "var(--fg)" }}>
          {t("worktree_cleanup_confirm_title")}
        </div>
        <p style={{ fontSize: "13px", color: "var(--mu)", margin: "10px 0" }}>
          {t("worktree_cleanup_confirm_msg")}
        </p>
        {isDirty && <ForceCheckbox force={force} onChange={onForceChange} />}
        <DialogButtons onCancel={onCancel} onConfirm={onConfirm} />
      </div>
    </div>
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

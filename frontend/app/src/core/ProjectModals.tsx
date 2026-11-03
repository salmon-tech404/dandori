import { FC, FormEvent, useEffect, useMemo, useState, ReactNode } from "react";
import { FolderOpen } from "lucide-react";
import { t } from "./i18n";
import { useProject } from "./project";
import { Button } from "../ui/components/Button";
import { Input } from "../ui/components/Input";
import { apiClient, getSafeErrorMessage } from "./api";
import { DirectoryPicker } from "./DirectoryPicker";
import { BranchSelect } from "./BranchSelect";

export interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
}

function ModalBackdrop({ children }: { children: ReactNode }) {
  return (
    <div
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
      data-testid="modal-error-alert"
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

function useWorkspaceModalLogic(onClose: () => void) {
  const { createWorkspace } = useProject();
  const [name, setName] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!name.trim() || loading) return;
    setLoading(true);
    setError(null);
    try {
      await createWorkspace(name.trim());
      setName("");
      onClose();
    } catch (err) {
      setError(getSafeErrorMessage(err).userMessage);
    } finally {
      setLoading(false);
    }
  };

  return { name, setName, loading, error, handleSubmit };
}

export const CreateWorkspaceModal: FC<ModalProps> = ({ isOpen, onClose }) => {
  const { name, setName, loading, error, handleSubmit } =
    useWorkspaceModalLogic(onClose);

  if (!isOpen) return null;

  return (
    <ModalBackdrop>
      <h2 style={{ margin: "0 0 var(--sp-16) 0", fontSize: "var(--fs-18)" }}>
        {t("workspace_create_title")}
      </h2>
      <ModalErrorBanner error={error} />
      <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)" }}>
        <Input
          label={t("workspace_name_label")}
          type="text"
          required
          value={name}
          onChange={(e) => setName(e.target.value)}
          disabled={loading}
        />
        <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--sp-8)", marginTop: "var(--sp-8)" }}>
          <Button type="button" variant="ghost" onClick={onClose} disabled={loading}>
            {t("btn_cancel")}
          </Button>
          <Button type="submit" variant="primary" loading={loading} disabled={loading}>
            {t("btn_create_workspace")}
          </Button>
        </div>
      </form>
    </ModalBackdrop>
  );
};

/** Nhánh để chọn: nhánh của kho đang chọn + nhánh đã dùng ở các dự án khác (lưu trong hệ thống) + "main". */
function useBranchOptions(repoPath: string, projects: Array<{ default_branch?: string }> | undefined, current: string) {
  const [fromRepo, setFromRepo] = useState<string[]>([]);
  const [added, setAdded] = useState<string[]>([]);

  useEffect(() => {
    const path = repoPath.trim();
    if (!path) {
      setFromRepo([]);
      return undefined;
    }
    let cancelled = false;
    const timer = setTimeout(() => {
      apiClient<{ branches: string[] }>(`/api/v1/fs/git-branches?path=${encodeURIComponent(path)}`)
        .then((res) => { if (!cancelled) setFromRepo(res.branches); })
        .catch(() => { if (!cancelled) setFromRepo([]); });
    }, 400);
    return () => { cancelled = true; clearTimeout(timer); };
  }, [repoPath]);

  const options = useMemo(() => {
    const used = (projects || []).map((p) => p.default_branch).filter((b): b is string => Boolean(b));
    return Array.from(new Set(["main", ...fromRepo, ...used, ...added, current].filter(Boolean)));
  }, [fromRepo, projects, added, current]);

  return { options, add: (b: string) => setAdded((prev) => [...prev, b]) };
}

function useProjectModalLogic(onClose: () => void) {
  const { createProject, currentWorkspace, projects } = useProject();
  const [name, setName] = useState("");
  const [repoPath, setRepoPath] = useState("");
  const [defaultBranch, setDefaultBranch] = useState("main");
  const branchOptions = useBranchOptions(repoPath, projects, defaultBranch);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !repoPath.trim() || loading) return;
    setLoading(true);
    setError(null);
    try {
      await createProject(name.trim(), repoPath.trim(), defaultBranch.trim() || "main");
      setName("");
      setRepoPath("");
      onClose();
    } catch (err) {
      setError(getSafeErrorMessage(err).userMessage);
    } finally {
      setLoading(false);
    }
  };

  return {
    name,
    setName,
    repoPath,
    setRepoPath,
    defaultBranch,
    setDefaultBranch,
    branchOptions: branchOptions.options,
    addBranchOption: branchOptions.add,
    loading,
    error,
    handleSubmit,
    currentWorkspace,
  };
}

type ProjectModalLogic = ReturnType<typeof useProjectModalLogic>;

function RepoPathField({ logic }: { logic: ProjectModalLogic }) {
  const [picking, setPicking] = useState(false);
  return (
    <div style={{ display: "flex", alignItems: "flex-end", gap: "var(--sp-8)" }}>
      <Input
        label={t("repo_path_label")}
        type="text"
        required
        data-testid="project-repo-input"
        value={logic.repoPath}
        placeholder={t("repo_path_placeholder")}
        onChange={(e) => logic.setRepoPath(e.target.value)}
        disabled={logic.loading}
      />
      <Button type="button" variant="secondary" size="icon" aria-label={t("pick_folder")} title={t("pick_folder")} disabled={logic.loading} onClick={() => setPicking(true)} icon={<FolderOpen size={16} />} />
      {picking && (
        <DirectoryPicker
          initialPath={logic.repoPath.trim()}
          onClose={() => setPicking(false)}
          onSelect={(p) => { logic.setRepoPath(p); setPicking(false); }}
        />
      )}
    </div>
  );
}

function ProjectModalForm({ logic, onClose }: { logic: ProjectModalLogic; onClose: () => void }) {
  return (
    <form onSubmit={logic.handleSubmit} style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)" }}>
      {!logic.currentWorkspace && (
        <div role="alert" data-testid="no-workspace-note" style={{ padding: "var(--sp-8)", color: "var(--rd)", backgroundColor: "var(--rdb)", borderRadius: "var(--radius-6)", fontSize: "var(--fs-13)" }}>
          {t("project_need_workspace")}
        </div>
      )}
      <Input
        label={t("project_name_label")}
        type="text"
        required
        data-testid="project-name-input"
        value={logic.name}
        onChange={(e) => logic.setName(e.target.value)}
        disabled={logic.loading}
      />
      <RepoPathField logic={logic} />
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-4)" }}>
        <span style={{ fontSize: "var(--fs-13)", fontWeight: 500, color: "var(--mu)" }}>{t("default_branch_label")}</span>
        <BranchSelect value={logic.defaultBranch} options={logic.branchOptions} onChange={logic.setDefaultBranch} onAddOption={logic.addBranchOption} disabled={logic.loading} />
        <span style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t("branch_from_repo_hint")}</span>
      </div>
      <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--sp-8)", marginTop: "var(--sp-8)" }}>
        <Button type="button" variant="ghost" onClick={onClose} disabled={logic.loading}>
          {t("btn_cancel")}
        </Button>
        <Button type="submit" variant="primary" loading={logic.loading} disabled={logic.loading || !logic.currentWorkspace}>
          {t("btn_create_project")}
        </Button>
      </div>
    </form>
  );
}

export const CreateProjectModal: FC<ModalProps> = ({ isOpen, onClose }) => {
  const logic = useProjectModalLogic(onClose);
  if (!isOpen) return null;

  return (
    <ModalBackdrop>
      <h2 style={{ margin: "0 0 var(--sp-16) 0", fontSize: "var(--fs-18)" }}>
        {t("project_create_title")}
      </h2>
      <ModalErrorBanner error={logic.error} />
      <ProjectModalForm logic={logic} onClose={onClose} />
    </ModalBackdrop>
  );
};

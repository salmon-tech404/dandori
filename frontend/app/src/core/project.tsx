import {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  ReactNode,
  FC,
} from "react";
import { apiClient, getSafeErrorMessage } from "./api";

export interface Workspace {
  id: string;
  name: string;
  created_at?: string;
}

export interface Project {
  id: string;
  workspace_id: string;
  name: string;
  repo_path: string;
  default_branch: string;
  created_at?: string;
}

export interface ProjectContextType {
  workspaces: Workspace[];
  currentWorkspace: Workspace | null;
  projects: Project[];
  currentProject: Project | null;
  loading: boolean;
  error: string | null;
  selectWorkspace: (wsId: string) => Promise<void>;
  selectProject: (projId: string) => void;
  createWorkspace: (name: string) => Promise<Workspace>;
  createProject: (
    name: string,
    repoPath: string,
    defaultBranch?: string
  ) => Promise<Project>;
  refresh: () => Promise<void>;
}

export const ProjectContext = createContext<ProjectContextType | null>(null);

const WS_STORAGE_KEY = "dandori_last_workspace_id";
const PROJ_STORAGE_KEY = "dandori_last_project_id";

function useProjectsLoader(
  setProjects: (p: Project[]) => void,
  setCurrentProject: (p: Project | null) => void
) {
  return useCallback(
    async (wsId: string) => {
      try {
        const res = await apiClient<{ projects: Project[] }>(
          `/api/v1/workspaces/${wsId}/projects`
        );
        setProjects(res.projects);
        const savedId = localStorage.getItem(PROJ_STORAGE_KEY);
        const match = res.projects.find((p) => p.id === savedId);
        const chosen = match || res.projects[0] || null;
        setCurrentProject(chosen);
        if (chosen) localStorage.setItem(PROJ_STORAGE_KEY, chosen.id);
      } catch {
        setProjects([]);
        setCurrentProject(null);
      }
    },
    [setProjects, setCurrentProject]
  );
}

function pickWorkspace(workspaces: Workspace[]): Workspace | null {
  const savedWsId = localStorage.getItem(WS_STORAGE_KEY);
  const match = workspaces.find((w) => w.id === savedWsId);
  return match || workspaces[0] || null;
}

function useWorkspaceRefresh(
  setWorkspaces: (ws: Workspace[]) => void,
  setCurrentWorkspace: (ws: Workspace | null) => void,
  loadProjects: (wsId: string) => Promise<void>,
  setError: (err: string | null) => void,
  setLoading: (l: boolean) => void
) {
  return useCallback(async () => {
    setLoading(true);
    try {
      const res = await apiClient<{ workspaces: Workspace[] }>("/api/v1/workspaces");
      setWorkspaces(res.workspaces);
      const chosen = pickWorkspace(res.workspaces);
      setCurrentWorkspace(chosen);
      if (chosen) {
        localStorage.setItem(WS_STORAGE_KEY, chosen.id);
        await loadProjects(chosen.id);
      }
    } catch (err) {
      setError(getSafeErrorMessage(err).userMessage);
    } finally {
      setLoading(false);
    }
  }, [setWorkspaces, setCurrentWorkspace, loadProjects, setError, setLoading]);
}

function useWorkspaceState() {
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [currentWorkspace, setCurrentWorkspace] = useState<Workspace | null>(null);
  const [projects, setProjects] = useState<Project[]>([]);
  const [currentProject, setCurrentProject] = useState<Project | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadProjects = useProjectsLoader(setProjects, setCurrentProject);
  const refresh = useWorkspaceRefresh(
    setWorkspaces,
    setCurrentWorkspace,
    loadProjects,
    setError,
    setLoading
  );

  return {
    workspaces,
    setWorkspaces,
    currentWorkspace,
    setCurrentWorkspace,
    projects,
    setProjects,
    currentProject,
    setCurrentProject,
    loading,
    error,
    setError,
    refresh,
    loadProjects,
  };
}

type WorkspaceState = ReturnType<typeof useWorkspaceState>;

function useWorkspaceMutations(state: WorkspaceState) {
  const selectWorkspace = useCallback(
    async (wsId: string) => {
      const ws = state.workspaces.find((w) => w.id === wsId);
      if (ws) {
        state.setCurrentWorkspace(ws);
        localStorage.setItem(WS_STORAGE_KEY, ws.id);
        await state.loadProjects(ws.id);
      }
    },
    [state]
  );

  const createWorkspace = useCallback(
    async (name: string) => {
      state.setError(null);
      try {
        const res = await apiClient<{ workspace: Workspace }>("/api/v1/workspaces", {
          method: "POST",
          body: JSON.stringify({ name }),
        });
        state.setWorkspaces((prev) => [...(prev || []), res.workspace]);
        state.setCurrentWorkspace(res.workspace);
        localStorage.setItem(WS_STORAGE_KEY, res.workspace.id);
        state.setProjects([]);
        state.setCurrentProject(null);
        return res.workspace;
      } catch (err) {
        state.setError(getSafeErrorMessage(err).userMessage);
        throw err;
      }
    },
    [state]
  );

  return { selectWorkspace, createWorkspace };
}

function useProjectMutations(state: WorkspaceState) {
  const selectProject = useCallback(
    (projId: string) => {
      const proj = state.projects.find((p) => p.id === projId);
      if (proj) {
        state.setCurrentProject(proj);
        localStorage.setItem(PROJ_STORAGE_KEY, proj.id);
      }
    },
    [state]
  );

  const createProject = useCallback(
    async (name: string, repoPath: string, defaultBranch = "main") => {
      if (!state.currentWorkspace) throw new Error("Chưa chọn không gian làm việc");
      state.setError(null);
      try {
        const res = await apiClient<{ project: Project }>(
          `/api/v1/workspaces/${state.currentWorkspace.id}/projects`,
          {
            method: "POST",
            body: JSON.stringify({ name, repo_path: repoPath, default_branch: defaultBranch }),
          }
        );
        state.setProjects((prev) => [...(prev || []), res.project]);
        state.setCurrentProject(res.project);
        localStorage.setItem(PROJ_STORAGE_KEY, res.project.id);
        return res.project;
      } catch (err) {
        state.setError(getSafeErrorMessage(err).userMessage);
        throw err;
      }
    },
    [state]
  );

  return { selectProject, createProject };
}

export const ProjectProvider: FC<{ children: ReactNode }> = ({ children }) => {
  const state = useWorkspaceState();
  const wsMutations = useWorkspaceMutations(state);
  const projMutations = useProjectMutations(state);
  const { refresh } = state;

  useEffect(() => {
    refresh();
  }, [refresh]);

  return (
    <ProjectContext.Provider
      value={{
        workspaces: state.workspaces,
        currentWorkspace: state.currentWorkspace,
        projects: state.projects,
        currentProject: state.currentProject,
        loading: state.loading,
        error: state.error,
        refresh: state.refresh,
        ...wsMutations,
        ...projMutations,
      }}
    >
      {children}
    </ProjectContext.Provider>
  );
};

const DEFAULT_PROJECT_CONTEXT: ProjectContextType = {
  workspaces: [],
  currentWorkspace: null,
  projects: [],
  currentProject: null,
  loading: false,
  error: null,
  createWorkspace: async () => ({} as Workspace),
  createProject: async () => ({} as Project),
  selectWorkspace: async () => {},
  selectProject: () => {},
  refresh: async () => {},
};

export function useProject(): ProjectContextType {
  const context = useContext(ProjectContext);
  if (!context) {
    return DEFAULT_PROJECT_CONTEXT;
  }
  return context;
}

import { FC, ReactNode, useState, useContext } from "react";
import { Link, useLocation } from "react-router-dom";
import { Sun, Moon, Plus } from "lucide-react";
import { t } from "./i18n";
import { useFeatures, SCREEN_PRIMARY_MODULE } from "./useFeatures";
import { Slot } from "./Slot";
import { Button } from "../ui/components/Button";
import { Table, Column } from "../ui/components/Table";
import { StatusBadge } from "../ui/components/StatusBadge";
import { Skeleton } from "../ui/components/Skeleton";
import { ProjectContext } from "./project";
import { AppSidebar } from "./AppSidebar";
import { MENU_ITEMS, type MenuItem } from "./menu";
import { CreateProjectModal, CreateWorkspaceModal } from "./ProjectModals";
import { CreateTaskModal } from "../features/kanban/CreateTaskModal";
import type { Task, TaskStatus } from "../../../../shared/types";

export { MENU_ITEMS };
export type { MenuItem };
function HeaderLeft({
  title,
  isBoard,
  projectId,
}: {
  title: string;
  isBoard: boolean;
  projectId?: string;
}) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-12)" }}>
      <h1 style={{ margin: 0, fontSize: "var(--fs-18)", fontWeight: 600 }}>
        {title}
      </h1>
      {isBoard ? (
        <Slot name="board.header.chip" context={{ projectId }} />
      ) : (
        <span
          style={{
            height: "var(--chip-height)",
            padding: "0 8px",
            borderRadius: "var(--radius-pill)",
            backgroundColor: "var(--grb)",
            color: "var(--gr)",
            fontSize: "var(--fs-12)",
            fontWeight: 600,
            display: "inline-flex",
            alignItems: "center",
          }}
        >
          {t("server_running")}
        </span>
      )}
    </div>
  );
}

function HeaderRight({
  isBoard,
  theme,
  onToggleTheme,
  onOpenCreateTask,
}: {
  isBoard: boolean;
  theme: string;
  onToggleTheme: () => void;
  onOpenCreateTask?: () => void;
}) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
      {isBoard && (
        <Button
          variant="primary"
          icon={<Plus size={16} />}
          onClick={onOpenCreateTask}
        >
          {t("btn_create_task")}
        </Button>
      )}
      <Button
        variant="ghost"
        size="icon"
        onClick={onToggleTheme}
        aria-label={t("theme_toggle")}
      >
        {theme === "light" ? <Moon size={16} /> : <Sun size={16} />}
      </Button>
    </div>
  );
}

function AppHeader({
  title,
  isBoard,
  projectId,
  theme,
  onToggleTheme,
  onOpenCreateTask,
}: {
  title: string;
  isBoard: boolean;
  projectId?: string;
  theme: string;
  onToggleTheme: () => void;
  onOpenCreateTask?: () => void;
}) {
  return (
    <header
      style={{
        height: "56px",
        padding: "0 var(--sp-24)",
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        borderBottom: "1px solid var(--ln)",
        backgroundColor: "var(--sf)",
        position: "sticky",
        top: 0,
        zIndex: 10,
      }}
    >
      <HeaderLeft title={title} isBoard={isBoard} projectId={projectId} />
      <HeaderRight
        isBoard={isBoard}
        theme={theme}
        onToggleTheme={onToggleTheme}
        onOpenCreateTask={onOpenCreateTask}
      />
    </header>
  );
}

function getTaskColumns(): Column<Task>[] {
  return [
    {
      key: "title",
      header: t("task_col_title"),
      render: (task) => <span style={{ fontWeight: 600 }}>{task.title}</span>,
    },
    {
      key: "status",
      header: t("task_col_status"),
      width: "160px",
      render: (task) => <StatusBadge status={task.status as TaskStatus} />,
    },
    {
      key: "created_at",
      header: t("task_col_created"),
      width: "180px",
      render: (task) => (
        <span style={{ fontSize: "var(--fs-13)", color: "var(--mu)" }}>
          {task.created_at
            ? new Date(task.created_at).toLocaleDateString("vi-VN")
            : "—"}
        </span>
      ),
    },
  ];
}

function CoreBoardView({ tasks }: { tasks: Task[] }) {
  const taskCols = getTaskColumns();
  return (
    <div>
      <div style={{ marginBottom: "var(--sp-16)" }}>
        <h2
          style={{
            margin: "0 0 var(--sp-4) 0",
            fontSize: "var(--fs-16)",
            fontWeight: 600,
          }}
        >
          {t("core_task_list_title")}
        </h2>
        <p style={{ margin: 0, color: "var(--mu)", fontSize: "var(--fs-13)" }}>
          {t("core_task_list_desc")}
        </p>
      </div>
      <Table
        columns={taskCols}
        data={tasks}
        emptyMessage={t("core_task_list_empty")}
      />
    </div>
  );
}

function ModuleDisabledNotice({ screenId }: { screenId: string }) {
  const mod = SCREEN_PRIMARY_MODULE[screenId] || { id: screenId, name: screenId };
  return (
    <div
      style={{
        padding: "var(--sp-32)",
        backgroundColor: "var(--sf)",
        borderRadius: "var(--radius-10)",
        border: "1px solid var(--ln)",
        textAlign: "center",
        maxWidth: "480px",
        margin: "40px auto",
      }}
    >
      <h2
        style={{
          margin: "0 0 var(--sp-8) 0",
          fontSize: "var(--fs-18)",
          fontWeight: 600,
        }}
      >
        {t("feature_disabled_title")}
      </h2>
      <p
        style={{
          margin: "0 0 var(--sp-24) 0",
          color: "var(--mu)",
          fontSize: "var(--fs-14)",
        }}
      >
        {t("feature_disabled_desc", { module: mod.name, id: mod.id })}
      </p>
      <Link to="/settings" style={{ textDecoration: "none" }}>
        <Button variant="primary">{t("go_to_settings")}</Button>
      </Link>
    </div>
  );
}

function EmptyProjectGuide({ onOpenCreate }: { onOpenCreate: () => void }) {
  return (
    <div
      style={{
        padding: "var(--sp-32)",
        backgroundColor: "var(--sf)",
        borderRadius: "var(--radius-10)",
        border: "1px solid var(--ln)",
        textAlign: "center",
        maxWidth: "520px",
        margin: "40px auto",
      }}
    >
      <h2 style={{ margin: "0 0 var(--sp-8) 0", fontSize: "var(--fs-18)", fontWeight: 600 }}>
        {t("empty_projects_title")}
      </h2>
      <p style={{ margin: "0 0 var(--sp-24) 0", color: "var(--mu)", fontSize: "var(--fs-14)" }}>
        {t("empty_projects_desc")}
      </p>
      <Button variant="primary" onClick={onOpenCreate}>
        {t("btn_create_first_project")}
      </Button>
    </div>
  );
}

export interface AppLayoutProps {
  tasks?: Task[];
  onOpenCreateTask?: () => void;
  children?: ReactNode;
}

function getScreenId(path: string): string {
  if (path.startsWith("/agents")) return "agents";
  if (path.startsWith("/dashboard")) return "dashboard";
  if (path.startsWith("/calendar")) return "calendar";
  if (path.startsWith("/knowledge")) return "knowledge";
  if (path.startsWith("/integrations")) return "integrations";
  if (path.startsWith("/settings")) return "settings";
  return "board";
}

function useAppTheme() {
  const [theme, setTheme] = useState<"light" | "dark">("light");
  const toggleTheme = () => {
    const next = theme === "light" ? "dark" : "light";
    setTheme(next);
    document.documentElement.setAttribute("data-theme", next);
  };
  return { theme, toggleTheme };
}

interface LayoutMainProps {
  active: boolean;
  screenId: string;
  tasks: Task[];
  projectId?: string;
  hasProject: boolean;
  onOpenCreateProject: () => void;
  children?: ReactNode;
}

function LayoutMain(props: LayoutMainProps) {
  const { isModuleEnabled, loading } = useFeatures();
  if (!props.active) {
    if (loading) {
      return <Skeleton height="400px" width="100%" radius="lg" />;
    }
    return <ModuleDisabledNotice screenId={props.screenId} />;
  }
  if (props.screenId === "board") {
    if (!props.hasProject && props.tasks.length === 0) {
      return <EmptyProjectGuide onOpenCreate={props.onOpenCreateProject} />;
    }
    return isModuleEnabled("kanban") ? (
      <Slot name="board.columns" context={{ projectId: props.projectId }} />
    ) : (
      <CoreBoardView tasks={props.tasks} />
    );
  }
  return <>{props.children}</>;
}

interface LayoutContentProps {
  title: string;
  screenId: string;
  theme: "light" | "dark";
  onToggleTheme: () => void;
  onOpenCreateTask?: () => void;
  active: boolean;
  tasks: Task[];
  projectId?: string;
  hasProject: boolean;
  onOpenCreateProject: () => void;
  children?: ReactNode;
}

function LayoutContent(props: LayoutContentProps) {
  return (
    <div
      style={{
        flex: 1,
        display: "flex",
        flexDirection: "column",
        minWidth: 0,
        height: "100vh",
        overflow: "hidden",
      }}
    >
      <AppHeader
        title={props.title}
        isBoard={props.screenId === "board"}
        projectId={props.projectId}
        theme={props.theme}
        onToggleTheme={props.onToggleTheme}
        onOpenCreateTask={props.onOpenCreateTask}
      />
      <main style={{ padding: "var(--sp-24)", flex: 1, minHeight: 0, overflowY: "auto", scrollbarGutter: "stable" }}>
        <LayoutMain
          active={props.active}
          screenId={props.screenId}
          tasks={props.tasks}
          projectId={props.projectId}
          hasProject={props.hasProject}
          onOpenCreateProject={props.onOpenCreateProject}
        >
          {props.children}
        </LayoutMain>
      </main>
    </div>
  );
}

function useAppLayoutNav() {
  const location = useLocation();
  const { isScreenActive } = useFeatures();
  const { theme, toggleTheme } = useAppTheme();
  const screenId = getScreenId(location.pathname);
  const active = isScreenActive(screenId);
  const currentMenu =
    MENU_ITEMS.find((m) => m.id === screenId) || MENU_ITEMS[0];
  return { screenId, active, currentMenu, theme, toggleTheme };
}

const layoutRootStyle = {
  display: "flex",
  height: "100vh",
  overflow: "hidden",
  backgroundColor: "var(--bg)",
  color: "var(--fg)",
};

function useAppLayoutModals() {
  const projContext = useContext(ProjectContext);
  const [showCreateProj, setShowCreateProj] = useState(false);
  const [showCreateWs, setShowCreateWs] = useState(false);
  const [showCreateTask, setShowCreateTask] = useState(false);

  const handleOpenCreate = () => {
    if (!projContext?.currentWorkspace) {
      setShowCreateWs(true);
    } else {
      setShowCreateProj(true);
    }
  };

  const handleOpenCreateTask = () => {
    if (!projContext?.currentProject) {
      handleOpenCreate();
    } else {
      setShowCreateTask(true);
    }
  };

  return {
    currentProjectId: projContext?.currentProject?.id,
    hasProject: Boolean(projContext?.currentProject),
    showCreateProj,
    setShowCreateProj,
    showCreateWs,
    setShowCreateWs,
    showCreateTask,
    setShowCreateTask,
    handleOpenCreate,
    handleOpenCreateTask,
  };
}

function LayoutModals({
  modals,
}: {
  modals: ReturnType<typeof useAppLayoutModals>;
}) {
  return (
    <>
      <CreateProjectModal
        isOpen={modals.showCreateProj}
        onClose={() => modals.setShowCreateProj(false)}
      />
      <CreateWorkspaceModal
        isOpen={modals.showCreateWs}
        onClose={() => modals.setShowCreateWs(false)}
      />
      {modals.currentProjectId && (
        <CreateTaskModal
          isOpen={modals.showCreateTask}
          onClose={() => modals.setShowCreateTask(false)}
          projectId={modals.currentProjectId}
        />
      )}
    </>
  );
}

export const AppLayout: FC<AppLayoutProps> = ({
  tasks = [],
  onOpenCreateTask,
  children,
}) => {
  const nav = useAppLayoutNav();
  const modals = useAppLayoutModals();

  return (
    <div style={layoutRootStyle}>
      <AppSidebar screenId={nav.screenId} theme={nav.theme} onToggleTheme={nav.toggleTheme} />
      <LayoutContent
        title={nav.currentMenu.label}
        screenId={nav.screenId}
        theme={nav.theme}
        onToggleTheme={nav.toggleTheme}
        onOpenCreateTask={onOpenCreateTask || modals.handleOpenCreateTask}
        active={nav.active}
        tasks={tasks}
        projectId={modals.currentProjectId}
        hasProject={modals.hasProject}
        onOpenCreateProject={modals.handleOpenCreate}
      >
        {children}
      </LayoutContent>
      <LayoutModals modals={modals} />
    </div>
  );
};

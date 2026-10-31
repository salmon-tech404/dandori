import { FC, ReactNode, useState } from "react";
import { Link, useLocation } from "react-router-dom";
import { LayoutDashboard, Users, BarChart3, Calendar, BookOpen, Boxes, Settings, Sun, Moon, Plus } from "lucide-react";
import { t } from "./i18n";
import { useFeatures } from "./useFeatures";
import { Slot } from "./Slot";
import { Button } from "../ui/components/Button";
import { Table, Column } from "../ui/components/Table";
import { StatusBadge } from "../ui/components/StatusBadge";
import type { Task, TaskStatus } from "../../../../shared/types";

interface MenuItem {
  id: string;
  label: string;
  path: string;
  icon: React.ComponentType<{ size?: number }>;
}

export const MENU_ITEMS: MenuItem[] = [
  { id: "board", label: t("nav_board"), path: "/", icon: LayoutDashboard },
  { id: "agents", label: t("nav_agents"), path: "/agents", icon: Users },
  { id: "dashboard", label: t("nav_dashboard"), path: "/dashboard", icon: BarChart3 },
  { id: "calendar", label: t("nav_calendar"), path: "/calendar", icon: Calendar },
  { id: "knowledge", label: t("nav_knowledge"), path: "/knowledge", icon: BookOpen },
  { id: "integrations", label: t("nav_integrations"), path: "/integrations", icon: Boxes },
  { id: "settings", label: t("nav_settings"), path: "/settings", icon: Settings },
];

function AppSidebar({ currentProjectName, projectReposCount, currentScreenId }: { currentProjectName: string; projectReposCount: number; currentScreenId: string }) {
  const { isScreenActive, effectiveModules, totalModulesCount } = useFeatures();
  const visibleItems = MENU_ITEMS.filter((item) => isScreenActive(item.id));

  return (
    <aside style={{ width: "var(--sidebar-width)", minWidth: "var(--sidebar-width)", backgroundColor: "var(--sf)", borderRight: "1px solid var(--ln)", display: "flex", flexDirection: "column", justifyContent: "space-between", padding: "var(--sp-16) var(--sp-12)" }}>
      <div>
        <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)", padding: "0 var(--sp-8) var(--sp-16) var(--sp-8)", borderBottom: "1px solid var(--ln)", marginBottom: "var(--sp-16)" }}>
          <div style={{ width: "28px", height: "28px", borderRadius: "var(--radius-8)", backgroundColor: "var(--ac)", color: "var(--acf)", display: "flex", alignItems: "center", justifyContent: "center", fontWeight: 700, fontSize: "var(--fs-13)" }}>AH</div>
          <span style={{ fontSize: "var(--fs-16)", fontWeight: 700 }}>{t("app_title")}</span>
        </div>
        <div style={{ padding: "var(--sp-8) var(--sp-12)", backgroundColor: "var(--sf2)", borderRadius: "var(--radius-8)", marginBottom: "var(--sp-16)", display: "flex", flexDirection: "column" }}>
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--fs-13)", fontWeight: 600 }}>{currentProjectName}</span>
          <span style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t("project_selector_repos", { count: projectReposCount })}</span>
        </div>
        <nav aria-label="Main Navigation">
          <ul style={{ listStyle: "none", padding: 0, margin: 0, display: "flex", flexDirection: "column", gap: "2px" }}>
            {visibleItems.map((item) => {
              const isSelected = item.id === currentScreenId;
              const IconComp = item.icon;
              return (
                <li key={item.id}>
                  <Link to={item.path} style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)", height: "var(--menu-height)", padding: "0 var(--sp-12)", borderRadius: "var(--radius-8)", textDecoration: "none", color: isSelected ? "var(--fg)" : "var(--mu)", backgroundColor: isSelected ? "var(--sf2)" : "transparent", fontWeight: isSelected ? 600 : 500, fontSize: "var(--fs-14)" }}>
                    <IconComp size={16} />
                    <span>{item.label}</span>
                  </Link>
                </li>
              );
            })}
          </ul>
        </nav>
      </div>
      <div style={{ padding: "var(--sp-12)", backgroundColor: "var(--sf2)", borderRadius: "var(--radius-8)", fontSize: "var(--fs-12)", color: "var(--mu)" }}>
        {t("active_features_note", { on: effectiveModules.length, total: totalModulesCount })}
      </div>
    </aside>
  );
}

function AppHeader({ title, isBoard, theme, onToggleTheme, onOpenCreateTask }: { title: string; isBoard: boolean; theme: string; onToggleTheme: () => void; onOpenCreateTask?: () => void }) {
  return (
    <header style={{ height: "56px", padding: "0 var(--sp-24)", display: "flex", alignItems: "center", justifyContent: "space-between", borderBottom: "1px solid var(--ln)", backgroundColor: "var(--sf)", position: "sticky", top: 0, zIndex: 10 }}>
      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-12)" }}>
        <h1 style={{ margin: 0, fontSize: "var(--fs-18)", fontWeight: 600 }}>{title}</h1>
        <span style={{ height: "var(--chip-height)", padding: "0 8px", borderRadius: "var(--radius-pill)", backgroundColor: "var(--grb)", color: "var(--gr)", fontSize: "var(--fs-12)", fontWeight: 600, display: "inline-flex", alignItems: "center" }}>
          {t("server_running")}
        </span>
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
        {isBoard && <Button variant="primary" icon={<Plus size={16} />} onClick={onOpenCreateTask}>{t("btn_create_task")}</Button>}
        <Button variant="ghost" size="icon" onClick={onToggleTheme} aria-label="Đổi giao diện sáng/tối">
          {theme === "light" ? <Moon size={16} /> : <Sun size={16} />}
        </Button>
      </div>
    </header>
  );
}

function CoreBoardView({ tasks }: { tasks: Task[] }) {
  const taskCols: Column<Task>[] = [
    { key: "title", header: t("task_col_title"), render: (task) => <span style={{ fontWeight: 600 }}>{task.title}</span> },
    { key: "status", header: t("task_col_status"), width: "160px", render: (task) => <StatusBadge status={task.status as TaskStatus} /> },
    { key: "created_at", header: t("task_col_created"), width: "180px", render: (task) => <span style={{ fontSize: "var(--fs-13)", color: "var(--mu)" }}>{task.created_at ? new Date(task.created_at).toLocaleDateString("vi-VN") : "—"}</span> },
  ];
  return (
    <div>
      <div style={{ marginBottom: "var(--sp-16)" }}>
        <h2 style={{ margin: "0 0 var(--sp-4) 0", fontSize: "var(--fs-16)", fontWeight: 600 }}>{t("core_task_list_title")}</h2>
        <p style={{ margin: 0, color: "var(--mu)", fontSize: "var(--fs-13)" }}>{t("core_task_list_desc")}</p>
      </div>
      <Table columns={taskCols} data={tasks} emptyMessage={t("core_task_list_empty")} />
    </div>
  );
}

function DisabledNotice() {
  return (
    <div style={{ padding: "var(--sp-32)", backgroundColor: "var(--sf)", borderRadius: "var(--radius-10)", border: "1px solid var(--ln)", textAlign: "center", maxWidth: "480px", margin: "40px auto" }}>
      <h2 style={{ margin: "0 0 var(--sp-8) 0", fontSize: "var(--fs-18)", fontWeight: 600 }}>{t("feature_disabled_title")}</h2>
      <p style={{ margin: "0 0 var(--sp-24) 0", color: "var(--mu)", fontSize: "var(--fs-14)" }}>{t("feature_disabled_desc")}</p>
      <Link to="/settings" style={{ textDecoration: "none" }}><Button variant="primary">{t("go_to_settings")}</Button></Link>
    </div>
  );
}

export interface AppLayoutProps {
  currentProjectName?: string;
  projectReposCount?: number;
  tasks?: Task[];
  onOpenCreateTask?: () => void;
  children?: ReactNode;
}

export const AppLayout: FC<AppLayoutProps> = ({ currentProjectName = "Dandori Core", projectReposCount = 1, tasks = [], onOpenCreateTask, children }) => {
  const location = useLocation();
  const { isScreenActive, isModuleEnabled } = useFeatures();
  const [theme, setTheme] = useState<"light" | "dark">("light");

  const path = location.pathname;
  const screenId = path.startsWith("/agents") ? "agents" : path.startsWith("/dashboard") ? "dashboard" : path.startsWith("/calendar") ? "calendar" : path.startsWith("/knowledge") ? "knowledge" : path.startsWith("/integrations") ? "integrations" : path.startsWith("/settings") ? "settings" : "board";
  const active = isScreenActive(screenId);
  const currentMenu = MENU_ITEMS.find((m) => m.id === screenId) || MENU_ITEMS[0];

  const handleToggleTheme = () => {
    const next = theme === "light" ? "dark" : "light";
    setTheme(next);
    document.documentElement.setAttribute("data-theme", next);
  };

  return (
    <div style={{ display: "flex", minHeight: "100vh", backgroundColor: "var(--bg)", color: "var(--fg)" }}>
      <AppSidebar currentProjectName={currentProjectName} projectReposCount={projectReposCount} currentScreenId={screenId} />
      <div style={{ flex: 1, display: "flex", flexDirection: "column", minWidth: 0, overflowY: "auto" }}>
        <AppHeader title={currentMenu.label} isBoard={screenId === "board"} theme={theme} onToggleTheme={handleToggleTheme} onOpenCreateTask={onOpenCreateTask} />
        <main style={{ padding: "var(--sp-24)", flex: 1 }}>
          {!active ? <DisabledNotice /> : screenId === "board" ? (isModuleEnabled("kanban") ? <Slot name="board.columns" /> : <CoreBoardView tasks={tasks} />) : children}
        </main>
      </div>
    </div>
  );
};

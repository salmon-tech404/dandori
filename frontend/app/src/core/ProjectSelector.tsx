import { FC, useRef, useState } from "react";
import { ChevronDown, Plus } from "lucide-react";
import { t } from "./i18n";
import { useProject, Project } from "./project";
import { CreateProjectModal, CreateWorkspaceModal } from "./ProjectModals";
import { Button } from "../ui/components/Button";
import { Tooltip } from "../ui/components/Tooltip";
import { useDismiss } from "../ui/components/useDismiss";

interface DropdownListProps {
  projects: Project[];
  currentProjectId?: string;
  onSelect: (id: string) => void;
  onOpenNewProj: () => void;
  onOpenNewWs: () => void;
}

function DropdownItem({
  project,
  isSelected,
  onSelect,
}: {
  project: Project;
  isSelected: boolean;
  onSelect: (id: string) => void;
}) {
  return (
    <Button
      variant="ghost"
      type="button"
      onClick={() => onSelect(project.id)}
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        width: "100%",
        padding: "6px 8px",
        borderRadius: "var(--radius-6)",
        backgroundColor: isSelected ? "var(--sf2)" : "transparent",
        color: "var(--fg)",
        textAlign: "left",
        fontSize: "var(--fs-13)",
        fontFamily: "var(--font-mono)",
        height: "auto",
        minHeight: "unset",
      }}
    >
      <span>{project.name}</span>
      <span style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>
        {t("project_selector_repos", { count: 1 })}
      </span>
    </Button>
  );
}

function DropdownActionButtons({
  onOpenNewProj,
  onOpenNewWs,
}: {
  onOpenNewProj: () => void;
  onOpenNewWs: () => void;
}) {
  return (
    <>
      <hr style={{ border: "none", borderTop: "1px solid var(--ln)", margin: "4px 0" }} />
      <Button
        variant="ghost"
        size="sm"
        icon={<Plus size={14} />}
        onClick={onOpenNewProj}
        style={{ justifyContent: "flex-start" }}
      >
        {t("project_create_title")}
      </Button>
      <Button
        variant="ghost"
        size="sm"
        icon={<Plus size={14} />}
        onClick={onOpenNewWs}
        style={{ justifyContent: "flex-start" }}
      >
        {t("workspace_create_title")}
      </Button>
    </>
  );
}

const dropdownContainerStyle = {
  position: "absolute" as const,
  top: "100%",
  left: 0,
  right: 0,
  marginTop: "4px",
  backgroundColor: "var(--sf)",
  border: "1px solid var(--ln)",
  borderRadius: "var(--radius-8)",
  boxShadow: "0 4px 12px rgba(0,0,0,0.15)",
  zIndex: 40,
  padding: "var(--sp-8)",
  display: "flex",
  flexDirection: "column" as const,
  gap: "4px",
};

function ProjectDropdownList(props: DropdownListProps) {
  return (
    <div data-testid="project-selector-dropdown" style={dropdownContainerStyle}>
      <div style={{ fontSize: "var(--fs-12)", color: "var(--mu)", padding: "4px 8px" }}>
        {t("all_projects_label")}
      </div>
      {props.projects.map((p) => (
        <DropdownItem
          key={p.id}
          project={p}
          isSelected={p.id === props.currentProjectId}
          onSelect={props.onSelect}
        />
      ))}
      <DropdownActionButtons
        onOpenNewProj={props.onOpenNewProj}
        onOpenNewWs={props.onOpenNewWs}
      />
    </div>
  );
}

const triggerBtnStyle = {
  width: "100%",
  padding: "var(--sp-8) var(--sp-12)",
  backgroundColor: "var(--sf2)",
  border: "1px solid var(--ln)",
  borderRadius: "var(--radius-8)",
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  cursor: "pointer",
  color: "var(--fg)",
  textAlign: "left" as const,
};

function SelectorTrigger({
  name,
  reposCount,
  onClick,
}: {
  name: string;
  reposCount: number;
  onClick: () => void;
}) {
  return (
    <Tooltip content={t("project_selector_tooltip")}>
      <Button
        variant="ghost"
        type="button"
        onClick={onClick}
        aria-label={t("project_selector_tooltip")}
        style={{ ...triggerBtnStyle, height: "auto", minHeight: "unset" }}
      >
      <div style={{ display: "flex", flexDirection: "column", minWidth: 0 }}>
        <span
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "var(--fs-13)",
            fontWeight: 600,
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
        >
          {name}
        </span>
        <span style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>
          {t("project_selector_repos", { count: reposCount })}
        </span>
      </div>
      <ChevronDown size={14} style={{ color: "var(--mu)", flexShrink: 0 }} />
      </Button>
    </Tooltip>
  );
}

function ProjectModalsGroup({
  showProj,
  onCloseProj,
  showWs,
  onCloseWs,
}: {
  showProj: boolean;
  onCloseProj: () => void;
  showWs: boolean;
  onCloseWs: () => void;
}) {
  return (
    <>
      <CreateProjectModal isOpen={showProj} onClose={onCloseProj} />
      <CreateWorkspaceModal isOpen={showWs} onClose={onCloseWs} />
    </>
  );
}

export const ProjectSelector: FC = () => {
  const { currentProject, projects, selectProject } = useProject();
  const [open, setOpen] = useState(false);
  const [showNewProj, setShowNewProj] = useState(false);
  const [showNewWs, setShowNewWs] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  useDismiss({ isOpen: open, onDismiss: () => setOpen(false), ref: containerRef });
  const projName = currentProject ? currentProject.name : t("no_project_selected");
  const reposCount = currentProject ? 1 : 0;

  return (
    <div ref={containerRef} style={{ position: "relative", marginBottom: "var(--sp-16)" }}>
      <SelectorTrigger name={projName} reposCount={reposCount} onClick={() => setOpen(!open)} />
      {open && (
        <ProjectDropdownList
          projects={projects}
          currentProjectId={currentProject?.id}
          onSelect={(id) => { selectProject(id); setOpen(false); }}
          onOpenNewProj={() => { setOpen(false); setShowNewProj(true); }}
          onOpenNewWs={() => { setOpen(false); setShowNewWs(true); }}
        />
      )}
      <ProjectModalsGroup
        showProj={showNewProj}
        onCloseProj={() => setShowNewProj(false)}
        showWs={showNewWs}
        onCloseWs={() => setShowNewWs(false)}
      />
    </div>
  );
};

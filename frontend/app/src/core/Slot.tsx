import { FC, ReactNode, useContext, useEffect, useMemo, useState } from "react";
import { SLOT_CONTRIBUTIONS, SlotContribution } from "./features_generated";
import { useFeatures } from "./useFeatures";
import { ErrorBoundary } from "./ErrorBoundary";
import { ProjectContext } from "./project";

/** Mọi màn module nhận `context` và cả `workspaceId`/`projectId` thành thuộc tính riêng (nhiều màn khai báo theo kiểu này). */
export type SlotComponent = React.ComponentType<{
  context?: Record<string, unknown>;
  workspaceId?: string;
  projectId?: string;
}>;

export interface SlotProps {
  name: string;
  context?: Record<string, unknown>;
  fallback?: ReactNode;
  asTabs?: boolean;
}

interface LoadedContribution {
  id: string;
  label: string;
  Component: SlotComponent;
}

async function loadContribution(contrib: SlotContribution): Promise<LoadedContribution | null> {
  try {
    const mod = (await contrib.load()) as {
      default?: unknown;
      contributions?: Record<string, SlotComponent>;
    };
    let Comp: SlotComponent | undefined;
    if (typeof mod.default === "function") {
      Comp = mod.default as SlotComponent;
    } else if (mod.contributions && typeof mod.contributions[contrib.id] === "function") {
      Comp = mod.contributions[contrib.id];
    } else if (mod.default && typeof (mod.default as Record<string, unknown>)[contrib.id] === "function") {
      Comp = (mod.default as Record<string, unknown>)[contrib.id] as SlotComponent;
    }
    if (Comp) return { id: contrib.id, label: contrib.label, Component: Comp };
  } catch (err) {
    console.error(`Không thể nạp ô cắm ${contrib.id}:`, err);
  }
  return null;
}

/** Lấy workspaceId/projectId (nếu có) từ context để truyền thành thuộc tính riêng cho màn module. */
function idsFromContext(context?: Record<string, unknown>): { workspaceId?: string; projectId?: string } {
  const ws = context?.workspaceId;
  const proj = context?.projectId;
  return {
    workspaceId: typeof ws === "string" ? ws : undefined,
    projectId: typeof proj === "string" ? proj : undefined,
  };
}

function TabButton({
  item,
  isActive,
  onClick,
}: {
  item: LoadedContribution;
  isActive: boolean;
  onClick: () => void;
}) {
  return (
    <button
      role="tab"
      aria-selected={isActive}
      type="button"
      onClick={onClick}
      style={{
        padding: "6px 14px",
        borderRadius: "6px",
        border: "none",
        background: isActive ? "var(--blb, #eff6ff)" : "transparent",
        color: isActive ? "var(--bl, #2563eb)" : "var(--mu, #64748b)",
        fontWeight: isActive ? 600 : 500,
        fontSize: "13px",
        cursor: "pointer",
      }}
    >
      {item.label}
    </button>
  );
}

const tablistStyle = {
  display: "flex",
  gap: "8px",
  borderBottom: "1px solid var(--ln)",
  paddingBottom: "8px",
  flexWrap: "wrap" as const,
};

function TabBar({
  loaded,
  activeTabId,
  onSelectTab,
}: {
  loaded: LoadedContribution[];
  activeTabId: string;
  onSelectTab: (id: string) => void;
}) {
  return (
    <div role="tablist" style={tablistStyle}>
      {loaded.map((item) => (
        <TabButton
          key={item.id}
          item={item}
          isActive={item.id === activeTabId}
          onClick={() => onSelectTab(item.id)}
        />
      ))}
    </div>
  );
}

function TabbedSlotLayout({
  loaded,
  context,
}: {
  loaded: LoadedContribution[];
  context?: Record<string, unknown>;
}) {
  const ids = idsFromContext(context);
  const [activeTabId, setActiveTabId] = useState<string>(loaded[0]?.id || "");
  const currentTab = loaded.find((t) => t.id === activeTabId) || loaded[0];
  if (!currentTab) return null;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
      <TabBar loaded={loaded} activeTabId={currentTab.id} onSelectTab={setActiveTabId} />
      <ErrorBoundary key={currentTab.id} name={currentTab.label}>
        <currentTab.Component context={context} {...ids} />
      </ErrorBoundary>
    </div>
  );
}

/** Mọi màn module nhận workspace/dự án hiện tại từ ngữ cảnh chung; không để màn tự dùng id "default" (gây 404). */
function useMergedContext(ownContext?: Record<string, unknown>): Record<string, unknown> {
  const project = useContext(ProjectContext);
  const workspaceId = project?.currentWorkspace?.id;
  const projectId = project?.currentProject?.id;
  return useMemo(() => ({ workspaceId, projectId, ...ownContext }), [workspaceId, projectId, ownContext]);
}

export const Slot: FC<SlotProps> = ({ name, context: ownContext, fallback = null, asTabs = false }) => {
  const { isModuleEnabled } = useFeatures();
  const context = useMergedContext(ownContext);
  const [loaded, setLoaded] = useState<LoadedContribution[]>([]);
  const activeContribs = (SLOT_CONTRIBUTIONS[name] || [])
    .filter((c) => isModuleEnabled(c.moduleId))
    .sort((a, b) => a.order - b.order);

  useEffect(() => {
    let mounted = true;
    if (activeContribs.length === 0) {
      setLoaded([]);
      return;
    }
    Promise.all(activeContribs.map(loadContribution)).then((res) => {
      if (mounted) setLoaded(res.filter((r): r is LoadedContribution => r !== null));
    });
    return () => {
      mounted = false;
    };
  }, [name, JSON.stringify(activeContribs.map((c) => c.id))]);

  const ids = idsFromContext(context);
  if (activeContribs.length === 0) return <>{fallback}</>;
  if (asTabs && loaded.length > 1) {
    return <TabbedSlotLayout loaded={loaded} context={context} />;
  }

  return (
    <>
      {loaded.map(({ id, label, Component }) => (
        <ErrorBoundary key={id} name={label}>
          <Component context={context} {...ids} />
        </ErrorBoundary>
      ))}
    </>
  );
};

import { FC, ReactNode, useRef, KeyboardEvent, CSSProperties, RefObject } from "react";

export interface TabItem {
  id: string;
  label: ReactNode;
  badge?: ReactNode;
  disabled?: boolean;
}

export interface TabsProps {
  tabs: TabItem[];
  activeTab: string;
  onChange: (tabId: string) => void;
  ariaLabel?: string;
  className?: string;
  style?: CSSProperties;
}

function handleTabsArrowKeys(e: KeyboardEvent<HTMLDivElement>, tabs: TabItem[], activeTab: string, onChange: (id: string) => void, listRef: RefObject<HTMLDivElement | null>) {
  if (e.key !== "ArrowRight" && e.key !== "ArrowLeft") return;
  const enabledTabs = tabs.filter((t) => !t.disabled);
  if (enabledTabs.length === 0) return;
  const currentIndex = enabledTabs.findIndex((t) => t.id === activeTab);
  const nextIndex = e.key === "ArrowRight"
    ? (currentIndex < enabledTabs.length - 1 ? currentIndex + 1 : 0)
    : (currentIndex > 0 ? currentIndex - 1 : enabledTabs.length - 1);
  const nextTab = enabledTabs[nextIndex];
  if (nextTab) {
    e.preventDefault();
    onChange(nextTab.id);
    listRef.current?.querySelectorAll<HTMLButtonElement>('[role="tab"]')?.[nextIndex]?.focus();
  }
}

function TabButton({ tab, isActive, onSelect }: { tab: TabItem; isActive: boolean; onSelect: () => void }) {
  const tabStyle: CSSProperties = {
    height: "36px",
    minHeight: "var(--touch-target)",
    padding: "0 12px",
    border: "none",
    borderBottom: isActive ? "2px solid var(--fg)" : "2px solid transparent",
    backgroundColor: "transparent",
    color: isActive ? "var(--fg)" : "var(--mu)",
    fontWeight: 600,
    fontSize: "var(--fs-14)",
    fontFamily: "var(--font-sans)",
    cursor: tab.disabled ? "not-allowed" : "pointer",
    opacity: tab.disabled ? 0.5 : 1,
    display: "inline-flex",
    alignItems: "center",
    gap: "var(--sp-8)",
    outline: "none",
  };

  return (
    <button role="tab" id={`tab-${tab.id}`} aria-controls={`panel-${tab.id}`} aria-selected={isActive} tabIndex={isActive ? 0 : -1} disabled={tab.disabled} onClick={onSelect} style={tabStyle}>
      <span>{tab.label}</span>
      {tab.badge && <span style={{ display: "inline-flex", alignItems: "center" }}>{tab.badge}</span>}
    </button>
  );
}

export const Tabs: FC<TabsProps> = ({ tabs, activeTab, onChange, ariaLabel, className = "", style }) => {
  const tabListRef = useRef<HTMLDivElement>(null);

  return (
    <div
      ref={tabListRef}
      role="tablist"
      aria-label={ariaLabel}
      onKeyDown={(e) => handleTabsArrowKeys(e, tabs, activeTab, onChange, tabListRef)}
      style={{ display: "flex", flexWrap: "wrap", gap: "var(--sp-4)", borderBottom: "1px solid var(--ln)", padding: "0 var(--sp-8)", ...style }}
      className={`dandori-tabs ${className}`}
    >
      {tabs.map((tab) => (
        <TabButton key={tab.id} tab={tab} isActive={tab.id === activeTab} onSelect={() => { if (!tab.disabled) onChange(tab.id); }} />
      ))}
    </div>
  );
};

import React, { createContext, useContext, useState, useMemo } from "react";
import { SLOT_CONTRIBUTIONS } from "./features_generated";

export interface FeaturesContextType {
  effectiveModules: string[];
  totalModulesCount: number;
  isModuleEnabled: (moduleId: string) => boolean;
  isSlotActive: (slotName: string) => boolean;
  isScreenActive: (screenId: string) => boolean;
  setEffectiveModules: (modules: string[]) => void;
}

// Bảng ánh xạ các màn hình với các ô cắm (slot) của nó theo 04_GIAO_DIEN.md mục 3.3
export const SCREEN_SLOTS_MAP: Record<string, string[]> = {
  board: [
    "board.columns",
    "board.header.chip",
    "board.card.badge",
    "board.task-panel.header",
    "board.task-panel.tab",
    "board.task-panel.section",
    "board.task-panel.action",
    "board.create-task",
  ],
  agents: [
    "agents.defaults",
    "agents.accounts",
    "agents.banner",
    "agents.panel",
  ],
  dashboard: [
    "dashboard.kpi",
    "dashboard.panel",
  ],
  calendar: [
    "calendar.toolbar",
    "calendar.grid",
    "calendar.panel",
  ],
  knowledge: [
    "knowledge.tab",
  ],
  integrations: [
    "integrations.card",
  ],
  settings: [
    "settings.section",
  ],
};

const FeaturesContext = createContext<FeaturesContextType | null>(null);

export interface FeaturesProviderProps {
  initialModules?: string[];
  totalCount?: number;
  children: React.ReactNode;
}

export const FeaturesProvider: React.FC<FeaturesProviderProps> = ({
  initialModules = [],
  totalCount = 43,
  children,
}) => {
  const [effectiveModules, setEffectiveModules] = useState<string[]>(initialModules);

  const enabledSet = useMemo(() => new Set(effectiveModules), [effectiveModules]);

  const isModuleEnabled = (moduleId: string) => enabledSet.has(moduleId);

  const isSlotActive = (slotName: string) => {
    const list = SLOT_CONTRIBUTIONS[slotName] || [];
    return list.some((contrib) => enabledSet.has(contrib.moduleId));
  };

  const isScreenActive = (screenId: string) => {
    // Board và Settings luôn có phần lõi (Core)
    if (screenId === "board" || screenId === "settings") {
      return true;
    }
    const slots = SCREEN_SLOTS_MAP[screenId] || [];
    return slots.some((slot) => isSlotActive(slot));
  };

  const value: FeaturesContextType = {
    effectiveModules,
    totalModulesCount: totalCount,
    isModuleEnabled,
    isSlotActive,
    isScreenActive,
    setEffectiveModules,
  };

  return (
    <FeaturesContext.Provider value={value}>
      {children}
    </FeaturesContext.Provider>
  );
};

export function useFeatures(): FeaturesContextType {
  const ctx = useContext(FeaturesContext);
  if (!ctx) {
    // Fallback mặc định an toàn nếu dùng ngoài provider
    return {
      effectiveModules: [],
      totalModulesCount: 43,
      isModuleEnabled: () => false,
      isSlotActive: () => false,
      isScreenActive: (screenId: string) => screenId === "board" || screenId === "settings",
      setEffectiveModules: () => {},
    };
  }
  return ctx;
}

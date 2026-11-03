import React, { createContext, useContext, useState, useMemo, useEffect, useCallback } from "react";
import { SLOT_CONTRIBUTIONS } from "./features_generated";
import { apiClient, getSafeErrorMessage } from "./api";
import { useProject } from "./project";

export interface ScreenPrimaryModule {
  id: string;
  name: string;
}

export const SCREEN_PRIMARY_MODULE: Record<string, ScreenPrimaryModule> = {
  agents: { id: "lead-worker", name: "Agent và tài khoản" },
  dashboard: { id: "quota-tracking", name: "Dashboard" },
  calendar: { id: "calendar", name: "Lịch" },
  knowledge: { id: "skills-md", name: "Prompt và tri thức" },
  integrations: { id: "integrations", name: "Tích hợp" },
};

export interface FeaturesContextType {
  effectiveModules: string[];
  totalModulesCount: number;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  isModuleEnabled: (moduleId: string) => boolean;
  isSlotActive: (slotName: string) => boolean;
  isScreenActive: (screenId: string) => boolean;
  setEffectiveModules: (modules: string[]) => void;
}

export const SCREEN_SLOTS_MAP: Record<string, string[]> = {
  board: [
    "board.columns", "board.header.chip", "board.card.badge", "board.task-panel.header",
    "board.task-panel.tab", "board.task-panel.section", "board.task-panel.action", "board.create-task",
  ],
  agents: ["agents.defaults", "agents.accounts", "agents.banner", "agents.panel"],
  dashboard: ["dashboard.kpi", "dashboard.panel"],
  calendar: ["calendar.toolbar", "calendar.grid", "calendar.panel"],
  knowledge: ["knowledge.tab"],
  integrations: ["integrations.card"],
  settings: ["settings.section"],
};

const FeaturesContext = createContext<FeaturesContextType | null>(null);

export interface FeaturesProviderProps {
  initialModules?: string[];
  totalCount?: number;
  children: React.ReactNode;
}

async function fetchFeatures(
  workspaceId: string | undefined,
  projectId: string | undefined
): Promise<{ effective: string[]; totalCount: number | null }> {
  const q = new URLSearchParams();
  if (workspaceId) q.set("workspace", workspaceId);
  if (projectId) q.set("project", projectId);
  const queryStr = q.toString() ? `?${q.toString()}` : "";
  const effRes = await apiClient<{ effective: Array<{ module_id: string; enabled: boolean }> }>(
    `/api/v1/features/effective${queryStr}`
  );
  const effective = Array.isArray(effRes.effective)
    ? effRes.effective.filter((e) => e.enabled).map((e) => e.module_id)
    : [];
  const modRes = await apiClient<{ modules: unknown[] }>("/api/v1/features/modules").catch(() => null);
  const totalCount = Array.isArray(modRes?.modules) ? modRes.modules.length : null;
  return { effective, totalCount };
}

function useFeaturesLoader(
  initialModules: string[] | undefined,
  setEffectiveModules: (mods: string[]) => void,
  setTotalCount: (total: number) => void
) {
  const { currentWorkspace, currentProject } = useProject();
  const workspaceId = currentWorkspace?.id;
  const projectId = currentProject?.id;
  const [loading, setLoading] = useState(initialModules === undefined);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await fetchFeatures(workspaceId, projectId);
      setEffectiveModules(data.effective);
      if (data.totalCount !== null) setTotalCount(data.totalCount);
    } catch (err) {
      setError(getSafeErrorMessage(err).userMessage);
    } finally {
      setLoading(false);
    }
  }, [workspaceId, projectId, setEffectiveModules, setTotalCount]);

  useEffect(() => {
    if (initialModules === undefined) {
      refresh();
    }
  }, [initialModules, refresh]);

  return { loading, error, refresh };
}

function useFeaturesChecks(effectiveModules: string[]) {
  const enabledSet = useMemo(() => new Set(effectiveModules), [effectiveModules]);

  const isModuleEnabled = useCallback((moduleId: string) => enabledSet.has(moduleId), [enabledSet]);

  const isSlotActive = useCallback((slotName: string) => {
    const list = SLOT_CONTRIBUTIONS[slotName] || [];
    return list.some((contrib) => enabledSet.has(contrib.moduleId));
  }, [enabledSet]);

  const isScreenActive = useCallback((screenId: string) => {
    if (screenId === "board" || screenId === "settings") return true;
    const slots = SCREEN_SLOTS_MAP[screenId] || [];
    return slots.some((slot) => {
      const list = SLOT_CONTRIBUTIONS[slot] || [];
      return list.some((contrib) => enabledSet.has(contrib.moduleId));
    });
  }, [enabledSet]);

  return { isModuleEnabled, isSlotActive, isScreenActive };
}

export const FeaturesProvider: React.FC<FeaturesProviderProps> = ({
  initialModules,
  totalCount: initTotal = 43,
  children,
}) => {
  const [effectiveModules, setEffectiveModules] = useState<string[]>(initialModules ?? []);
  const [totalModulesCount, setTotalCount] = useState<number>(initTotal);

  const loader = useFeaturesLoader(initialModules, setEffectiveModules, setTotalCount);
  const checks = useFeaturesChecks(effectiveModules);

  const value: FeaturesContextType = {
    effectiveModules,
    totalModulesCount,
    loading: loader.loading,
    error: loader.error,
    refresh: loader.refresh,
    setEffectiveModules,
    ...checks,
  };

  return <FeaturesContext.Provider value={value}>{children}</FeaturesContext.Provider>;
};

const DEFAULT_FEATURES_CONTEXT: FeaturesContextType = {
  effectiveModules: [],
  totalModulesCount: 43,
  loading: false,
  error: null,
  refresh: async () => {},
  isModuleEnabled: () => false,
  isSlotActive: () => false,
  isScreenActive: (id: string) => id === "board" || id === "settings",
  setEffectiveModules: () => {},
};

export function useFeatures(): FeaturesContextType {
  const ctx = useContext(FeaturesContext);
  return ctx || DEFAULT_FEATURES_CONTEXT;
}

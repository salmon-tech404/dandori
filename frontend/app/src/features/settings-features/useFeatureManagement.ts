import { useState, useEffect, useCallback } from "react";
import { FeatureModuleItem, FeatureAuditItem, ScopeType } from "./types";
import { ModalState } from "./Modals";
import { apiClient } from "../../core/api";
import { useFeatures } from "../../core/useFeatures";
import { useProject } from "../../core/project";

export interface FeaturePackItem {
  id: string;
  name: string;
  modules: string[];
}

interface RawModule {
  id: string;
  name: string;
  description: string;
  group_name: string;
  core: boolean;
  requires: string[];
  effort: "light" | "medium" | "heavy";
  risk: "low" | "medium" | "high";
  requires_worker_restart?: boolean;
}

interface EnablePlanData {
  will_enable_with: string[];
  needs_risk_ack: boolean;
  risk_note?: string | null;
}

interface DisablePlanData {
  will_break: string[];
}

const DEFAULT_PACK_MAP: Record<string, string[]> = {
  "solo-light": ["sample-hello", "auth-login", "kanban", "live-status"],
  "small-team": ["sample-hello", "auth-login", "kanban", "live-status", "multi-account", "approval-gate"],
  "full": ["sample-hello", "auth-login", "kanban", "live-status", "multi-account", "approval-gate", "calendar"],
};

function buildTargetId(scope: ScopeType, wsId?: string, projId?: string): string {
  if (scope === "system") return "system";
  if (scope === "workspace") return wsId || "default";
  return projId || "default";
}

function mapRawModules(
  rawList: RawModule[],
  effList: Array<{ module_id: string; enabled: boolean; locked?: boolean }>
): FeatureModuleItem[] {
  const effMap = new Map((effList || []).map((e) => [e.module_id, e]));
  return rawList.map((m) => {
    const eff = effMap.get(m.id);
    return {
      id: m.id,
      name: m.name,
      description: m.description,
      group: m.group_name,
      core: m.core,
      requires: m.requires || [],
      effort: m.effort || "light",
      risk: m.risk || "low",
      requires_worker_restart: m.requires_worker_restart || false,
      enabled: eff ? eff.enabled : false,
      locked_by_parent: eff ? Boolean(eff.locked) : false,
    };
  });
}

function useFeatureLoader(
  initialModules: FeatureModuleItem[],
  scope: ScopeType,
  setModules: (m: FeatureModuleItem[]) => void,
  setPacks: (p: FeaturePackItem[]) => void,
  setAudits: (a: FeatureAuditItem[]) => void
) {
  const { currentWorkspace, currentProject } = useProject();

  const loadData = useCallback(async () => {
    if (initialModules.length > 0) return;
    try {
      // Mỗi tab phạm vi hiển thị trạng thái đặt ở ĐÚNG phạm vi đó: hệ thống không gắn workspace/dự án,
      // workspace chỉ gắn workspace, dự án gắn cả hai.
      const wsId = scope === "system" ? undefined : currentWorkspace?.id;
      const projId = scope === "project" ? currentProject?.id : undefined;
      const [modRes, packRes, auditRes, effRes] = await Promise.all([
        apiClient<{ modules: RawModule[] }>("/api/v1/features/modules"),
        apiClient<{ packs: FeaturePackItem[] }>("/api/v1/features/packs").catch(() => ({ packs: [] })),
        apiClient<{ audit: FeatureAuditItem[] }>("/api/v1/features/audit").catch(() => ({ audit: [] })),
        apiClient<{ effective: Array<{ module_id: string; enabled: boolean; locked?: boolean }> }>(
          `/api/v1/features/effective?workspace=${wsId || "default"}&project=${projId || "default"}`
        ).catch(() => ({ effective: [] })),
      ]);

      setModules(mapRawModules(modRes.modules || [], effRes.effective || []));
      setPacks(packRes.packs || []);
      setAudits(auditRes.audit || []);
    } catch {
      // ignore
    }
  }, [initialModules.length, scope, currentWorkspace?.id, currentProject?.id, setModules, setPacks, setAudits]);

  useEffect(() => {
    loadData();
  }, [loadData, scope]);

  return { loadData };
}

interface ApplyPlanParams {
  id: string;
  enable: boolean;
  scope: ScopeType;
  wsId?: string;
  projId?: string;
  riskAck?: boolean;
}

async function executeApply(p: ApplyPlanParams): Promise<boolean> {
  const targetId = buildTargetId(p.scope, p.wsId, p.projId);
  try {
    await apiClient("/api/v1/features/apply", {
      method: "POST",
      body: JSON.stringify({
        module_id: p.id,
        scope: p.scope,
        target_id: targetId,
        workspace_id: p.wsId || targetId,
        enable: p.enable,
        risk_acknowledged: p.riskAck ?? true,
        reason: p.enable ? "Bật từ giao diện Cài đặt" : "Tắt từ giao diện Cài đặt",
      }),
    });
    return true;
  } catch {
    return false;
  }
}

async function requestPlanEnable(
  id: string,
  scope: ScopeType,
  wsId?: string,
  projId?: string
): Promise<EnablePlanData | null> {
  const targetId = buildTargetId(scope, wsId, projId);
  try {
    const res = await apiClient<{ plan: EnablePlanData }>("/api/v1/features/plan-enable", {
      method: "POST",
      body: JSON.stringify({
        module_id: id,
        scope,
        target_id: targetId,
        workspace_id: wsId || targetId,
      }),
    });
    return res.plan;
  } catch {
    return null;
  }
}

function handlePlanEnableResult(
  item: FeatureModuleItem,
  plan: EnablePlanData | null,
  modules: FeatureModuleItem[],
  setModal: (m: ModalState) => void,
  applyToggle: (id: string, enable: boolean) => void
) {
  if (plan?.needs_risk_ack || (!plan && item.risk === "high")) {
    const note = plan?.risk_note || item.risk_note;
    setModal({
      type: "risk",
      targetId: item.id,
      note: note || undefined,
      onConfirm: () => applyToggle(item.id, true),
    });
    return;
  }

  const missing = plan
    ? plan.will_enable_with.filter((id) => id !== item.id)
    : item.requires.filter((r) => !modules.find((m) => m.id === r)?.enabled);

  if (missing.length > 0) {
    setModal({
      type: "dependencies",
      targetId: item.id,
      names: missing,
      onConfirm: () => {
        missing.forEach((d) => applyToggle(d, true));
        applyToggle(item.id, true);
      },
    });
    return;
  }

  applyToggle(item.id, true);
}

async function requestPlanDisable(
  id: string,
  scope: ScopeType,
  wsId?: string,
  projId?: string
): Promise<DisablePlanData | null> {
  const targetId = buildTargetId(scope, wsId, projId);
  try {
    const res = await apiClient<{ plan: DisablePlanData }>("/api/v1/features/plan-disable", {
      method: "POST",
      body: JSON.stringify({
        module_id: id,
        scope,
        target_id: targetId,
        workspace_id: wsId || targetId,
      }),
    });
    return res.plan;
  } catch {
    return null;
  }
}

function handlePlanDisableResult(
  item: FeatureModuleItem,
  plan: DisablePlanData | null,
  modules: FeatureModuleItem[],
  setModal: (m: ModalState) => void,
  applyToggle: (id: string, enable: boolean) => void
) {
  const willBreak = plan
    ? plan.will_break
    : modules.filter((m) => m.enabled && m.requires.includes(item.id)).map((m) => m.id);

  if (willBreak.length > 0) {
    setModal({
      type: "dependents",
      targetId: item.id,
      names: willBreak,
      onConfirm: () => {
        willBreak.forEach((d) => applyToggle(d, false));
        applyToggle(item.id, false);
      },
    });
    return;
  }

  applyToggle(item.id, false);
}

function usePlanActions(
  modules: FeatureModuleItem[],
  scope: ScopeType,
  setModal: (m: ModalState) => void,
  applyToggle: (id: string, enable: boolean) => void,
  isMock: boolean
) {
  const { currentWorkspace, currentProject } = useProject();

  const planEnableAction = (item: FeatureModuleItem) => {
    if (isMock) {
      handlePlanEnableResult(item, null, modules, setModal, applyToggle);
      return;
    }
    void requestPlanEnable(item.id, scope, currentWorkspace?.id, currentProject?.id).then((plan) => {
      handlePlanEnableResult(item, plan, modules, setModal, applyToggle);
    });
  };

  const planDisableAction = (item: FeatureModuleItem) => {
    if (isMock) {
      handlePlanDisableResult(item, null, modules, setModal, applyToggle);
      return;
    }
    void requestPlanDisable(item.id, scope, currentWorkspace?.id, currentProject?.id).then((plan) => {
      handlePlanDisableResult(item, plan, modules, setModal, applyToggle);
    });
  };

  return { planEnableAction, planDisableAction };
}

async function applyPackModules(
  packModules: string[],
  scope: ScopeType,
  wsId?: string,
  projId?: string
) {
  for (const mid of packModules) {
    await executeApply({ id: mid, enable: true, scope, wsId, projId, riskAck: true });
  }
}

function usePackApplier(
  packs: FeaturePackItem[],
  scope: ScopeType,
  setModules: React.Dispatch<React.SetStateAction<FeatureModuleItem[]>>,
  setModal: (m: ModalState) => void,
  refreshContext: () => void,
  loadData: () => void,
  isMock: boolean
) {
  const { currentWorkspace, currentProject } = useProject();

  return useCallback(
    (packId: string) => {
      const serverPack = packs.find((p) => p.id === packId);
      const packModules = serverPack ? serverPack.modules : DEFAULT_PACK_MAP[packId] || [];
      setModal({
        type: "pack",
        targetId: packId,
        names: packModules,
        onConfirm: async () => {
          setModules((prev) => prev.map((m) => (packModules.includes(m.id) ? { ...m, enabled: true } : m)));
          setModal({ type: null });
          if (!isMock) {
            await applyPackModules(packModules, scope, currentWorkspace?.id, currentProject?.id);
            refreshContext();
            loadData();
          }
        },
      });
    },
    [packs, scope, currentWorkspace?.id, currentProject?.id, refreshContext, loadData, isMock, setModules, setModal]
  );
}

function useToggleApplier(
  scope: ScopeType,
  setModules: React.Dispatch<React.SetStateAction<FeatureModuleItem[]>>,
  setAudits: React.Dispatch<React.SetStateAction<FeatureAuditItem[]>>,
  setModal: (m: ModalState) => void,
  refreshContext: () => void,
  isMock: boolean
) {
  const { currentWorkspace, currentProject } = useProject();

  return useCallback(
    async (id: string, enable: boolean) => {
      setModules((prev) => prev.map((m) => (m.id === id ? { ...m, enabled: enable } : m)));
      setAudits((prev) => [
        {
          id: `audit-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
          module_id: id,
          action: enable ? "enable" : "disable",
          actor: "Admin",
          created_at: new Date().toISOString(),
        },
        ...prev,
      ]);
      setModal({ type: null });
      if (!isMock) {
        await executeApply({
          id,
          enable,
          scope,
          wsId: currentWorkspace?.id,
          projId: currentProject?.id,
          riskAck: true,
        });
        refreshContext();
      }
    },
    [scope, currentWorkspace?.id, currentProject?.id, refreshContext, isMock, setModules, setAudits, setModal]
  );
}

function useUndoHandler(
  audits: FeatureAuditItem[],
  setModules: React.Dispatch<React.SetStateAction<FeatureModuleItem[]>>,
  setAudits: React.Dispatch<React.SetStateAction<FeatureAuditItem[]>>
) {
  return useCallback(() => {
    if (audits.length === 0) return;
    const latest = audits[0];
    setModules((prev) => prev.map((m) => (m.id === latest.module_id ? { ...m, enabled: latest.action !== "enable" } : m)));
    setAudits((prev) => prev.slice(1));
  }, [audits, setModules, setAudits]);
}

export function useFeatureManagement(initialModules: FeatureModuleItem[] = [], scope: ScopeType = "workspace") {
  const isMock = initialModules.length > 0;
  const [modules, setModules] = useState<FeatureModuleItem[]>(initialModules);
  const [packs, setPacks] = useState<FeaturePackItem[]>([]);
  const [audits, setAudits] = useState<FeatureAuditItem[]>([]);
  const [modal, setModal] = useState<ModalState>({ type: null });

  const { refresh: refreshContext } = useFeatures();
  const { loadData } = useFeatureLoader(initialModules, scope, setModules, setPacks, setAudits);

  const applyToggle = useToggleApplier(scope, setModules, setAudits, setModal, refreshContext, isMock);
  const { planEnableAction, planDisableAction } = usePlanActions(modules, scope, setModal, applyToggle, isMock);

  const handleToggle = useCallback(
    (item: FeatureModuleItem) => {
      if (item.core || item.locked_by_parent) return;
      if (!item.enabled) {
        planEnableAction(item);
      } else {
        planDisableAction(item);
      }
    },
    [planEnableAction, planDisableAction]
  );

  const handleApplyPack = usePackApplier(packs, scope, setModules, setModal, refreshContext, loadData, isMock);
  const handleUndo = useUndoHandler(audits, setModules, setAudits);

  return {
    modules,
    setModules,
    packs,
    audits,
    modal,
    setModal,
    handleToggle,
    handleApplyPack,
    handleUndo,
    applyToggle,
    reload: loadData,
  };
}

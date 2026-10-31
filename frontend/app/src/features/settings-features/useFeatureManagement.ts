import { useState } from "react";
import { FeatureModuleItem, FeatureAuditItem } from "./types";
import { ModalState } from "./Modals";

export function useFeatureManagement(initialModules: FeatureModuleItem[]) {
  const [modules, setModules] = useState<FeatureModuleItem[]>(initialModules);
  const [audits, setAudits] = useState<FeatureAuditItem[]>([]);
  const [modal, setModal] = useState<ModalState>({ type: null });

  const applyToggle = (id: string, enable: boolean) => {
    setModules((prev) => prev.map((m) => (m.id === id ? { ...m, enabled: enable } : m)));
    setAudits((prev) => [{ id: `audit-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`, module_id: id, action: enable ? "enable" : "disable", actor: "Admin", created_at: new Date().toISOString() }, ...prev]);
    setModal({ type: null });
  };

  const handleToggle = (item: FeatureModuleItem) => {
    if (item.core || item.locked_by_parent) return;
    if (!item.enabled) {
      const missing = item.requires.filter((r) => !modules.find((m) => m.id === r)?.enabled);
      if (missing.length > 0) {
        setModal({ type: "dependencies", targetId: item.id, names: missing, onConfirm: () => { missing.forEach((d) => applyToggle(d, true)); applyToggle(item.id, true); } });
        return;
      }
      if (item.risk === "high") {
        setModal({ type: "risk", targetId: item.id, note: item.risk_note, onConfirm: () => applyToggle(item.id, true) });
        return;
      }
      applyToggle(item.id, true);
    } else {
      const dependents = modules.filter((m) => m.enabled && m.requires.includes(item.id)).map((m) => m.id);
      if (dependents.length > 0) {
        setModal({ type: "dependents", targetId: item.id, names: dependents, onConfirm: () => { dependents.forEach((d) => applyToggle(d, false)); applyToggle(item.id, false); } });
        return;
      }
      applyToggle(item.id, false);
    }
  };

  const handleUndo = () => {
    if (audits.length === 0) return;
    const latest = audits[0];
    setModules((prev) => prev.map((m) => (m.id === latest.module_id ? { ...m, enabled: latest.action !== "enable" } : m)));
    setAudits((prev) => prev.slice(1));
  };

  return { modules, setModules, audits, modal, setModal, handleToggle, handleUndo, applyToggle };
}

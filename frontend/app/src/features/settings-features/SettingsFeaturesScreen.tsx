import { FC, useState } from "react";
import { t } from "../../core/i18n";
import { SettingsHeader } from "./SettingsHeader";
import { PacksSection } from "./PacksSection";
import { GroupCard } from "./GroupCard";
import { AuditLogSection } from "./AuditLogSection";
import { ActionModals } from "./Modals";
import { FeatureModuleItem, ScopeType } from "./types";
import { useFeatureManagement } from "./useFeatureManagement";

const GROUP_TITLES: Record<string, string> = {
  safety: t("group_safety"), accounts: t("group_accounts"), interface: t("group_interface"),
  coordination: t("group_coordination"), knowledge: t("group_knowledge"), integrations: t("group_integrations"),
  automation: t("group_automation"), infrastructure: t("group_infrastructure"),
};

export const SettingsFeaturesScreen: FC<{ initialModules?: FeatureModuleItem[] }> = ({ initialModules = [] }) => {
  const { modules, setModules, audits, modal, setModal, handleToggle, handleUndo } = useFeatureManagement(initialModules);
  const [scope, setScope] = useState<ScopeType>("workspace");
  const [search, setSearch] = useState("");
  const [filter, setFilter] = useState<"all" | "enabled" | "disabled">("all");

  const handleApplyPack = (packId: string) => {
    const packModules = packId === "solo-light" ? ["sample-hello", "auth-login", "kanban", "live-status"] : packId === "small-team" ? ["sample-hello", "auth-login", "kanban", "live-status", "multi-account", "approval-gate"] : ["sample-hello", "auth-login", "kanban", "live-status", "multi-account", "approval-gate", "calendar"];
    setModal({ type: "pack", targetId: packId, names: packModules, onConfirm: () => {
      setModules((prev) => prev.map((m) => (packModules.includes(m.id) ? { ...m, enabled: true } : m)));
      setModal({ type: null });
    }});
  };

  const filtered = modules.filter((m) => {
    if (search && !m.name.toLowerCase().includes(search.toLowerCase()) && !m.description.toLowerCase().includes(search.toLowerCase())) return false;
    return filter === "all" || (filter === "enabled" ? m.enabled : !m.enabled);
  });
  const groups = Array.from(new Set(modules.map((m) => m.group)));

  return (
    <div style={{ maxWidth: "1000px", margin: "0 auto" }}>
      <SettingsHeader scope={scope} onScopeChange={setScope} search={search} onSearchChange={setSearch} filter={filter} onFilterChange={setFilter} />
      <PacksSection onApplyPack={handleApplyPack} enabledCount={modules.filter((m) => m.enabled).length} totalCount={modules.length} />
      <div>
        {groups.map((grp) => {
          const groupItems = filtered.filter((m) => m.group === grp);
          return groupItems.length > 0 ? <GroupCard key={grp} title={GROUP_TITLES[grp] || grp} items={groupItems} onToggle={handleToggle} onConfigure={() => {}} /> : null;
        })}
      </div>
      <AuditLogSection audits={audits} onUndo={handleUndo} undoDisabled={audits.length === 0} />
      <ActionModals modal={modal} onClose={() => setModal({ type: null })} />
    </div>
  );
};

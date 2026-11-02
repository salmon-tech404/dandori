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
  // Giá trị `group` thật do máy chủ trả về (ModuleGroup): ánh xạ về nhóm hiển thị, không để lộ mã thô.
  orchestration: t("group_coordination"), runtime: t("group_infrastructure"),
  scheduling: t("group_automation"), dashboard: t("group_dashboard"),
};

export const SettingsFeaturesScreen: FC<{ initialModules?: FeatureModuleItem[] }> = ({ initialModules = [] }) => {
  const [scope, setScope] = useState<ScopeType>("workspace");
  const { modules, audits, modal, setModal, handleToggle, handleApplyPack, handleUndo } =
    useFeatureManagement(initialModules, scope);
  const [search, setSearch] = useState("");
  const [filter, setFilter] = useState<"all" | "enabled" | "disabled">("all");

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
      <AuditLogSection audits={audits} onUndo={initialModules.length > 0 ? handleUndo : undefined} undoDisabled={audits.length === 0} />
      <ActionModals modal={modal} onClose={() => setModal({ type: null })} />
    </div>
  );
};

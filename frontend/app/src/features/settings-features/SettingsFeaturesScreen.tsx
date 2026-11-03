import { FC, useContext, useState } from "react";
import { t } from "../../core/i18n";
import { SettingsHeader } from "./SettingsHeader";
import { PacksSection } from "./PacksSection";
import { GroupCard } from "./GroupCard";
import { AuditLogSection } from "./AuditLogSection";
import { ActionModals } from "./Modals";
import { FeatureModuleItem, ScopeType } from "./types";
import { useFeatureManagement } from "./useFeatureManagement";
import { ProjectContext } from "../../core/project";

const GROUP_TITLES: Record<string, string> = {
  safety: t("group_safety"), accounts: t("group_accounts"), interface: t("group_interface"),
  coordination: t("group_coordination"), knowledge: t("group_knowledge"), integrations: t("group_integrations"),
  automation: t("group_automation"), infrastructure: t("group_infrastructure"),
  // Giá trị `group` thật do máy chủ trả về (ModuleGroup): ánh xạ về nhóm hiển thị, không để lộ mã thô.
  orchestration: t("group_coordination"), runtime: t("group_infrastructure"),
  scheduling: t("group_automation"), dashboard: t("group_dashboard"),
};

/** Dòng giải thích phạm vi đang chọn; `blocked` khi phạm vi cần workspace/dự án mà chưa có. */
function describeScope(
  scope: ScopeType,
  hasContext: boolean,
  wsName?: string,
  projName?: string,
): { text: string; blocked: boolean } {
  if (scope === "system") return { text: t("scope_desc_system"), blocked: false };
  // Không có ngữ cảnh workspace/dự án (ví dụ màn dùng riêng lẻ): không chặn, máy chủ sẽ kiểm tra.
  if (!hasContext) return { text: "", blocked: false };
  if (scope === "workspace") {
    return wsName ? { text: t("scope_desc_workspace", { name: wsName }), blocked: false } : { text: t("scope_need_workspace"), blocked: true };
  }
  return projName ? { text: t("scope_desc_project", { name: projName }), blocked: false } : { text: t("scope_need_project"), blocked: true };
}

export const SettingsFeaturesScreen: FC<{ initialModules?: FeatureModuleItem[] }> = ({ initialModules = [] }) => {
  const project = useContext(ProjectContext);
  // Chưa có workspace thì mở thẳng tab "Hệ thống" để vẫn bật/tắt được tính năng.
  const [scope, setScope] = useState<ScopeType>(project && !project.currentWorkspace ? "system" : "workspace");
  const { modules, audits, modal, setModal, handleToggle, handleApplyPack, handleUndo } =
    useFeatureManagement(initialModules, scope);
  const [search, setSearch] = useState("");
  const [filter, setFilter] = useState<"all" | "enabled" | "disabled">("all");
  const scopeNote = describeScope(scope, Boolean(project), project?.currentWorkspace?.name, project?.currentProject?.name);

  const filtered = modules.filter((m) => {
    if (search && !m.name.toLowerCase().includes(search.toLowerCase()) && !m.description.toLowerCase().includes(search.toLowerCase())) return false;
    return filter === "all" || (filter === "enabled" ? m.enabled : !m.enabled);
  });
  const groups = Array.from(new Set(modules.map((m) => m.group)));

  return (
    <div style={{ maxWidth: "1000px", margin: "0 auto" }}>
      <SettingsHeader scope={scope} onScopeChange={setScope} search={search} onSearchChange={setSearch} filter={filter} onFilterChange={setFilter} />
      {scopeNote.text && <p
        role={scopeNote.blocked ? "alert" : undefined}
        data-testid="scope-note"
        style={{ margin: "0 0 var(--sp-16) 0", fontSize: "var(--fs-13)", color: scopeNote.blocked ? "var(--rd)" : "var(--mu)" }}
      >
        {scopeNote.text}
      </p>}
      <PacksSection onApplyPack={scopeNote.blocked ? () => {} : handleApplyPack} enabledCount={modules.filter((m) => m.enabled).length} totalCount={modules.length} />
      <div>
        {groups.map((grp) => {
          const groupItems = filtered.filter((m) => m.group === grp);
          return groupItems.length > 0 ? <GroupCard key={grp} title={GROUP_TITLES[grp] || grp} items={groupItems} onToggle={scopeNote.blocked ? () => {} : handleToggle} onConfigure={() => {}} /> : null;
        })}
      </div>
      <AuditLogSection audits={audits} onUndo={initialModules.length > 0 ? handleUndo : undefined} undoDisabled={audits.length === 0} />
      <ActionModals modal={modal} onClose={() => setModal({ type: null })} />
    </div>
  );
};

import { useState } from "react";
import { t } from "../../core/i18n";
import { fetchSkillDetail } from "./api";
import { ImportSkillDialog } from "./ImportSkillDialog";
import { SkillDetailDialog } from "./SkillDetailDialog";
import { SkillsTable } from "./SkillsTable";
import type { SkillDetail, SkillSummary } from "./types";
import { useSkills } from "./useSkills";

interface SkillsPanelProps {
  workspaceId: string;
}

export function SkillsPanel({ workspaceId }: SkillsPanelProps) {
  const { skills, loading, error, reload, onApprove, onDelete } = useSkills(workspaceId);
  const [selectedSkill, setSelectedSkill] = useState<SkillDetail | null>(null);
  const [showImportModal, setShowImportModal] = useState(false);

  const handleViewDetail = async (skillId: string) => {
    try {
      const detail = await fetchSkillDetail(workspaceId, skillId);
      setSelectedSkill(detail);
    } catch {
      // Bỏ qua lỗi
    }
  };

  return (
    <div className="space-y-6 p-6">
      <PanelHeader onOpenImport={() => setShowImportModal(true)} />
      <ScriptWarningBanner />
      <SkillsContentSection
        loading={loading}
        error={error}
        skills={skills}
        onViewDetail={handleViewDetail}
        onApprove={onApprove}
        onDelete={onDelete}
      />
      <SkillModals
        workspaceId={workspaceId}
        selectedSkill={selectedSkill}
        showImportModal={showImportModal}
        onCloseDetail={() => setSelectedSkill(null)}
        onCloseImport={() => setShowImportModal(false)}
        onReloadList={() => void reload()}
        onReloadDetail={(id) => void handleViewDetail(id)}
      />
    </div>
  );
}

function PanelHeader({ onOpenImport }: { onOpenImport: () => void }) {
  return (
    <div className="flex items-center justify-between">
      <div>
        <h2 className="text-xl font-bold">{t("skills_title")}</h2>
        <p className="text-sm text-gray-500">{t("skills_desc")}</p>
      </div>
      <button
        type="button"
        onClick={onOpenImport}
        className="rounded bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
      >
        {t("skills_import_btn")}
      </button>
    </div>
  );
}

function ScriptWarningBanner() {
  return (
    <div className="rounded border border-amber-200 bg-amber-50 p-4 text-sm text-amber-800">
      <p className="font-medium">{t("skills_scripts_warning")}</p>
    </div>
  );
}

interface SkillsContentSectionProps {
  loading: boolean;
  error: string | null;
  skills: SkillSummary[];
  onViewDetail: (id: string) => void;
  onApprove: (id: string) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
}

function SkillsContentSection(props: SkillsContentSectionProps) {
  if (props.loading) return <p className="text-sm text-gray-500">{t("skills_loading")}</p>;
  if (props.error) return <p className="text-sm text-red-600">{props.error}</p>;
  if (props.skills.length === 0) {
    return (
      <div className="rounded border border-dashed p-8 text-center text-sm text-gray-500">
        {t("skills_empty")}
      </div>
    );
  }
  return (
    <SkillsTable
      skills={props.skills}
      onViewDetail={props.onViewDetail}
      onApprove={props.onApprove}
      onDelete={props.onDelete}
    />
  );
}

interface SkillModalsProps {
  workspaceId: string;
  selectedSkill: SkillDetail | null;
  showImportModal: boolean;
  onCloseDetail: () => void;
  onCloseImport: () => void;
  onReloadList: () => void;
  onReloadDetail: (id: string) => void;
}

function SkillModals(props: SkillModalsProps) {
  return (
    <>
      {props.selectedSkill && (
        <SkillDetailDialog
          workspaceId={props.workspaceId}
          skill={props.selectedSkill}
          onClose={props.onCloseDetail}
          onReload={() => {
            props.onReloadList();
            props.onReloadDetail(props.selectedSkill?.id ?? "");
          }}
        />
      )}

      {props.showImportModal && (
        <ImportSkillDialog
          workspaceId={props.workspaceId}
          onClose={props.onCloseImport}
          onSuccess={() => {
            props.onCloseImport();
            props.onReloadList();
          }}
        />
      )}
    </>
  );
}

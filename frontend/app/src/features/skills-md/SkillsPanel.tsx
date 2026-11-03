import { useState } from "react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
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
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-24)", padding: "var(--sp-24)" }}>
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
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
      <div>
        <h2 style={{ margin: 0, fontSize: "var(--fs-18)", fontWeight: 700 }}>{t("skills_title")}</h2>
        <p style={{ margin: "var(--sp-4) 0 0", fontSize: "var(--fs-13)", color: "var(--mu)" }}>{t("skills_desc")}</p>
      </div>
      <Button
        type="button"
        variant="primary"
        onClick={onOpenImport}
      >
        {t("skills_import_btn")}
      </Button>
    </div>
  );
}

function ScriptWarningBanner() {
  return (
    <div
      style={{
        borderRadius: "var(--radius-8)",
        border: "1px solid var(--or)",
        backgroundColor: "var(--orb)",
        padding: "var(--sp-16)",
        fontSize: "var(--fs-13)",
        color: "var(--or)",
      }}
    >
      <p style={{ margin: 0, fontWeight: 500 }}>{t("skills_scripts_warning")}</p>
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
  if (props.loading) return <p style={{ fontSize: "var(--fs-13)", color: "var(--mu)" }}>{t("skills_loading")}</p>;
  if (props.error) return <p style={{ fontSize: "var(--fs-13)", color: "var(--rd)" }}>{props.error}</p>;
  if (props.skills.length === 0) {
    return (
      <div
        style={{
          borderRadius: "var(--radius-8)",
          border: "1px dashed var(--ln)",
          padding: "var(--sp-32)",
          textAlign: "center",
          fontSize: "var(--fs-13)",
          color: "var(--mu)",
        }}
      >
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

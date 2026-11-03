import { t } from "../../core/i18n";
import { PreviewModal } from "./PreviewModal";
import { SyncForm } from "./SyncForm";
import { useInstructionConfig } from "./useInstructionConfig";

interface InstructionSyncPanelProps {
  context?: Record<string, unknown>;
  workspaceId?: string;
  projectId?: string;
}

function SyncHeader() {
  return (
    <>
      <div style={{ borderBottom: "1px solid var(--ln)", paddingBottom: "var(--sp-16)" }}>
        <h2 style={{ margin: 0, fontSize: "var(--fs-18)", fontWeight: 700, color: "var(--fg)" }}>
          {t("instruction_sync_title")}
        </h2>
        <p style={{ margin: "4px 0 0", fontSize: "var(--fs-14)", color: "var(--mu)" }}>
          {t("instruction_sync_desc")}
        </p>
      </div>
      <div style={{ borderRadius: "var(--radius-6)", backgroundColor: "var(--blb)", padding: "var(--sp-16)", fontSize: "var(--fs-12)", color: "var(--bl)" }}>
        {t("instruction_sync_notice_exclude")}
      </div>
    </>
  );
}

export function InstructionSyncPanel(props: InstructionSyncPanelProps) {
  const wsId =
    props.workspaceId ||
    (props.context?.workspaceId as string | undefined) ||
    "default-ws";
  const projId =
    props.projectId ||
    (props.context?.projectId as string | undefined) ||
    "default-proj";

  const cfg = useInstructionConfig(wsId, projId);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-24)", padding: "var(--sp-24)" }}>
      <SyncHeader />
      <SyncForm
        provider={cfg.provider}
        targetFilename={cfg.targetFilename}
        strategy={cfg.strategy}
        isEnabled={cfg.isEnabled}
        loading={cfg.loading}
        statusMsg={cfg.statusMsg}
        onProviderChange={cfg.handleProviderChange}
        onTargetFilenameChange={cfg.setTargetFilename}
        onStrategyChange={cfg.setStrategy}
        onEnabledChange={cfg.setIsEnabled}
        onPreview={() => void cfg.handlePreview()}
        onSave={() => void cfg.handleSave()}
      />
      <PreviewModal preview={cfg.preview} onClose={() => cfg.setPreview(null)} />
    </div>
  );
}

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
      <div className="border-b border-gray-200 pb-4 dark:border-gray-700">
        <h2 className="text-xl font-bold text-gray-900 dark:text-gray-100">
          {t("instruction_sync_title")}
        </h2>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          {t("instruction_sync_desc")}
        </p>
      </div>
      <div className="rounded-md bg-blue-50 p-4 text-xs text-blue-800 dark:bg-blue-950/40 dark:text-blue-300">
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
    <div className="space-y-6 p-6">
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

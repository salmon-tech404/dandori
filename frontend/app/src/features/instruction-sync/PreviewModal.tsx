import { useState } from "react";
import { t } from "../../core/i18n";
import type { PreviewResult } from "./types";

interface PreviewModalProps {
  preview: PreviewResult | null;
  onClose: () => void;
}

interface PreviewHeaderProps {
  preview: PreviewResult;
  copied: boolean;
  onCopy: () => void;
  onClose: () => void;
}

function PreviewHeader({ preview, copied, onCopy, onClose }: PreviewHeaderProps) {
  return (
    <div className="flex items-center justify-between border-b border-gray-200 p-4 dark:border-gray-700">
      <div>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
          {t("instruction_sync_preview_title", { filename: preview.target_filename })}
        </h3>
        <p className="text-xs text-gray-500 dark:text-gray-400">
          {t("instruction_sync_preview_stats", {
            tech: preview.tech_profile_count,
            skills: preview.skills_count,
          })}
        </p>
      </div>
      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={onCopy}
          className="rounded bg-blue-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-blue-700"
        >
          {copied ? t("instruction_sync_preview_copied") : t("instruction_sync_preview_copy")}
        </button>
        <button
          type="button"
          onClick={onClose}
          className="rounded bg-gray-200 px-3 py-1.5 text-xs font-medium text-gray-700 hover:bg-gray-300 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600"
        >
          {t("instruction_sync_preview_close")}
        </button>
      </div>
    </div>
  );
}

function PreviewBody({ content }: { content: string }) {
  return (
    <div className="flex-1 overflow-auto p-4">
      <pre className="font-mono whitespace-pre-wrap rounded bg-gray-50 p-4 text-xs text-gray-800 dark:bg-gray-900 dark:text-gray-200">
        {content}
      </pre>
    </div>
  );
}

export function PreviewModal({ preview, onClose }: PreviewModalProps) {
  const [copied, setCopied] = useState(false);

  if (!preview) {
    return null;
  }

  const handleCopy = () => {
    void navigator.clipboard.writeText(preview.content).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div className="flex max-h-[90vh] w-full max-w-4xl flex-col rounded-lg bg-white shadow-xl dark:bg-gray-800">
        <PreviewHeader
          preview={preview}
          copied={copied}
          onCopy={handleCopy}
          onClose={onClose}
        />
        <PreviewBody content={preview.content} />
      </div>
    </div>
  );
}

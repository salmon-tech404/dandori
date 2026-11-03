import { useState } from "react";
import { t } from "../../core/i18n";
import { Modal } from "../../ui/components/Modal";
import { Button } from "../../ui/components/Button";
import type { PreviewResult } from "./types";

interface PreviewModalProps {
  preview: PreviewResult | null;
  onClose: () => void;
}

function PreviewTitle({ preview }: { preview: PreviewResult }) {
  return (
    <div>
      <div style={{ fontSize: "var(--fs-16)", fontWeight: 600 }}>
        {t("instruction_sync_preview_title", { filename: preview.target_filename })}
      </div>
      <div style={{ fontSize: "var(--fs-12)", color: "var(--mu)", fontWeight: 400 }}>
        {t("instruction_sync_preview_stats", { tech: preview.tech_profile_count, skills: preview.skills_count })}
      </div>
    </div>
  );
}

function PreviewFooter({ copied, onCopy, onClose }: { copied: boolean; onCopy: () => void; onClose: () => void }) {
  return (
    <div style={{ display: "flex", gap: "var(--sp-8)" }}>
      <Button variant="outline" onClick={onCopy}>
        {copied ? t("instruction_sync_preview_copied") : t("instruction_sync_preview_copy")}
      </Button>
      <Button variant="primary" onClick={onClose}>
        {t("instruction_sync_preview_close")}
      </Button>
    </div>
  );
}

export function PreviewModal({ preview, onClose }: PreviewModalProps) {
  const [copied, setCopied] = useState(false);

  if (!preview) return null;

  const handleCopy = () => {
    void navigator.clipboard.writeText(preview.content).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  };

  return (
    <Modal
      open={true}
      onClose={onClose}
      title={<PreviewTitle preview={preview} />}
      size="lg"
      footer={<PreviewFooter copied={copied} onCopy={handleCopy} onClose={onClose} />}
    >
      <pre style={{ margin: 0, padding: "var(--sp-12)", backgroundColor: "var(--sf2)", borderRadius: "var(--radius-8)", fontFamily: "var(--font-mono)", fontSize: "var(--fs-12)", whiteSpace: "pre-wrap", color: "var(--fg)" }}>
        {preview.content}
      </pre>
    </Modal>
  );
}

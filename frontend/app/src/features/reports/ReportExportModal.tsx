import React, { useState } from 'react';
import { t } from '../../core/i18n';
import { Modal } from '../../ui/components/Modal';
import { Button } from '../../ui/components/Button';
import { Textarea } from '../../ui/components/Textarea';

export const ReportExportModal: React.FC<{
  isOpen: boolean;
  markdown: string;
  onClose: () => void;
}> = ({ isOpen, markdown, onClose }) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(markdown);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const footer = (
    <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--sp-8)" }}>
      <Button type="button" variant="primary" size="sm" onClick={handleCopy}>
        {copied ? t('reports_copied') : t('reports_copy_md')}
      </Button>
      <Button type="button" variant="secondary" size="sm" onClick={onClose}>
        {t('reports_close')}
      </Button>
    </div>
  );

  return (
    <Modal open={isOpen} onClose={onClose} title={t('reports_export_title')} footer={footer} size="lg">
      <Textarea
        readOnly
        value={markdown}
        style={{ height: 256, fontFamily: "var(--font-mono)", fontSize: "var(--fs-12)", resize: "none" }}
      />
    </Modal>
  );
};

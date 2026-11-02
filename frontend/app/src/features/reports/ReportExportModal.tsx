import React, { useState } from 'react';
import { t } from '../../core/i18n';

export const ReportExportModal: React.FC<{
  isOpen: boolean;
  markdown: string;
  onClose: () => void;
}> = ({ isOpen, markdown, onClose }) => {
  const [copied, setCopied] = useState(false);

  if (!isOpen) return null;

  const handleCopy = async () => {
    await navigator.clipboard.writeText(markdown);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div className="bg-white dark:bg-slate-900 rounded-lg p-5 max-w-2xl w-full shadow-xl">
        <h3 className="font-bold text-base mb-3">{t('reports_export_title')}</h3>
        <textarea
          readOnly
          value={markdown}
          className="w-full h-64 p-3 font-mono text-xs border rounded bg-slate-50 dark:bg-slate-800 resize-none mb-4"
        />
        <div className="flex justify-end gap-2">
          <button type="button" onClick={handleCopy} className="px-3 py-1.5 bg-blue-600 text-white rounded text-xs">
            {copied ? t('reports_copied') : t('reports_copy_md')}
          </button>
          <button type="button" onClick={onClose} className="px-3 py-1.5 border rounded text-xs">
            {t('reports_close')}
          </button>
        </div>
      </div>
    </div>
  );
};

import { Eye, X } from 'lucide-react';
import React, { useEffect, useState } from 'react';
import { t } from '../../core/i18n';
import { fetchProfilePreview } from './useTechProfile';

interface Props {
  projectId: string;
  workspaceId: string;
  onClose: () => void;
}

const PreviewHeader: React.FC<{ onClose: () => void }> = ({ onClose }) => (
  <div className="flex items-center justify-between pb-3 border-b border-neutral-200 dark:border-neutral-800">
    <div className="flex items-center gap-2">
      <Eye size={18} className="text-indigo-600" />
      <h3 className="text-base font-semibold text-neutral-900 dark:text-neutral-100">{t('tech_profile_preview_title')}</h3>
    </div>
    <button type="button" onClick={onClose} className="text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300">
      <X size={20} />
    </button>
  </div>
);

const PreviewContent: React.FC<{ preview: string }> = ({ preview }) => (
  <div className="mt-4">
    <div className="inline-block mb-2 px-2 py-0.5 rounded bg-amber-50 border border-amber-200 text-amber-800 text-[11px] font-medium dark:bg-amber-950/40 dark:border-amber-900 dark:text-amber-300">
      {t('tech_profile_all_agents_read')}
    </div>
    {preview ? (
      <pre className="p-3 bg-neutral-100 dark:bg-neutral-950 rounded text-xs font-mono whitespace-pre-wrap max-h-80 overflow-y-auto text-neutral-800 dark:text-neutral-200 border border-neutral-200 dark:border-neutral-800">
        {preview}
      </pre>
    ) : (
      <p className="text-xs text-neutral-500 italic py-4">{t('tech_profile_preview_empty')}</p>
    )}
  </div>
);

export const ProfilePreviewModal: React.FC<Props> = ({ projectId, workspaceId, onClose }) => {
  const [preview, setPreview] = useState<string>('');
  const [loading, setLoading] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    setLoading(true);
    fetchProfilePreview(projectId, workspaceId)
      .then(setPreview)
      .catch((e: unknown) => setErr(e instanceof Error ? e.message : String(e)))
      .finally(() => setLoading(false));
  }, [projectId, workspaceId]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div className="w-full max-w-2xl rounded-lg border border-neutral-200 bg-white p-6 shadow-xl dark:border-neutral-800 dark:bg-neutral-900">
        <PreviewHeader onClose={onClose} />
        {loading && <p className="py-4 text-xs text-neutral-500">{t('loading')}</p>}
        {err && <p className="py-2 text-xs text-rose-500">{err}</p>}
        {!loading && <PreviewContent preview={preview} />}
      </div>
    </div>
  );
};

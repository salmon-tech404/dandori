import { RotateCcw, X } from 'lucide-react';
import React, { useEffect, useState } from 'react';
import { t } from '../../core/i18n';
import { PromptSummary, PromptVersion } from './types';
import { fetchPromptVersions, restorePromptVersion } from './usePromptLibrary';

interface Props {
  prompt: PromptSummary;
  onClose: () => void;
  onRestored: () => void;
}

interface ListProps {
  versions: PromptVersion[];
  selectedVer: PromptVersion | null;
  onSelect: (v: PromptVersion) => void;
}

const VersionList: React.FC<ListProps> = ({ versions, selectedVer, onSelect }) => (
  <div className="col-span-1 border-r border-neutral-200 dark:border-neutral-800 pr-3 overflow-y-auto space-y-2">
    {versions.map((v) => (
      <button
        type="button"
        key={v.id}
        onClick={() => onSelect(v)}
        className={`w-full text-left p-2 rounded text-xs transition ${
          selectedVer?.id === v.id
            ? 'bg-indigo-50 border border-indigo-200 text-indigo-700 dark:bg-indigo-950/40 dark:border-indigo-800 dark:text-indigo-300'
            : 'border border-transparent hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-700 dark:text-neutral-300'
        }`}
      >
        <div className="font-semibold">v{v.version}</div>
        <div className="text-[10px] text-neutral-400">{new Date(v.created_at).toLocaleString()}</div>
      </button>
    ))}
  </div>
);

interface DetailProps {
  selectedVer: PromptVersion | null;
  isCurrent: boolean;
  restoring: boolean;
  onRestore: () => void;
}

const VersionDetail: React.FC<DetailProps> = ({ selectedVer, isCurrent, restoring, onRestore }) => {
  if (!selectedVer) return <div className="text-xs text-neutral-400">{t('empty_state')}</div>;

  return (
    <div className="col-span-2 flex flex-col justify-between">
      <div className="flex-1 overflow-y-auto">
        <div className="text-xs font-semibold text-neutral-700 dark:text-neutral-300 mb-2">
          Phiên bản v{selectedVer.version}
        </div>
        <pre className="p-3 bg-neutral-50 dark:bg-neutral-950 rounded text-xs font-mono whitespace-pre-wrap text-neutral-800 dark:text-neutral-200 border border-neutral-200 dark:border-neutral-800">
          {selectedVer.content}
        </pre>
      </div>
      <div className="pt-3 border-t border-neutral-200 dark:border-neutral-800 flex justify-end">
        <button
          type="button"
          disabled={restoring || isCurrent}
          onClick={onRestore}
          className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded bg-amber-600 text-white hover:bg-amber-700 disabled:opacity-50"
        >
          <RotateCcw size={14} />
          <span>{t('prompt_history_restore')}</span>
        </button>
      </div>
    </div>
  );
};

function useVersionHistory(promptId: string, onRestored: () => void) {
  const [versions, setVersions] = useState<PromptVersion[]>([]);
  const [selectedVer, setSelectedVer] = useState<PromptVersion | null>(null);
  const [loading, setLoading] = useState(false);
  const [restoring, setRestoring] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    setLoading(true);
    fetchPromptVersions(promptId)
      .then((vers) => {
        setVersions(vers);
        if (vers.length > 0) setSelectedVer(vers[0]);
      })
      .catch((e: unknown) => setErr(e instanceof Error ? e.message : String(e)))
      .finally(() => setLoading(false));
  }, [promptId]);

  const handleRestore = async () => {
    if (!selectedVer) return;
    setRestoring(true);
    setErr(null);
    try {
      await restorePromptVersion(promptId, selectedVer.version);
      onRestored();
    } catch (e: unknown) {
      setErr(e instanceof Error ? e.message : String(e));
    } finally {
      setRestoring(false);
    }
  };

  return { versions, selectedVer, setSelectedVer, loading, restoring, err, handleRestore };
}

export const VersionHistoryModal: React.FC<Props> = ({ prompt, onClose, onRestored }) => {
  const h = useVersionHistory(prompt.id, onRestored);
  const isCurrent = h.selectedVer?.version === prompt.current_version;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div className="w-full max-w-3xl rounded-lg border border-neutral-200 bg-white p-6 shadow-xl dark:border-neutral-800 dark:bg-neutral-900">
        <div className="flex items-center justify-between pb-3 border-b border-neutral-200 dark:border-neutral-800">
          <h3 className="text-base font-semibold text-neutral-900 dark:text-neutral-100">{t('prompt_history_title', { name: prompt.name })}</h3>
          <button type="button" onClick={onClose} className="text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300">
            <X size={20} />
          </button>
        </div>
        {h.loading && <p className="py-4 text-xs text-neutral-500">{t('loading')}</p>}
        {h.err && <p className="py-2 text-xs text-rose-500">{h.err}</p>}
        {!h.loading && (
          <div className="mt-4 grid grid-cols-3 gap-4 h-80">
            <VersionList versions={h.versions} selectedVer={h.selectedVer} onSelect={h.setSelectedVer} />
            <VersionDetail selectedVer={h.selectedVer} isCurrent={isCurrent} restoring={h.restoring} onRestore={h.handleRestore} />
          </div>
        )}
      </div>
    </div>
  );
};

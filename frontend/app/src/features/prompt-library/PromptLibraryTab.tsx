import { Edit3, History, Play, Plus } from 'lucide-react';
import React, { useState } from 'react';
import { t } from '../../core/i18n';
import { PromptEditorModal } from './PromptEditorModal';
import { TestRenderModal } from './TestRenderModal';
import { PromptDetail, PromptSummary } from './types';
import { fetchPromptDetail, usePromptLibrary } from './usePromptLibrary';
import { VersionHistoryModal } from './VersionHistoryModal';

interface Props {
  workspaceId?: string;
}

interface RowProps {
  prompt: PromptSummary;
  onTest: (p: PromptSummary) => void;
  onHistory: (p: PromptSummary) => void;
  onEdit: (p: PromptSummary) => void;
}

const PromptTableRow: React.FC<RowProps> = ({ prompt: p, onTest, onHistory, onEdit }) => (
  <tr className="hover:bg-neutral-50/50 dark:hover:bg-neutral-800/30">
    <td className="px-4 py-2.5 font-medium text-neutral-900 dark:text-neutral-100">{p.name}</td>
    <td className="px-4 py-2.5">
      <span className="inline-block px-2 py-0.5 rounded-full bg-neutral-100 text-neutral-700 dark:bg-neutral-800 dark:text-neutral-300 text-[11px]">
        {p.role || t('prompt_role_all')}
      </span>
    </td>
    <td className="px-4 py-2.5 text-neutral-500">{p.skill_tag || '-'}</td>
    <td className="px-4 py-2.5">
      <span className="inline-block px-1.5 py-0.5 rounded bg-blue-50 text-blue-700 dark:bg-blue-950/40 dark:text-blue-300 font-mono text-[10px]">
        v{p.current_version}
      </span>
    </td>
    <td className="px-4 py-2.5 text-neutral-500">{new Date(p.updated_at).toLocaleDateString()}</td>
    <td className="px-4 py-2.5 text-right space-x-2">
      <button type="button" onClick={() => onTest(p)} title={t('prompt_btn_test')} className="p-1 text-neutral-500 hover:text-indigo-600 dark:hover:text-indigo-400">
        <Play size={14} />
      </button>
      <button type="button" onClick={() => onHistory(p)} title={t('prompt_btn_history')} className="p-1 text-neutral-500 hover:text-amber-600 dark:hover:text-amber-400">
        <History size={14} />
      </button>
      <button type="button" onClick={() => onEdit(p)} title={t('prompt_btn_edit')} className="p-1 text-neutral-500 hover:text-emerald-600 dark:hover:text-emerald-400">
        <Edit3 size={14} />
      </button>
    </td>
  </tr>
);

interface TableProps {
  prompts: PromptSummary[];
  onTest: (p: PromptSummary) => void;
  onHistory: (p: PromptSummary) => void;
  onEdit: (p: PromptSummary) => void;
}

const PromptTable: React.FC<TableProps> = ({ prompts, onTest, onHistory, onEdit }) => (
  <div className="overflow-x-auto rounded border border-neutral-200 dark:border-neutral-800">
    <table className="w-full text-left text-xs">
      <thead className="bg-neutral-50 border-b border-neutral-200 text-neutral-600 dark:bg-neutral-800/50 dark:border-neutral-700 dark:text-neutral-400">
        <tr>
          <th className="px-4 py-2 font-medium">{t('prompt_col_name')}</th>
          <th className="px-4 py-2 font-medium">{t('prompt_col_role')}</th>
          <th className="px-4 py-2 font-medium">{t('prompt_col_skill')}</th>
          <th className="px-4 py-2 font-medium">{t('prompt_col_version')}</th>
          <th className="px-4 py-2 font-medium">{t('prompt_col_updated')}</th>
          <th className="px-4 py-2 font-medium text-right">{t('prompt_col_actions')}</th>
        </tr>
      </thead>
      <tbody className="divide-y divide-neutral-200 dark:divide-neutral-800">
        {prompts.map((p) => (
          <PromptTableRow key={p.id} prompt={p} onTest={onTest} onHistory={onHistory} onEdit={onEdit} />
        ))}
      </tbody>
    </table>
  </div>
);

const TabHeader: React.FC<{ onNew: () => void }> = ({ onNew }) => (
  <div className="flex items-center justify-between">
    <div>
      <h2 className="text-base font-semibold text-neutral-900 dark:text-neutral-100">{t('prompt_library_title')}</h2>
      <p className="text-xs text-neutral-500 dark:text-neutral-400">{t('prompt_library_desc')}</p>
    </div>
    <button
      type="button"
      onClick={onNew}
      className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded bg-indigo-600 text-white hover:bg-indigo-700"
    >
      <Plus size={14} />
      <span>{t('prompt_btn_create')}</span>
    </button>
  </div>
);

export const PromptLibraryTab: React.FC<Props> = ({ workspaceId = '' }) => {
  const { prompts, loading, error, refresh } = usePromptLibrary(workspaceId);
  const [editorOpen, setEditorOpen] = useState(false);
  const [editingPrompt, setEditingPrompt] = useState<PromptDetail | null>(null);
  const [testContent, setTestContent] = useState<string | null>(null);
  const [testRole, setTestRole] = useState<string | null>(null);
  const [historyPrompt, setHistoryPrompt] = useState<PromptSummary | null>(null);

  const handleOpenEdit = async (p: PromptSummary) => {
    try {
      const detail = await fetchPromptDetail(p.id);
      setEditingPrompt(detail);
      setEditorOpen(true);
    } catch { /* ignore */ }
  };

  const handleOpenTest = async (p: PromptSummary) => {
    try {
      const detail = await fetchPromptDetail(p.id);
      setTestContent(detail.content);
      setTestRole(detail.role);
    } catch { /* ignore */ }
  };

  return (
    <div className="space-y-4 p-4">
      <TabHeader onNew={() => { setEditingPrompt(null); setEditorOpen(true); }} />
      {loading && <p className="text-xs text-neutral-500">{t('loading')}</p>}
      {error && <p className="text-xs text-rose-500">{error}</p>}
      {!loading && prompts.length === 0 && (
        <div className="rounded border border-dashed border-neutral-300 dark:border-neutral-700 p-8 text-center text-xs text-neutral-500">
          {t('prompt_empty')}
        </div>
      )}
      {prompts.length > 0 && <PromptTable prompts={prompts} onTest={handleOpenTest} onHistory={setHistoryPrompt} onEdit={handleOpenEdit} />}
      {editorOpen && <PromptEditorModal initial={editingPrompt} workspaceId={workspaceId} onClose={() => setEditorOpen(false)} onSaved={() => { setEditorOpen(false); refresh(); }} />}
      {testContent !== null && <TestRenderModal content={testContent} defaultRole={testRole} onClose={() => setTestContent(null)} />}
      {historyPrompt && <VersionHistoryModal prompt={historyPrompt} onClose={() => setHistoryPrompt(null)} onRestored={() => { setHistoryPrompt(null); refresh(); }} />}
    </div>
  );
};

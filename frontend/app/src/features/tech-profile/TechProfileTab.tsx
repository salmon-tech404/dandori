import { AlertTriangle, Edit3, Eye, Plus, ShieldCheck, Trash2 } from 'lucide-react';
import React, { useState } from 'react';
import { t } from '../../core/i18n';
import { EntryEditorModal } from './EntryEditorModal';
import { ProfilePreviewModal } from './ProfilePreviewModal';
import { TechProfileEntry } from './types';
import { deleteProfileEntry, useTechProfile } from './useTechProfile';

interface Props {
  context?: Record<string, unknown>;
  projectId?: string;
  workspaceId?: string;
}

const SECRET_PATTERNS = [/sk-[a-zA-Z0-9_-]{8,}/, /ghp_[a-zA-Z0-9]{8,}/, /gho_[a-zA-Z0-9]{8,}/, /glpat-[a-zA-Z0-9_-]{8,}/];

function hasSecret(value: string): boolean {
  return SECRET_PATTERNS.some((p) => p.test(value));
}

const TabHeader: React.FC<{ onPreview: () => void; onNew: () => void }> = ({ onPreview, onNew }) => (
  <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
    <div>
      <div className="flex items-center gap-2">
        <h2 className="text-base font-semibold text-neutral-900 dark:text-neutral-100">{t('tech_profile_title')}</h2>
        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300 text-[11px] font-medium">
          <ShieldCheck size={12} />
          {t('tech_profile_all_agents_read')}
        </span>
      </div>
      <p className="text-xs text-neutral-500 dark:text-neutral-400 mt-0.5">{t('tech_profile_desc')}</p>
    </div>
    <div className="flex items-center gap-2">
      <button
        type="button"
        onClick={onPreview}
        className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded border border-neutral-300 dark:border-neutral-700 text-neutral-700 dark:text-neutral-200 hover:bg-neutral-50 dark:hover:bg-neutral-800"
      >
        <Eye size={14} />
        <span>{t('tech_profile_btn_preview')}</span>
      </button>
      <button
        type="button"
        onClick={onNew}
        className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded bg-indigo-600 text-white hover:bg-indigo-700"
      >
        <Plus size={14} />
        <span>{t('tech_profile_btn_add')}</span>
      </button>
    </div>
  </div>
);

const SecretWarning: React.FC = () => (
  <div className="flex items-start gap-2 p-3 rounded bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800 text-amber-800 dark:text-amber-200 text-xs">
    <AlertTriangle size={16} className="shrink-0 mt-0.5 text-amber-600" />
    <span>{t('tech_profile_warning_secret')}</span>
  </div>
);

const RowActions: React.FC<{ entry: TechProfileEntry; onEdit: (e: TechProfileEntry) => void; onDelete: (e: TechProfileEntry) => void }> = ({
  entry,
  onEdit,
  onDelete,
}) => (
  <td className="px-4 py-2.5 text-right space-x-2">
    <button
      type="button"
      onClick={() => onEdit(entry)}
      title="Sửa"
      className="p-1 text-neutral-500 hover:text-emerald-600 dark:hover:text-emerald-400"
    >
      <Edit3 size={14} />
    </button>
    <button
      type="button"
      onClick={() => onDelete(entry)}
      title="Xóa"
      className="p-1 text-neutral-500 hover:text-rose-600 dark:hover:text-rose-400"
    >
      <Trash2 size={14} />
    </button>
  </td>
);

const EntryRow: React.FC<{ entry: TechProfileEntry; onEdit: (e: TechProfileEntry) => void; onDelete: (e: TechProfileEntry) => void }> = ({
  entry,
  onEdit,
  onDelete,
}) => (
  <tr className="hover:bg-neutral-50/50 dark:hover:bg-neutral-800/30">
    <td className="px-4 py-2.5 font-mono text-neutral-900 dark:text-neutral-100">
      <span className="font-semibold">{entry.key}</span>
      <span className="ml-2 inline-block px-1.5 py-0.5 rounded text-[10px] bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-400">
        {entry.category}
      </span>
    </td>
    <td className="px-4 py-2.5 font-mono text-[11px] text-neutral-600 dark:text-neutral-300 max-w-md truncate">
      {entry.value}
    </td>
    <td className="px-4 py-2.5">
      <span className="inline-block px-1.5 py-0.5 rounded bg-blue-50 text-blue-700 dark:bg-blue-950/40 dark:text-blue-300 font-mono text-[10px]">
        v{entry.version}
      </span>
    </td>
    <td className="px-4 py-2.5 text-neutral-500">{new Date(entry.updated_at).toLocaleDateString()}</td>
    <RowActions entry={entry} onEdit={onEdit} onDelete={onDelete} />
  </tr>
);

interface TableProps {
  entries: TechProfileEntry[];
  onEdit: (e: TechProfileEntry) => void;
  onDelete: (e: TechProfileEntry) => void;
}

const EntryTable: React.FC<TableProps> = ({ entries, onEdit, onDelete }) => (
  <div className="overflow-x-auto rounded border border-neutral-200 dark:border-neutral-800">
    <table className="w-full text-left text-xs">
      <thead className="bg-neutral-50 border-b border-neutral-200 text-neutral-600 dark:bg-neutral-800/50 dark:border-neutral-700 dark:text-neutral-400">
        <tr>
          <th className="px-4 py-2 font-medium">{t('tech_profile_col_key')}</th>
          <th className="px-4 py-2 font-medium">{t('tech_profile_col_value')}</th>
          <th className="px-4 py-2 font-medium">{t('tech_profile_col_version')}</th>
          <th className="px-4 py-2 font-medium">{t('tech_profile_col_updated_at')}</th>
          <th className="px-4 py-2 font-medium text-right">{t('tech_profile_col_actions')}</th>
        </tr>
      </thead>
      <tbody className="divide-y divide-neutral-200 dark:divide-neutral-800">
        {entries.map((e) => (
          <EntryRow key={e.id} entry={e} onEdit={onEdit} onDelete={onDelete} />
        ))}
      </tbody>
    </table>
  </div>
);

export const TechProfileTab: React.FC<Props> = ({ context, projectId, workspaceId }) => {
  const pId = projectId || (context?.projectId as string) || 'default';
  const wId = workspaceId || (context?.workspaceId as string) || '';
  const { entries, loading, error, refresh } = useTechProfile(pId, wId);
  const [editorOpen, setEditorOpen] = useState(false);
  const [editingEntry, setEditingEntry] = useState<TechProfileEntry | null>(null);
  const [previewOpen, setPreviewOpen] = useState(false);

  const handleDelete = async (entry: TechProfileEntry) => {
    try {
      await deleteProfileEntry(entry.id, wId);
      refresh();
    } catch { /* ignore */ }
  };

  const hasAnySecret = entries.some((e) => hasSecret(e.value));

  return (
    <div className="space-y-4 p-4">
      <TabHeader onPreview={() => setPreviewOpen(true)} onNew={() => { setEditingEntry(null); setEditorOpen(true); }} />
      {hasAnySecret && <SecretWarning />}
      {loading && <p className="text-xs text-neutral-500">{t('loading')}</p>}
      {error && <p className="text-xs text-rose-500">{error}</p>}
      {!loading && entries.length === 0 && (
        <div className="rounded border border-dashed border-neutral-300 dark:border-neutral-700 p-8 text-center text-xs text-neutral-500">
          {t('tech_profile_empty')}
        </div>
      )}
      {entries.length > 0 && <EntryTable entries={entries} onEdit={(e) => { setEditingEntry(e); setEditorOpen(true); }} onDelete={handleDelete} />}
      {editorOpen && <EntryEditorModal initial={editingEntry} projectId={pId} workspaceId={wId} onClose={() => setEditorOpen(false)} onSaved={() => { setEditorOpen(false); refresh(); }} />}
      {previewOpen && <ProfilePreviewModal projectId={pId} workspaceId={wId} onClose={() => setPreviewOpen(false)} />}
    </div>
  );
};

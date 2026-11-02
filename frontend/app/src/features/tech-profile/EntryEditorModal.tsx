import { AlertTriangle, X } from 'lucide-react';
import React, { useState } from 'react';
import { t } from '../../core/i18n';
import { TechProfileCategory, TechProfileEntry } from './types';

interface Props {
  initial?: TechProfileEntry | null;
  projectId: string;
  workspaceId: string;
  onClose: () => void;
  onSaved: () => void;
}

const CATEGORIES: TechProfileCategory[] = ['stack', 'commands', 'naming', 'protected_paths', 'architecture', 'notes'];

const SECRET_PATTERNS = [/sk-[a-zA-Z0-9_-]{8,}/, /ghp_[a-zA-Z0-9]{8,}/, /gho_[a-zA-Z0-9]{8,}/, /glpat-[a-zA-Z0-9_-]{8,}/];

function checkSecret(val: string): boolean {
  return SECRET_PATTERNS.some((p) => p.test(val));
}

const CatSelect: React.FC<{ cat: TechProfileCategory; setCat: (c: TechProfileCategory) => void }> = ({ cat, setCat }) => (
  <div>
    <label className="block text-xs font-medium text-neutral-700 dark:text-neutral-300 mb-1">
      {t('tech_profile_dialog_cat_label')}
    </label>
    <select
      value={cat}
      onChange={(e) => setCat(e.target.value as TechProfileCategory)}
      className="w-full rounded border border-neutral-300 px-3 py-1.5 text-xs dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100"
    >
      {CATEGORIES.map((c) => (
        <option key={c} value={c}>
          {c}
        </option>
      ))}
    </select>
  </div>
);

const KeyAndSortInputs: React.FC<{
  keyName: string;
  setKeyName: (k: string) => void;
  sort: number;
  setSort: (s: number) => void;
}> = ({ keyName, setKeyName, sort, setSort }) => (
  <>
    <div>
      <label className="block text-xs font-medium text-neutral-700 dark:text-neutral-300 mb-1">
        {t('tech_profile_dialog_key_label')}
      </label>
      <input
        type="text"
        required
        value={keyName}
        onChange={(e) => setKeyName(e.target.value)}
        className="w-full rounded border border-neutral-300 px-3 py-1.5 text-xs dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100"
      />
    </div>
    <div>
      <label className="block text-xs font-medium text-neutral-700 dark:text-neutral-300 mb-1">
        {t('tech_profile_dialog_sort_label')}
      </label>
      <input
        type="number"
        value={sort}
        onChange={(e) => setSort(parseInt(e.target.value, 10) || 0)}
        className="w-full rounded border border-neutral-300 px-3 py-1.5 text-xs dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100"
      />
    </div>
  </>
);

const ValueInput: React.FC<{
  val: string;
  setVal: (v: string) => void;
  isOverLength: boolean;
  isSecret: boolean;
}> = ({ val, setVal, isOverLength, isSecret }) => (
  <div>
    <div className="flex justify-between items-center mb-1">
      <label className="block text-xs font-medium text-neutral-700 dark:text-neutral-300">
        {t('tech_profile_dialog_val_label')}
      </label>
      <span className={`text-xs ${isOverLength ? 'text-rose-500 font-semibold' : 'text-neutral-500'}`}>
        {val.length}/2.000
      </span>
    </div>
    <textarea
      data-testid="entry-val-input"
      required
      rows={4}
      value={val}
      onChange={(e) => setVal(e.target.value)}
      className="w-full font-mono rounded border border-neutral-300 px-3 py-1.5 text-xs dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100"
    />
    {isSecret && (
      <div className="flex items-center gap-2 p-2 mt-2 rounded bg-rose-50 border border-rose-200 text-rose-700 text-xs dark:bg-rose-950/30 dark:border-rose-900 dark:text-rose-400">
        <AlertTriangle size={14} />
        <span>{t('tech_profile_warning_secret')}</span>
      </div>
    )}
  </div>
);

const ModalHeader: React.FC<{ isEdit: boolean; onClose: () => void }> = ({ isEdit, onClose }) => (
  <div className="flex items-center justify-between pb-3 border-b border-neutral-200 dark:border-neutral-800">
    <h3 className="text-base font-semibold text-neutral-900 dark:text-neutral-100">
      {isEdit ? t('tech_profile_dialog_edit_title') : t('tech_profile_dialog_add_title')}
    </h3>
    <button type="button" onClick={onClose} className="text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300">
      <X size={20} />
    </button>
  </div>
);

async function submitEntry(
  initial: TechProfileEntry | null | undefined,
  projectId: string,
  workspaceId: string,
  body: { category: string; key: string; value: string; sort_order: number }
) {
  const url = initial
    ? `/api/v1/m/tech-profile/entries/${encodeURIComponent(initial.id)}?workspace_id=${encodeURIComponent(workspaceId)}`
    : `/api/v1/m/tech-profile/projects/${encodeURIComponent(projectId)}/entries${workspaceId ? `?workspace_id=${encodeURIComponent(workspaceId)}` : ''}`;
  const res = await fetch(url, {
    method: initial ? 'PUT' : 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
}

function useEntryForm(
  initial: TechProfileEntry | null | undefined,
  projectId: string,
  workspaceId: string,
  onSaved: () => void
) {
  const [cat, setCat] = useState<TechProfileCategory>((initial?.category as TechProfileCategory) || 'stack');
  const [keyName, setKeyName] = useState(initial?.key || '');
  const [val, setVal] = useState(initial?.value || '');
  const [sort, setSort] = useState(initial?.sort_order || 0);
  const [saving, setSaving] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  const isSecret = checkSecret(val);
  const isOverLength = val.length > 2000;

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    if (isSecret || isOverLength) return;
    setSaving(true);
    setErr(null);
    try {
      await submitEntry(initial, projectId, workspaceId, { category: cat, key: keyName, value: val, sort_order: sort });
      onSaved();
    } catch (error) {
      setErr(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  };

  return { cat, setCat, keyName, setKeyName, val, setVal, sort, setSort, saving, err, isSecret, isOverLength, handleSave };
}

export const EntryEditorModal: React.FC<Props> = ({ initial, projectId, workspaceId, onClose, onSaved }) => {
  const f = useEntryForm(initial, projectId, workspaceId, onSaved);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div className="w-full max-w-xl rounded-lg border border-neutral-200 bg-white p-6 shadow-xl dark:border-neutral-800 dark:bg-neutral-900">
        <ModalHeader isEdit={!!initial} onClose={onClose} />
        <form onSubmit={f.handleSave} className="mt-4 space-y-3">
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
            <CatSelect cat={f.cat} setCat={f.setCat} />
            <KeyAndSortInputs keyName={f.keyName} setKeyName={f.setKeyName} sort={f.sort} setSort={f.setSort} />
          </div>
          <ValueInput val={f.val} setVal={f.setVal} isOverLength={f.isOverLength} isSecret={f.isSecret} />
          {f.err && <div className="text-xs text-rose-500">{f.err}</div>}
          <div className="flex justify-end gap-3 pt-3 border-t border-neutral-200 dark:border-neutral-800">
            <button type="button" onClick={onClose} className="px-3 py-1.5 text-xs rounded border border-neutral-300 hover:bg-neutral-100 dark:border-neutral-700 dark:hover:bg-neutral-800">
              {t('btn_cancel')}
            </button>
            <button type="submit" disabled={f.saving || f.isSecret || f.isOverLength} className="px-3 py-1.5 text-xs rounded bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50">
              {f.saving ? t('loading') : t('tech_profile_dialog_btn_save')}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

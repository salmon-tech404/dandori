import { AlertTriangle, X } from 'lucide-react';
import React, { useState } from 'react';
import { t } from '../../core/i18n';
import { PromptDetail } from './types';

interface Props {
  initial?: PromptDetail | null;
  workspaceId: string;
  onClose: () => void;
  onSaved: () => void;
}

function hasSecretWarning(content: string): boolean {
  const lower = content.toLowerCase();
  return lower.includes('sk-') || lower.includes('ghp_') || lower.includes('gho_') || lower.includes('bearer ');
}

interface MetadataProps {
  name: string;
  setName: (v: string) => void;
  role: string;
  setRole: (v: string) => void;
  skillTag: string;
  setSkillTag: (v: string) => void;
}

const MetadataInputs: React.FC<MetadataProps> = ({ name, setName, role, setRole, skillTag, setSkillTag }) => (
  <>
    <div>
      <label className="block text-xs font-medium text-neutral-700 dark:text-neutral-300 mb-1">{t('prompt_editor_name_label')}</label>
      <input
        type="text"
        required
        value={name}
        onChange={(e) => setName(e.target.value)}
        className="w-full rounded border border-neutral-300 px-3 py-2 text-sm dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100"
      />
    </div>
    <div className="grid grid-cols-2 gap-4">
      <div>
        <label className="block text-xs font-medium text-neutral-700 dark:text-neutral-300 mb-1">{t('prompt_editor_role_label')}</label>
        <select
          value={role}
          onChange={(e) => setRole(e.target.value)}
          className="w-full rounded border border-neutral-300 px-3 py-2 text-sm dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100"
        >
          <option value="">{t('prompt_role_all')}</option>
          <option value="pm">{t('prompt_role_pm')}</option>
          <option value="worker">{t('prompt_role_worker')}</option>
          <option value="test">{t('prompt_role_test')}</option>
          <option value="doc">{t('prompt_role_doc')}</option>
        </select>
      </div>
      <div>
        <label className="block text-xs font-medium text-neutral-700 dark:text-neutral-300 mb-1">{t('prompt_editor_skill_label')}</label>
        <input
          type="text"
          value={skillTag}
          onChange={(e) => setSkillTag(e.target.value)}
          placeholder="rust, git, frontend..."
          className="w-full rounded border border-neutral-300 px-3 py-2 text-sm dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100"
        />
      </div>
    </div>
  </>
);

const ContentInput: React.FC<{ content: string; setContent: (v: string) => void; isOverLength: boolean }> = ({ content, setContent, isOverLength }) => (
  <div>
    <div className="flex justify-between items-center mb-1">
      <label className="block text-xs font-medium text-neutral-700 dark:text-neutral-300">{t('prompt_editor_content_label')}</label>
      <span className={`text-xs ${isOverLength ? 'text-rose-500 font-semibold' : 'text-neutral-500'}`}>{content.length}/10.000</span>
    </div>
    <textarea
      data-testid="prompt-content-input"
      required
      rows={8}
      value={content}
      onChange={(e) => setContent(e.target.value)}
      className="w-full font-mono rounded border border-neutral-300 px-3 py-2 text-sm dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100"
    />
  </div>
);

const DialogHeader: React.FC<{ initial?: PromptDetail | null; onClose: () => void }> = ({ initial, onClose }) => (
  <div className="flex items-center justify-between pb-4 border-b border-neutral-200 dark:border-neutral-800">
    <h3 className="text-lg font-semibold text-neutral-900 dark:text-neutral-100">
      {initial ? t('prompt_editor_title_edit') : t('prompt_editor_title_new')}
    </h3>
    <button type="button" onClick={onClose} className="text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300">
      <X size={20} />
    </button>
  </div>
);

const FormFooter: React.FC<{ saving: boolean; disabled: boolean; onClose: () => void }> = ({ saving, disabled, onClose }) => (
  <div className="flex justify-end gap-3 pt-4 border-t border-neutral-200 dark:border-neutral-800">
    <button type="button" onClick={onClose} className="px-4 py-2 text-sm rounded border border-neutral-300 hover:bg-neutral-100 dark:border-neutral-700 dark:hover:bg-neutral-800">
      {t('btn_cancel')}
    </button>
    <button type="submit" disabled={disabled} className="px-4 py-2 text-sm rounded bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50">
      {saving ? t('loading') : t('prompt_editor_btn_save')}
    </button>
  </div>
);

async function submitPrompt(initial: PromptDetail | null | undefined, workspaceId: string, payload: Record<string, unknown>) {
  const url = initial ? `/api/v1/m/prompt-library/prompts/${initial.id}` : '/api/v1/m/prompt-library/prompts';
  const method = initial ? 'PUT' : 'POST';
  const body = initial ? payload : { workspace_id: workspaceId, ...payload };
  const res = await fetch(url, { method, headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  if (!res.ok) {
    const data = await res.json().catch(() => ({}));
    throw new Error(data.error?.message || `HTTP ${res.status}`);
  }
}

function usePromptEditor(initial: PromptDetail | null | undefined, workspaceId: string, onSaved: () => void) {
  const [name, setName] = useState(initial?.name || '');
  const [role, setRole] = useState(initial?.role || '');
  const [skillTag, setSkillTag] = useState(initial?.skill_tag || '');
  const [content, setContent] = useState(initial?.content || '');
  const [saving, setSaving] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  const isSecret = hasSecretWarning(content);
  const isOverLength = content.length > 10000;

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    if (isOverLength || isSecret) return;
    setSaving(true);
    setErr(null);
    try {
      await submitPrompt(initial, workspaceId, { name, role: role || null, skill_tag: skillTag || null, content });
      onSaved();
    } catch (e: unknown) {
      setErr(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  };

  return { name, setName, role, setRole, skillTag, setSkillTag, content, setContent, saving, err, isSecret, isOverLength, handleSave };
}

export const PromptEditorModal: React.FC<Props> = ({ initial, workspaceId, onClose, onSaved }) => {
  const ed = usePromptEditor(initial, workspaceId, onSaved);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div className="w-full max-w-2xl rounded-lg border border-neutral-200 bg-white p-6 shadow-xl dark:border-neutral-800 dark:bg-neutral-900">
        <DialogHeader initial={initial} onClose={onClose} />
        <form onSubmit={ed.handleSave} className="mt-4 space-y-4">
          <MetadataInputs name={ed.name} setName={ed.setName} role={ed.role} setRole={ed.setRole} skillTag={ed.skillTag} setSkillTag={ed.setSkillTag} />
          <ContentInput content={ed.content} setContent={ed.setContent} isOverLength={ed.isOverLength} />
          {ed.isSecret && (
            <div className="flex items-center gap-2 p-2.5 rounded bg-rose-50 border border-rose-200 text-rose-700 text-xs dark:bg-rose-950/30 dark:border-rose-900 dark:text-rose-400">
              <AlertTriangle size={16} />
              <span>{t('prompt_editor_warning_secret')}</span>
            </div>
          )}
          {ed.err && <div className="text-xs text-rose-500">{ed.err}</div>}
          <FormFooter saving={ed.saving} disabled={ed.saving || ed.isSecret || ed.isOverLength} onClose={onClose} />
        </form>
      </div>
    </div>
  );
};

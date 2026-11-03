import { AlertTriangle } from 'lucide-react';
import React, { useState } from 'react';
import { t } from '../../core/i18n';
import { Modal } from '../../ui/components/Modal';
import { Button } from '../../ui/components/Button';
import { Input } from '../../ui/components/Input';
import { Select } from '../../ui/components/Select';
import { Textarea } from '../../ui/components/Textarea';
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

const catOptions = CATEGORIES.map((c) => ({ value: c, label: c }));

const CatSelect: React.FC<{ cat: TechProfileCategory; setCat: (c: TechProfileCategory) => void }> = ({ cat, setCat }) => (
  <div>
    <label style={{ display: 'block', fontSize: 'var(--fs-12)', fontWeight: 500, color: 'var(--mu)', marginBottom: 'var(--sp-4)' }}>
      {t('tech_profile_dialog_cat_label')}
    </label>
    <Select
      value={cat}
      onChange={(e) => setCat(e.target.value as TechProfileCategory)}
      options={catOptions}
    />
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
      <label style={{ display: 'block', fontSize: 'var(--fs-12)', fontWeight: 500, color: 'var(--mu)', marginBottom: 'var(--sp-4)' }}>
        {t('tech_profile_dialog_key_label')}
      </label>
      <Input
        type="text"
        required
        value={keyName}
        onChange={(e) => setKeyName(e.target.value)}
      />
    </div>
    <div>
      <label style={{ display: 'block', fontSize: 'var(--fs-12)', fontWeight: 500, color: 'var(--mu)', marginBottom: 'var(--sp-4)' }}>
        {t('tech_profile_dialog_sort_label')}
      </label>
      <Input
        type="number"
        value={sort}
        onChange={(e) => setSort(parseInt(e.target.value, 10) || 0)}
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
    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 'var(--sp-4)' }}>
      <label style={{ display: 'block', fontSize: 'var(--fs-12)', fontWeight: 500, color: 'var(--mu)' }}>
        {t('tech_profile_dialog_val_label')}
      </label>
      <span style={{ fontSize: 'var(--fs-12)', fontWeight: isOverLength ? 600 : 'normal', color: isOverLength ? 'var(--rd)' : 'var(--mu)' }}>
        {val.length}/2.000
      </span>
    </div>
    <Textarea
      data-testid="entry-val-input"
      required
      rows={4}
      value={val}
      onChange={(e) => setVal(e.target.value)}
      style={{ fontFamily: 'var(--font-mono)', fontSize: 'var(--fs-12)' }}
    />
    {isSecret && (
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 'var(--sp-4)',
          padding: 'var(--sp-4)',
          marginTop: 'var(--sp-4)',
          borderRadius: 'var(--radius-6)',
          backgroundColor: 'rgba(239, 68, 68, 0.1)',
          border: '1px solid var(--rd)',
          color: 'var(--rd)',
          fontSize: 'var(--fs-12)',
        }}
      >
        <AlertTriangle size={14} />
        <span>{t('tech_profile_warning_secret')}</span>
      </div>
    )}
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
  const title = initial ? t('tech_profile_dialog_edit_title') : t('tech_profile_dialog_add_title');

  return (
    <Modal open onClose={onClose} title={title} size="lg">
      <form onSubmit={f.handleSave} style={{ display: 'flex', flexDirection: 'column', gap: 'var(--sp-16)' }}>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: 'var(--sp-16)' }}>
          <CatSelect cat={f.cat} setCat={f.setCat} />
          <KeyAndSortInputs keyName={f.keyName} setKeyName={f.setKeyName} sort={f.sort} setSort={f.setSort} />
        </div>
        <ValueInput val={f.val} setVal={f.setVal} isOverLength={f.isOverLength} isSecret={f.isSecret} />
        {f.err && <div style={{ fontSize: 'var(--fs-12)', color: 'var(--rd)' }}>{f.err}</div>}
        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 'var(--sp-16)', paddingTop: 'var(--sp-16)', borderTop: '1px solid var(--ln)' }}>
          <Button type="button" variant="secondary" onClick={onClose}>
            {t('btn_cancel')}
          </Button>
          <Button type="submit" variant="primary" disabled={f.saving || f.isSecret || f.isOverLength}>
            {f.saving ? t('loading') : t('tech_profile_dialog_btn_save')}
          </Button>
        </div>
      </form>
    </Modal>
  );
};

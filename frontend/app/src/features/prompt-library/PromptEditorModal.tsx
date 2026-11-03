import { AlertTriangle } from 'lucide-react';
import React, { useState } from 'react';
import { t } from '../../core/i18n';
import { Modal } from '../../ui/components/Modal';
import { Button } from '../../ui/components/Button';
import { Input } from '../../ui/components/Input';
import { Select } from '../../ui/components/Select';
import { Textarea } from '../../ui/components/Textarea';
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
  <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)" }}>
    <Input
      label={t('prompt_editor_name_label')}
      type="text"
      required
      value={name}
      onChange={(e) => setName(e.target.value)}
    />
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--sp-12)" }}>
      <Select
        label={t('prompt_editor_role_label')}
        value={role}
        onChange={(e) => setRole(e.target.value)}
        options={[
          { value: "", label: t('prompt_role_all') },
          { value: "pm", label: t('prompt_role_pm') },
          { value: "worker", label: t('prompt_role_worker') },
          { value: "test", label: t('prompt_role_test') },
          { value: "doc", label: t('prompt_role_doc') },
        ]}
      />
      <Input
        label={t('prompt_editor_skill_label')}
        type="text"
        value={skillTag}
        onChange={(e) => setSkillTag(e.target.value)}
        placeholder={t("prompt_tags_placeholder")}
      />
    </div>
  </div>
);

const ContentInput: React.FC<{ content: string; setContent: (v: string) => void; isOverLength: boolean }> = ({ content, setContent, isOverLength }) => (
  <div>
    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "var(--sp-4)" }}>
      <span style={{ fontSize: "var(--fs-13)", fontWeight: 500, color: "var(--mu)" }}>{t('prompt_editor_content_label')}</span>
      <span style={{ fontSize: "var(--fs-12)", color: isOverLength ? "var(--rd)" : "var(--mu)", fontWeight: isOverLength ? 600 : 400 }}>{content.length}/10.000</span>
    </div>
    <Textarea
      data-testid="prompt-content-input"
      required
      mono={true}
      rows={8}
      value={content}
      onChange={(e) => setContent(e.target.value)}
    />
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
    <Modal
      open={true}
      onClose={onClose}
      title={initial ? t('prompt_editor_title_edit') : t('prompt_editor_title_new')}
      size="lg"
      footer={
        <div style={{ display: "flex", gap: "var(--sp-8)" }}>
          <Button variant="ghost" onClick={onClose}>{t('btn_cancel')}</Button>
          <Button variant="primary" loading={ed.saving} disabled={ed.saving || ed.isSecret || ed.isOverLength} onClick={ed.handleSave}>
            {t('prompt_editor_btn_save')}
          </Button>
        </div>
      }
    >
      <form onSubmit={ed.handleSave} style={{ display: "flex", flexDirection: "column", gap: "var(--sp-16)" }}>
        <MetadataInputs name={ed.name} setName={ed.setName} role={ed.role} setRole={ed.setRole} skillTag={ed.skillTag} setSkillTag={ed.setSkillTag} />
        <ContentInput content={ed.content} setContent={ed.setContent} isOverLength={ed.isOverLength} />
        {ed.isSecret && (
          <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)", padding: "var(--sp-8) var(--sp-12)", borderRadius: "var(--radius-6)", backgroundColor: "var(--rdb)", border: "1px solid var(--rd)", color: "var(--rd)", fontSize: "var(--fs-12)" }}>
            <AlertTriangle size={16} />
            <span>{t('prompt_editor_warning_secret')}</span>
          </div>
        )}
        {ed.err && <div style={{ fontSize: "var(--fs-12)", color: "var(--rd)" }}>{ed.err}</div>}
      </form>
    </Modal>
  );
};

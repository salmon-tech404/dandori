import { Edit3, History, Play, Plus } from 'lucide-react';
import React, { useState } from 'react';
import { t } from '../../core/i18n';
import { Button } from '../../ui/components/Button';
import { IconButton } from '../../ui/components/IconButton';
import { Tooltip } from '../../ui/components/Tooltip';
import { PromptEditorModal } from './PromptEditorModal';
import { RolePromptSection } from './RolePromptSection';
import { TestRenderModal } from './TestRenderModal';
import { PromptDetail, PromptSummary } from './types';
import { fetchPromptDetail, usePromptLibrary } from './usePromptLibrary';
import { VersionHistoryModal } from './VersionHistoryModal';

interface Props {
  context?: Record<string, unknown>;
  workspaceId?: string;
}

interface RowProps {
  prompt: PromptSummary;
  onTest: (p: PromptSummary) => void;
  onHistory: (p: PromptSummary) => void;
  onEdit: (p: PromptSummary) => void;
}

const PromptTableRow: React.FC<RowProps> = ({ prompt: p, onTest, onHistory, onEdit }) => (
  <tr style={{ borderBottom: "1px solid var(--ln)" }}>
    <td style={{ padding: "10px 16px", fontWeight: 500, color: "var(--fg)" }}>{p.name}</td>
    <td style={{ padding: "10px 16px" }}>
      <span style={{ display: "inline-block", padding: "2px 8px", borderRadius: "var(--radius-pill)", backgroundColor: "var(--sf2)", color: "var(--fg)", fontSize: "11px" }}>
        {p.role || t('prompt_role_all')}
      </span>
    </td>
    <td style={{ padding: "10px 16px", color: "var(--mu)" }}>{p.skill_tag || '-'}</td>
    <td style={{ padding: "10px 16px" }}>
      <span style={{ display: "inline-block", padding: "2px 6px", borderRadius: "var(--radius-4)", backgroundColor: "var(--blb)", color: "var(--bl)", fontFamily: "var(--font-mono)", fontSize: "10px" }}>{t("version_prefix")}{p.current_version}</span>
    </td>
    <td style={{ padding: "10px 16px", color: "var(--mu)" }}>{new Date(p.updated_at).toLocaleDateString()}</td>
    <td style={{ padding: "10px 16px", textAlign: "right" }}>
      <div style={{ display: "inline-flex", gap: "var(--sp-8)" }}>
        <Tooltip content={t('prompt_btn_test')}><IconButton variant="ghost" size="sm" type="button" aria-label={t('prompt_btn_test')} onClick={() => onTest(p)} icon={<Play size={14} />} /></Tooltip>
        <Tooltip content={t('prompt_btn_history')}><IconButton variant="ghost" size="sm" type="button" aria-label={t('prompt_btn_history')} onClick={() => onHistory(p)} icon={<History size={14} />} /></Tooltip>
        <Tooltip content={t('prompt_btn_edit')}><IconButton variant="ghost" size="sm" type="button" aria-label={t('prompt_btn_edit')} onClick={() => onEdit(p)} icon={<Edit3 size={14} />} /></Tooltip>
      </div>
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
  <div style={{ overflowX: "auto", borderRadius: "var(--radius-8)", border: "1px solid var(--ln)", backgroundColor: "var(--sf)" }}>
    <table style={{ width: "100%", textAlign: "left", fontSize: "var(--fs-12)", borderCollapse: "collapse" }}>
      <thead style={{ backgroundColor: "var(--sf2)", borderBottom: "1px solid var(--ln)", color: "var(--mu)" }}>
        <tr>
          <th style={{ padding: "8px 16px", fontWeight: 500 }}>{t('prompt_col_name')}</th>
          <th style={{ padding: "8px 16px", fontWeight: 500 }}>{t('prompt_col_role')}</th>
          <th style={{ padding: "8px 16px", fontWeight: 500 }}>{t('prompt_col_skill')}</th>
          <th style={{ padding: "8px 16px", fontWeight: 500 }}>{t('prompt_col_version')}</th>
          <th style={{ padding: "8px 16px", fontWeight: 500 }}>{t('prompt_col_updated')}</th>
          <th style={{ padding: "8px 16px", fontWeight: 500, textAlign: "right" }}>{t('prompt_col_actions')}</th>
        </tr>
      </thead>
      <tbody>
        {prompts.map((p) => (
          <PromptTableRow key={p.id} prompt={p} onTest={onTest} onHistory={onHistory} onEdit={onEdit} />
        ))}
      </tbody>
    </table>
  </div>
);

const TabHeader: React.FC<{ onNew: () => void }> = ({ onNew }) => (
  <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
    <div>
      <h2 style={{ margin: 0, fontSize: "var(--fs-16)", fontWeight: 600, color: "var(--fg)" }}>{t('prompt_library_title')}</h2>
      <p style={{ margin: "2px 0 0", fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t('prompt_library_desc')}</p>
    </div>
    <Button variant="primary" size="sm" icon={<Plus size={14} />} type="button" onClick={onNew}>{t('prompt_btn_create')}</Button>
  </div>
);

function ModalsContainer({
  editorOpen,
  editingPrompt,
  testContent,
  historyPrompt,
  workspaceId,
  onCloseEditor,
  onSaved,
  onCloseTest,
  onCloseHistory,
  onRestored,
}: {
  editorOpen: boolean;
  editingPrompt: PromptDetail | null;
  testContent: string | null;
  historyPrompt: PromptSummary | null;
  workspaceId: string;
  onCloseEditor: () => void;
  onSaved: () => void;
  onCloseTest: () => void;
  onCloseHistory: () => void;
  onRestored: () => void;
}) {
  return (
    <>
      {editorOpen && <PromptEditorModal initial={editingPrompt} workspaceId={workspaceId} onClose={onCloseEditor} onSaved={onSaved} />}
      {testContent !== null && <TestRenderModal content={testContent} onClose={onCloseTest} />}
      {historyPrompt && <VersionHistoryModal prompt={historyPrompt} onClose={onCloseHistory} onRestored={onRestored} />}
    </>
  );
}

export const PromptLibraryTab: React.FC<Props> = ({ context, workspaceId = '' }) => {
  const wsId = workspaceId || (context?.workspaceId as string) || '';
  const activeRole = (context?.role as string) || 'pm';
  const { prompts, loading, error, refresh } = usePromptLibrary(wsId);
  const [editorOpen, setEditorOpen] = useState(false);
  const [editingPrompt, setEditingPrompt] = useState<PromptDetail | null>(null);
  const [testContent, setTestContent] = useState<string | null>(null);
  const [historyPrompt, setHistoryPrompt] = useState<PromptSummary | null>(null);

  const handleEdit = async (p: PromptSummary) => {
    try {
      const detail = await fetchPromptDetail(p.id);
      setEditingPrompt(detail);
      setEditorOpen(true);
    } catch {}
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-16)", padding: "var(--sp-16)" }}>
      <RolePromptSection role={activeRole} prompts={prompts} workspaceId={wsId} onTest={setTestContent} onSaved={refresh} />
      <TabHeader onNew={() => { setEditingPrompt(null); setEditorOpen(true); }} />
      {loading && <p style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t('loading')}</p>}
      {error && <p style={{ fontSize: "var(--fs-12)", color: "var(--rd)" }}>{error}</p>}
      {prompts.length > 0 && <PromptTable prompts={prompts} onTest={(p) => void handleEdit(p)} onHistory={setHistoryPrompt} onEdit={handleEdit} />}
      <ModalsContainer
        editorOpen={editorOpen} editingPrompt={editingPrompt} testContent={testContent} historyPrompt={historyPrompt}
        workspaceId={wsId} onCloseEditor={() => setEditorOpen(false)} onSaved={() => { setEditorOpen(false); refresh(); }}
        onCloseTest={() => setTestContent(null)} onCloseHistory={() => setHistoryPrompt(null)} onRestored={() => { setHistoryPrompt(null); refresh(); }}
      />
    </div>
  );
};

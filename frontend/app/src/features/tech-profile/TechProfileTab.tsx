import { AlertTriangle, Edit3, Eye, Plus, ShieldCheck, Trash2 } from 'lucide-react';
import React, { useState } from 'react';
import { t } from '../../core/i18n';
import { Button } from '../../ui/components/Button';
import { IconButton } from '../../ui/components/IconButton';
import { Tooltip } from '../../ui/components/Tooltip';
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

const TabTitle: React.FC = () => (
  <div>
    <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--sp-4)' }}>
      <h2 style={{ margin: 0, fontSize: 'var(--fs-14)', fontWeight: 600, color: 'var(--fg)' }}>{t('tech_profile_title')}</h2>
      <span
        style={{
          display: 'inline-flex',
          alignItems: 'center',
          gap: '4px',
          padding: '2px 8px',
          borderRadius: '9999px',
          backgroundColor: 'rgba(16, 185, 129, 0.15)',
          color: 'var(--gr)',
          fontSize: '11px',
          fontWeight: 500,
        }}
      >
        <ShieldCheck size={12} />
        {t('tech_profile_all_agents_read')}
      </span>
    </div>
    <p style={{ margin: '2px 0 0', fontSize: 'var(--fs-12)', color: 'var(--mu)' }}>{t('tech_profile_desc')}</p>
  </div>
);

const TabButtons: React.FC<{ onPreview: () => void; onNew: () => void }> = ({ onPreview, onNew }) => (
  <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--sp-4)' }}>
    <Button
      type="button"
      variant="secondary"
      size="sm"
      icon={<Eye size={14} />}
      onClick={onPreview}
    >
      {t('tech_profile_btn_preview')}
    </Button>
    <Button
      type="button"
      variant="primary"
      size="sm"
      icon={<Plus size={14} />}
      onClick={onNew}
    >
      {t('tech_profile_btn_add')}
    </Button>
  </div>
);

const TabHeader: React.FC<{ onPreview: () => void; onNew: () => void }> = ({ onPreview, onNew }) => (
  <div style={{ display: 'flex', flexWrap: 'wrap', alignItems: 'center', justifyContent: 'space-between', gap: 'var(--sp-16)' }}>
    <TabTitle />
    <TabButtons onPreview={onPreview} onNew={onNew} />
  </div>
);

const SecretWarning: React.FC = () => (
  <div
    style={{
      display: 'flex',
      alignItems: 'flex-start',
      gap: 'var(--sp-4)',
      padding: 'var(--sp-8)',
      borderRadius: 'var(--radius-6)',
      backgroundColor: 'rgba(245, 158, 11, 0.1)',
      border: '1px solid var(--or)',
      color: 'var(--or)',
      fontSize: 'var(--fs-12)',
    }}
  >
    <AlertTriangle size={16} style={{ flexShrink: 0, marginTop: '2px', color: 'var(--or)' }} />
    <span>{t('tech_profile_warning_secret')}</span>
  </div>
);

const RowActions: React.FC<{ entry: TechProfileEntry; onEdit: (e: TechProfileEntry) => void; onDelete: (e: TechProfileEntry) => void }> = ({
  entry,
  onEdit,
  onDelete,
}) => (
  <td style={{ padding: 'var(--sp-8) var(--sp-16)', textAlign: 'right' }}>
    <div style={{ display: 'inline-flex', gap: 'var(--sp-4)', justifyContent: 'flex-end' }}>
      <Tooltip content={t("btn_edit")}>
        <IconButton
          icon={<Edit3 size={14} />}
          aria-label={t("btn_edit")}
          onClick={() => onEdit(entry)}
        />
      </Tooltip>
      <Tooltip content={t("btn_delete")}>
        <IconButton
          icon={<Trash2 size={14} />}
          aria-label={t("btn_delete")}
          onClick={() => onDelete(entry)}
        />
      </Tooltip>
    </div>
  </td>
);

const rowStyle: React.CSSProperties = { borderBottom: '1px solid var(--ln)' };
const cellStyle: React.CSSProperties = { padding: 'var(--sp-8) var(--sp-16)' };
const keyCellStyle: React.CSSProperties = { ...cellStyle, fontFamily: 'var(--font-mono)', color: 'var(--fg)' };
const valCellStyle: React.CSSProperties = {
  ...cellStyle,
  fontFamily: 'var(--font-mono)',
  fontSize: '11px',
  color: 'var(--mu)',
  maxWidth: '280px',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
};

const catBadgeStyle: React.CSSProperties = {
  marginLeft: 'var(--sp-4)',
  display: 'inline-block',
  padding: '2px 6px',
  borderRadius: 'var(--radius-6)',
  fontSize: '10px',
  backgroundColor: 'var(--sf2)',
  color: 'var(--mu)',
};

const verBadgeStyle: React.CSSProperties = {
  display: 'inline-block',
  padding: '2px 6px',
  borderRadius: 'var(--radius-6)',
  backgroundColor: 'rgba(59, 130, 246, 0.1)',
  color: 'var(--ac)',
  fontFamily: 'var(--font-mono)',
  fontSize: '10px',
};

const EntryRow: React.FC<{ entry: TechProfileEntry; onEdit: (e: TechProfileEntry) => void; onDelete: (e: TechProfileEntry) => void }> = ({
  entry,
  onEdit,
  onDelete,
}) => (
  <tr style={rowStyle}>
    <td style={keyCellStyle}>
      <span style={{ fontWeight: 600 }}>{entry.key}</span>
      <span style={catBadgeStyle}>{entry.category}</span>
    </td>
    <td style={valCellStyle}>{entry.value}</td>
    <td style={cellStyle}>
      <span style={verBadgeStyle}>{t("version_prefix")}{entry.version}</span>
    </td>
    <td style={{ ...cellStyle, color: 'var(--mu)' }}>{new Date(entry.updated_at).toLocaleDateString()}</td>
    <RowActions entry={entry} onEdit={onEdit} onDelete={onDelete} />
  </tr>
);

interface TableProps {
  entries: TechProfileEntry[];
  onEdit: (e: TechProfileEntry) => void;
  onDelete: (e: TechProfileEntry) => void;
}

const EntryTableHeader: React.FC = () => (
  <thead
    style={{
      backgroundColor: 'var(--sf2)',
      borderBottom: '1px solid var(--ln)',
      color: 'var(--mu)',
    }}
  >
    <tr>
      <th style={{ padding: 'var(--sp-8) var(--sp-16)', fontWeight: 500 }}>{t('tech_profile_col_key')}</th>
      <th style={{ padding: 'var(--sp-8) var(--sp-16)', fontWeight: 500 }}>{t('tech_profile_col_value')}</th>
      <th style={{ padding: 'var(--sp-8) var(--sp-16)', fontWeight: 500 }}>{t('tech_profile_col_version')}</th>
      <th style={{ padding: 'var(--sp-8) var(--sp-16)', fontWeight: 500 }}>{t('tech_profile_col_updated_at')}</th>
      <th style={{ padding: 'var(--sp-8) var(--sp-16)', fontWeight: 500, textAlign: 'right' }}>{t('tech_profile_col_actions')}</th>
    </tr>
  </thead>
);

const EntryTable: React.FC<TableProps> = ({ entries, onEdit, onDelete }) => (
  <div style={{ overflowX: 'auto', borderRadius: 'var(--radius-8)', border: '1px solid var(--ln)' }}>
    <table style={{ width: '100%', textAlign: 'left', fontSize: 'var(--fs-12)', borderCollapse: 'collapse' }}>
      <EntryTableHeader />
      <tbody>
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
    <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--sp-16)', padding: 'var(--sp-16)' }}>
      <TabHeader onPreview={() => setPreviewOpen(true)} onNew={() => { setEditingEntry(null); setEditorOpen(true); }} />
      {hasAnySecret && <SecretWarning />}
      {loading && <p style={{ fontSize: 'var(--fs-12)', color: 'var(--mu)' }}>{t('loading')}</p>}
      {error && <p style={{ fontSize: 'var(--fs-12)', color: 'var(--rd)' }}>{error}</p>}
      {!loading && entries.length === 0 && (
        <div style={{ borderRadius: 'var(--radius-8)', border: '1px dashed var(--ln)', padding: 'var(--sp-32)', textAlign: 'center', fontSize: 'var(--fs-12)', color: 'var(--mu)' }}>
          {t('tech_profile_empty')}
        </div>
      )}
      {entries.length > 0 && <EntryTable entries={entries} onEdit={(e) => { setEditingEntry(e); setEditorOpen(true); }} onDelete={handleDelete} />}
      {editorOpen && <EntryEditorModal initial={editingEntry} projectId={pId} workspaceId={wId} onClose={() => setEditorOpen(false)} onSaved={() => { setEditorOpen(false); refresh(); }} />}
      {previewOpen && <ProfilePreviewModal projectId={pId} workspaceId={wId} onClose={() => setPreviewOpen(false)} />}
    </div>
  );
};

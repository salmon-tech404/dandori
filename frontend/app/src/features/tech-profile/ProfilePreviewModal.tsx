import React, { useEffect, useState } from 'react';
import { t } from '../../core/i18n';
import { Modal } from '../../ui/components/Modal';
import { fetchProfilePreview } from './useTechProfile';

interface Props {
  projectId: string;
  workspaceId: string;
  onClose: () => void;
}

const preStyle: React.CSSProperties = {
  padding: 'var(--sp-16)',
  backgroundColor: 'var(--sf2)',
  borderRadius: 'var(--radius-8)',
  fontSize: 'var(--fs-12)',
  fontFamily: 'var(--font-mono)',
  whiteSpace: 'pre-wrap',
  maxHeight: '320px',
  overflowY: 'auto',
  color: 'var(--fg)',
  border: '1px solid var(--ln)',
};

const badgeStyle: React.CSSProperties = {
  display: 'inline-block',
  marginBottom: 'var(--sp-4)',
  padding: '2px 8px',
  borderRadius: 'var(--radius-6)',
  backgroundColor: 'rgba(245, 158, 11, 0.15)',
  border: '1px solid var(--or)',
  color: 'var(--or)',
  fontSize: '11px',
  fontWeight: 500,
};

const PreviewContent: React.FC<{ preview: string }> = ({ preview }) => (
  <div>
    <div style={badgeStyle}>
      {t('tech_profile_all_agents_read')}
    </div>
    {preview ? (
      <pre style={preStyle}>{preview}</pre>
    ) : (
      <p style={{ fontSize: 'var(--fs-12)', color: 'var(--mu)', fontStyle: 'italic', padding: 'var(--sp-16) 0' }}>
        {t('tech_profile_preview_empty')}
      </p>
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
    <Modal open onClose={onClose} title={t('tech_profile_preview_title')} size="lg">
      {loading && <p style={{ padding: 'var(--sp-16) 0', fontSize: 'var(--fs-12)', color: 'var(--mu)' }}>{t('loading')}</p>}
      {err && <p style={{ padding: 'var(--sp-4) 0', fontSize: 'var(--fs-12)', color: 'var(--rd)' }}>{err}</p>}
      {!loading && <PreviewContent preview={preview} />}
    </Modal>
  );
};

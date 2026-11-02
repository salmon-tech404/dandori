import { t } from '../../../core/i18n';
import type { RunSummary } from '../types';

interface RowProps {
  run: RunSummary;
  onOpenReplay: (id: string) => void;
  onOpenNote: (id: string) => void;
  onExport: (id: string, format: 'markdown' | 'json') => void;
}

const getStatusStyle = (status: string) => {
  if (status === 'success') return { bg: 'var(--grb)', color: 'var(--gr)' };
  if (status === 'failed') return { bg: 'var(--rdb)', color: 'var(--rd)' };
  return { bg: 'var(--blb)', color: 'var(--bl)' };
};

const TagList = ({ tags }: { tags: string[] }) => (
  <div style={{ display: 'flex', gap: '4px', marginTop: '4px' }}>
    {tags.map((tg) => (
      <span
        key={tg}
        style={{
          fontSize: '10px',
          padding: '2px 6px',
          borderRadius: '4px',
          backgroundColor: 'var(--sf2)',
          color: 'var(--mu)',
        }}
      >
        {tg}
      </span>
    ))}
  </div>
);

const RowActions = ({
  id,
  onOpenReplay,
  onOpenNote,
  onExport,
}: {
  id: string;
  onOpenReplay: (id: string) => void;
  onOpenNote: (id: string) => void;
  onExport: (id: string, format: 'markdown' | 'json') => void;
}) => {
  const btnStyle = {
    padding: '4px 8px',
    minHeight: '44px',
    borderRadius: '6px',
    border: '1px solid var(--ln)',
    backgroundColor: 'var(--sf)',
  };
  return (
    <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '6px' }}>
      <button type="button" onClick={() => onOpenReplay(id)} style={btnStyle}>
        {t('run_history_btn_replay')}
      </button>
      <button type="button" onClick={() => onOpenNote(id)} style={btnStyle}>
        {t('run_history_note_label')}
      </button>
      <button type="button" onClick={() => onExport(id, 'markdown')} style={btnStyle}>
        {t('run_history_btn_export_md')}
      </button>
      <button type="button" onClick={() => onExport(id, 'json')} style={btnStyle}>
        {t('run_history_btn_export_json')}
      </button>
    </div>
  );
};

export const RunTableRow = ({ run, onOpenReplay, onOpenNote, onExport }: RowProps) => {
  const st = getStatusStyle(run.status);
  const dur = run.duration_seconds !== null && run.duration_seconds !== undefined ? `${run.duration_seconds}s` : '—';
  const exitCode = run.exit_code !== null && run.exit_code !== undefined ? run.exit_code : '—';
  return (
    <tr style={{ borderBottom: '1px solid var(--ln)' }}>
      <td style={{ padding: '10px 12px', fontFamily: 'monospace' }}>
        <div>{run.id}</div>
        {run.note && <div style={{ fontSize: '12px', color: 'var(--mu)', marginTop: '2px' }}>{run.note}</div>}
        {run.tags && run.tags.length > 0 && <TagList tags={run.tags} />}
      </td>
      <td style={{ padding: '10px 12px' }}>{run.agent} ({run.account})</td>
      <td style={{ padding: '10px 12px' }}>
        <span style={{ padding: '2px 8px', borderRadius: '999px', fontSize: '11px', fontWeight: 600, backgroundColor: st.bg, color: st.color }}>
          {run.status}
        </span>
      </td>
      <td style={{ padding: '10px 12px', color: 'var(--mu)' }}>{run.started_at}</td>
      <td style={{ padding: '10px 12px' }}>{dur}</td>
      <td style={{ padding: '10px 12px' }}>{exitCode}</td>
      <td style={{ padding: '10px 12px', textAlign: 'right' }}>
        <RowActions id={run.id} onOpenReplay={onOpenReplay} onOpenNote={onOpenNote} onExport={onExport} />
      </td>
    </tr>
  );
};

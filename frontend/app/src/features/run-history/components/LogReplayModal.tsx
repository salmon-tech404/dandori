import { t } from '../../../core/i18n';
import type { RunEventItem } from '../types';

interface ModalProps {
  runId: string;
  events: RunEventItem[];
  loading: boolean;
  onClose: () => void;
}

const EventItem = ({ ev }: { ev: RunEventItem }) => (
  <div
    style={{
      padding: '8px 12px',
      backgroundColor: 'var(--sf2)',
      borderRadius: '6px',
      borderLeft: '3px solid var(--bl)',
      wordBreak: 'break-word',
    }}
  >
    <div style={{ display: 'flex', gap: '8px', color: 'var(--mu)', marginBottom: '4px' }}>
      <span>#{ev.seq}</span>
      <span>[{ev.event_type}]</span>
      <span>{ev.created_at}</span>
    </div>
    <div style={{ whiteSpace: 'pre-wrap', color: 'var(--fg)' }}>{ev.data}</div>
  </div>
);

const ModalHeader = ({ runId, onClose }: { runId: string; onClose: () => void }) => (
  <div
    style={{
      padding: '16px',
      borderBottom: '1px solid var(--ln)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-between',
    }}
  >
    <div>
      <h3 style={{ margin: 0, fontSize: '16px', fontWeight: 600 }}>
        {t('run_history_replay_title')}
      </h3>
      <span style={{ fontSize: '12px', color: 'var(--mu)', fontFamily: 'monospace' }}>
        {runId}
      </span>
    </div>
    <button
      type="button"
      onClick={onClose}
      style={{
        padding: '6px 12px',
        minHeight: '44px',
        border: '1px solid var(--ln)',
        borderRadius: '8px',
        backgroundColor: 'var(--sf2)',
        cursor: 'pointer',
        color: 'var(--fg)',
      }}
    >
      {t('btn_cancel')}
    </button>
  </div>
);

const EventList = ({ events, loading }: { events: RunEventItem[]; loading: boolean }) => {
  if (loading) {
    return <p style={{ color: 'var(--mu)', textAlign: 'center' }}>{t('run_history_loading')}</p>;
  }
  if (events.length === 0) {
    return <p style={{ color: 'var(--mu)', textAlign: 'center' }}>{t('run_history_replay_empty')}</p>;
  }
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: '8px',
        fontFamily: 'monospace',
        fontSize: '12px',
      }}
    >
      {events.map((ev) => (
        <EventItem key={ev.id} ev={ev} />
      ))}
    </div>
  );
};

export const LogReplayModal = ({ runId, events, loading, onClose }: ModalProps) => (
  <div
    role="dialog"
    aria-modal="true"
    style={{
      position: 'fixed',
      inset: 0,
      backgroundColor: 'rgba(0, 0, 0, 0.5)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      zIndex: 1000,
      padding: '16px',
    }}
  >
    <div
      style={{
        backgroundColor: 'var(--sf)',
        border: '1px solid var(--ln)',
        borderRadius: '10px',
        width: '100%',
        maxWidth: '720px',
        maxHeight: '80vh',
        display: 'flex',
        flexDirection: 'column',
        boxShadow: '0 8px 24px rgba(0, 0, 0, 0.12)',
      }}
    >
      <ModalHeader runId={runId} onClose={onClose} />
      <div style={{ padding: '16px', flex: 1, overflowY: 'auto' }}>
        <EventList events={events} loading={loading} />
      </div>
    </div>
  </div>
);

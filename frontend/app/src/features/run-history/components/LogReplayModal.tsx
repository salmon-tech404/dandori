import { t } from '../../../core/i18n';
import { Modal } from '../../../ui/components/Modal';
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

export const LogReplayModal = ({ runId, events, loading, onClose }: ModalProps) => {
  const title = (
    <div>
      <div style={{ fontSize: '16px', fontWeight: 600 }}>{t('run_history_replay_title')}</div>
      <div style={{ fontSize: '12px', color: 'var(--mu)', fontFamily: 'monospace' }}>{runId}</div>
    </div>
  );

  return (
    <Modal open onClose={onClose} title={title} size="lg">
      <div style={{ maxHeight: '60vh', overflowY: 'auto' }}>
        <EventList events={events} loading={loading} />
      </div>
    </Modal>
  );
};

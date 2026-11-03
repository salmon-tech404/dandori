import React from 'react';
import { t } from '../../core/i18n';
import { Button } from '../../ui/components/Button';
import { MorningReviewChainBanner } from './MorningReviewChainBanner';
import type { OvernightQueueConfig, QueueItemWithTask } from './types';
import { useOvernightQueue } from './useOvernightQueue';

interface Props {
  workspaceId?: string;
}

const QueueItemRow: React.FC<{
  item: QueueItemWithTask;
  index: number;
  total: number;
  onMove: (idx: number, dir: 'up' | 'down') => void;
  onRemove: (id: string) => void;
}> = ({ item, index, total, onMove, onRemove }) => (
  <tr style={{ borderBottom: "1px solid var(--ln)" }}>
    <td style={{ padding: "8px 12px", fontFamily: "var(--font-mono)", fontSize: "var(--fs-12)", color: "var(--fg)" }}>{item.order_index}</td>
    <td style={{ padding: "8px 12px", fontWeight: 500, fontSize: "var(--fs-14)", color: "var(--fg)" }}>{item.task_title}</td>
    <td style={{ padding: "8px 12px", fontSize: "var(--fs-12)" }}>
      <span style={{ padding: "2px 8px", borderRadius: "var(--radius-4)", backgroundColor: "var(--sf2)", color: "var(--mu)" }}>
        {item.status}
      </span>
    </td>
    <td style={{ padding: "8px 12px", textAlign: "right", fontSize: "var(--fs-12)" }}>
      <div style={{ display: "inline-flex", gap: 4 }}>
        <Button variant="ghost" size="sm" type="button" disabled={index === 0} onClick={() => onMove(index, 'up')}>
          {t('overnight_queue_move_up')}
        </Button>
        <Button variant="ghost" size="sm" type="button" disabled={index === total - 1} onClick={() => onMove(index, 'down')}>
          {t('overnight_queue_move_down')}
        </Button>
        <Button variant="danger" size="sm" type="button" onClick={() => onRemove(item.id)}>
          {t('overnight_queue_remove')}
        </Button>
      </div>
    </td>
  </tr>
);

const QueueHeader: React.FC<{
  config: OvernightQueueConfig | null;
  onPause: () => void;
  onResume: () => void;
  onStartNow: () => void;
}> = ({ config, onPause, onResume, onStartNow }) => {
  const isRunning = config?.status === 'running';
  const isPaused = config?.status === 'paused';

  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "var(--sp-16)" }}>
      <div>
        <h2 style={{ margin: 0, fontSize: "var(--fs-18)", fontWeight: 700, color: "var(--fg)" }}>{t('overnight_queue_title')}</h2>
        <p style={{ margin: "4px 0 0", fontSize: "var(--fs-12)", color: "var(--mu)" }}>
          {config ? t('overnight_queue_start_time', { time: config.start_time }) : ''}
          {config ? ` | ${t('overnight_queue_concurrency', { count: String(config.max_concurrency) })}` : ''}
        </p>
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
        {isPaused ? (
          <Button variant="primary" size="sm" type="button" onClick={onResume}>
            {t('overnight_queue_resume')}
          </Button>
        ) : (
          <Button variant="secondary" size="sm" type="button" onClick={onPause}>
            {t('overnight_queue_pause')}
          </Button>
        )}
        {!isRunning && (
          <Button variant="primary" size="sm" type="button" onClick={onStartNow}>
            {t('overnight_queue_start_now')}
          </Button>
        )}
      </div>
    </div>
  );
};

const QueueTable: React.FC<{
  items: QueueItemWithTask[];
  onMove: (idx: number, dir: 'up' | 'down') => void;
  onRemove: (id: string) => void;
}> = ({ items, onMove, onRemove }) => {
  if (items.length === 0) {
    return <p style={{ fontSize: "var(--fs-12)", color: "var(--mu)", padding: "var(--sp-16) 0", textAlign: "center" }}>{t('overnight_queue_empty')}</p>;
  }
  return (
    <table style={{ width: "100%", textAlign: "left", fontSize: "var(--fs-14)", borderCollapse: "collapse", border: "1px solid var(--ln)", borderRadius: "var(--radius-6)" }}>
      <thead style={{ backgroundColor: "var(--sf2)", fontSize: "var(--fs-12)", color: "var(--mu)" }}>
        <tr>
          <th style={{ padding: "8px 12px" }}>{t('overnight_queue_col_order')}</th>
          <th style={{ padding: "8px 12px" }}>{t('overnight_queue_col_title')}</th>
          <th style={{ padding: "8px 12px" }}>{t('overnight_queue_col_status')}</th>
          <th style={{ padding: "8px 12px", textAlign: "right" }}>{t('overnight_queue_col_actions')}</th>
        </tr>
      </thead>
      <tbody>
        {items.map((item, idx) => (
          <QueueItemRow
            key={item.id}
            item={item}
            index={idx}
            total={items.length}
            onMove={onMove}
            onRemove={onRemove}
          />
        ))}
      </tbody>
    </table>
  );
};

export const OvernightQueuePanel: React.FC<Props> = ({ workspaceId = 'default' }) => {
  const {
    config,
    items,
    summary,
    loading,
    handlePause,
    handleResume,
    handleStartNow,
    handleRemoveItem,
    handleMoveItem,
  } = useOvernightQueue(workspaceId);

  if (loading && !config) {
    return <div style={{ padding: "var(--sp-16)", fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t('overnight_queue_loading')}</div>;
  }

  return (
    <div style={{ padding: "var(--sp-16)", backgroundColor: "var(--sf)", borderRadius: "var(--radius-8)", border: "1px solid var(--ln)" }}>
      <MorningReviewChainBanner summary={summary} />
      <QueueHeader
        config={config}
        onPause={handlePause}
        onResume={handleResume}
        onStartNow={handleStartNow}
      />
      <QueueTable items={items} onMove={handleMoveItem} onRemove={handleRemoveItem} />
    </div>
  );
};

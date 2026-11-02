import React from 'react';
import { t } from '../../core/i18n';
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
  <tr className="border-b hover:bg-slate-50/50 dark:hover:bg-slate-800/50">
    <td className="py-2 px-3 font-mono text-xs">{item.order_index}</td>
    <td className="py-2 px-3 font-medium text-sm">{item.task_title}</td>
    <td className="py-2 px-3 text-xs">
      <span className="px-2 py-0.5 rounded bg-slate-100 dark:bg-slate-700">
        {item.status}
      </span>
    </td>
    <td className="py-2 px-3 text-right text-xs space-x-1">
      <button
        type="button"
        disabled={index === 0}
        onClick={() => onMove(index, 'up')}
        className="px-1.5 py-0.5 border rounded disabled:opacity-30"
      >
        {t('overnight_queue_move_up')}
      </button>
      <button
        type="button"
        disabled={index === total - 1}
        onClick={() => onMove(index, 'down')}
        className="px-1.5 py-0.5 border rounded disabled:opacity-30"
      >
        {t('overnight_queue_move_down')}
      </button>
      <button
        type="button"
        onClick={() => onRemove(item.id)}
        className="px-1.5 py-0.5 text-red-500 hover:text-red-700"
      >
        {t('overnight_queue_remove')}
      </button>
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
    <div className="flex items-center justify-between mb-4">
      <div>
        <h2 className="text-lg font-bold">{t('overnight_queue_title')}</h2>
        <p className="text-xs text-slate-500">
          {config ? t('overnight_queue_start_time', { time: config.start_time }) : ''}
          {config ? ` | ${t('overnight_queue_concurrency', { count: String(config.max_concurrency) })}` : ''}
        </p>
      </div>
      <div className="flex items-center gap-2">
        {isPaused ? (
          <button type="button" onClick={onResume} className="px-3 py-1 bg-green-600 text-white rounded text-xs">
            {t('overnight_queue_resume')}
          </button>
        ) : (
          <button type="button" onClick={onPause} className="px-3 py-1 bg-amber-600 text-white rounded text-xs">
            {t('overnight_queue_pause')}
          </button>
        )}
        {!isRunning && (
          <button type="button" onClick={onStartNow} className="px-3 py-1 bg-blue-600 text-white rounded text-xs">
            {t('overnight_queue_start_now')}
          </button>
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
    return <p className="text-xs text-slate-500 py-4 text-center">{t('overnight_queue_empty')}</p>;
  }
  return (
    <table className="w-full text-left text-sm border rounded">
      <thead className="bg-slate-50 dark:bg-slate-800 text-xs">
        <tr>
          <th className="py-2 px-3">{t('overnight_queue_col_order')}</th>
          <th className="py-2 px-3">{t('overnight_queue_col_title')}</th>
          <th className="py-2 px-3">{t('overnight_queue_col_status')}</th>
          <th className="py-2 px-3 text-right">{t('overnight_queue_col_actions')}</th>
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
    return <div className="p-4 text-xs">{t('overnight_queue_loading')}</div>;
  }

  return (
    <div className="p-4 bg-white dark:bg-slate-900 rounded-lg shadow-sm">
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

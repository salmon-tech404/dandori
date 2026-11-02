import React from 'react';
import { t } from '../../core/i18n';
import type { MorningSummaryDto, ReviewChainItem } from './types';

const ReviewItemRow: React.FC<{ item: ReviewChainItem }> = ({ item }) => (
  <div className="flex items-center justify-between text-xs py-1 border-b last:border-b-0">
    <span className="font-medium">{item.title}</span>
    <div className="flex items-center gap-2">
      <span className="px-1.5 py-0.5 rounded bg-slate-100 dark:bg-slate-700 text-slate-700 dark:text-slate-300">
        {item.status}
      </span>
      {item.branch_name ? (
        <span className="font-mono text-blue-600 dark:text-blue-400">
          {t('overnight_queue_review_branch', { branch: item.branch_name })}
        </span>
      ) : null}
    </div>
  </div>
);

export const MorningReviewChainBanner: React.FC<{ summary: MorningSummaryDto | null }> = ({
  summary,
}) => {
  if (!summary || summary.review_items.length === 0) {
    return null;
  }

  const stats = t('overnight_queue_summary_stats', {
    completed: String(summary.completed_count),
    failed: String(summary.failed_count),
    needs_human: String(summary.needs_human_count),
  });

  return (
    <div className="p-3 mb-4 rounded-lg border border-amber-200 bg-amber-50 dark:bg-amber-950/30 dark:border-amber-800">
      <h3 className="font-semibold text-sm text-amber-900 dark:text-amber-200">
        {t('overnight_queue_morning_summary')}
      </h3>
      <p className="text-xs text-amber-700 dark:text-amber-300 mb-2">{stats}</p>
      <div className="divide-y divide-amber-200 dark:divide-amber-800/40">
        {summary.review_items.map((item) => (
          <ReviewItemRow key={item.task_id} item={item} />
        ))}
      </div>
    </div>
  );
};

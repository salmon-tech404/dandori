import React from 'react';
import { t } from '../../core/i18n';
import type { MorningSummaryDto, ReviewChainItem } from './types';

const ReviewItemRow: React.FC<{ item: ReviewChainItem }> = ({ item }) => (
  <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", fontSize: "var(--fs-12)", padding: "4px 0", borderBottom: "1px solid var(--ln)" }}>
    <span style={{ fontWeight: 500, color: "var(--fg)" }}>{item.title}</span>
    <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
      <span style={{ padding: "2px 6px", borderRadius: "var(--radius-4)", backgroundColor: "var(--sf2)", color: "var(--mu)" }}>
        {item.status}
      </span>
      {item.branch_name ? (
        <span style={{ fontFamily: "var(--font-mono)", color: "var(--bl)" }}>
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
    <div style={{ padding: "var(--sp-12)", marginBottom: "var(--sp-16)", borderRadius: "var(--radius-8)", border: "1px solid var(--or)", backgroundColor: "var(--orb)" }}>
      <h3 style={{ margin: 0, fontWeight: 600, fontSize: "var(--fs-14)", color: "var(--or)" }}>
        {t('overnight_queue_morning_summary')}
      </h3>
      <p style={{ margin: "4px 0 var(--sp-8)", fontSize: "var(--fs-12)", color: "var(--or)" }}>{stats}</p>
      <div style={{ display: "flex", flexDirection: "column" }}>
        {summary.review_items.map((item) => (
          <ReviewItemRow key={item.task_id} item={item} />
        ))}
      </div>
    </div>
  );
};

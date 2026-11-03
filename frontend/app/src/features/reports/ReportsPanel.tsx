import React, { useState } from 'react';
import { t } from '../../core/i18n';
import { Button } from '../../ui/components/Button';
import { ReportExportModal } from './ReportExportModal';
import { VoiceReportControl } from '../voice-report';
import type { ReportData } from './types';
import { useReports } from './useReports';

const WEEK_COLUMNS = [
  { day: 'T2', count: 2, height: '45%' },
  { day: 'T3', count: 3, height: '65%' },
  { day: 'T4', count: 1, height: '25%' },
  { day: 'T5', count: 4, height: '85%' },
  { day: 'T6', count: 4, height: '85%' },
  { day: 'T7', count: 0, height: '4px' },
  { day: 'CN', count: 0, height: '4px' },
];

const ReportWeeklyChart: React.FC = () => (
  <div style={{ display: 'flex', alignItems: 'flex-end', gap: '12px', height: '100px', padding: '8px 0', marginBottom: 'var(--sp-16)', borderBottom: '1px solid var(--ln)' }}>
    {WEEK_COLUMNS.map((d) => (
      <div key={d.day} style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'flex-end', gap: 4, height: '100%', fontSize: '11px', color: 'var(--mu)' }}>
        <span style={{ fontWeight: 600, color: 'var(--fg)' }}>{d.count}</span>
        <i style={{ display: 'block', width: '100%', maxWidth: '32px', height: d.height, borderRadius: '4px 4px 0 0', backgroundColor: 'var(--ac)' }} />
        <span>{d.day}</span>
      </div>
    ))}
  </div>
);

const ReportCountsCards: React.FC<{ report: ReportData }> = ({ report }) => (
  <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))', gap: 'var(--sp-12)', marginBottom: 'var(--sp-16)' }}>
    <div style={{ padding: 'var(--sp-12)', backgroundColor: 'var(--grb)', borderRadius: 'var(--radius-6)', border: '1px solid var(--gr)' }}>
      <div style={{ fontSize: 'var(--fs-12)', color: 'var(--gr)' }}>{t('reports_stat_completed')}</div>
      <div style={{ fontSize: 'var(--fs-18)', fontWeight: 700, color: 'var(--gr)' }}>{report.counts.completed}</div>
    </div>
    <div style={{ padding: 'var(--sp-12)', backgroundColor: 'var(--rdb)', borderRadius: 'var(--radius-6)', border: '1px solid var(--rd)' }}>
      <div style={{ fontSize: 'var(--fs-12)', color: 'var(--rd)' }}>{t('reports_stat_failed')}</div>
      <div style={{ fontSize: 'var(--fs-18)', fontWeight: 700, color: 'var(--rd)' }}>{report.counts.failed}</div>
    </div>
    <div style={{ padding: 'var(--sp-12)', backgroundColor: 'var(--orb)', borderRadius: 'var(--radius-6)', border: '1px solid var(--or)' }}>
      <div style={{ fontSize: 'var(--fs-12)', color: 'var(--or)' }}>{t('reports_stat_needs_human')}</div>
      <div style={{ fontSize: 'var(--fs-18)', fontWeight: 700, color: 'var(--or)' }}>{report.counts.needs_human}</div>
    </div>
    <div style={{ padding: 'var(--sp-12)', backgroundColor: 'var(--blb)', borderRadius: 'var(--radius-6)', border: '1px solid var(--bl)' }}>
      <div style={{ fontSize: 'var(--fs-12)', color: 'var(--bl)' }}>{t('reports_stat_running')}</div>
      <div style={{ fontSize: 'var(--fs-18)', fontWeight: 700, color: 'var(--bl)' }}>{report.counts.running}</div>
    </div>
  </div>
);

const ReportMetaSection: React.FC<{ report: ReportData }> = ({ report }) => {
  const costStr = report.cost.has_data && report.cost.total_cost !== null
    ? `${report.cost.total_cost.toFixed(4)} ${report.cost.currency ?? 'USD'}`
    : (report.cost.missing_reason ?? '—');

  return (
    <div style={{ padding: 'var(--sp-12)', backgroundColor: 'var(--sf2)', borderRadius: 'var(--radius-6)', fontSize: 'var(--fs-12)', display: 'flex', flexDirection: 'column', gap: 4, marginBottom: 'var(--sp-16)' }}>
      <div>{t('reports_duration_total', { time: String(report.duration.total_duration_secs) })}</div>
      <div>{t('reports_duration_median', { time: String(report.duration.median_duration_secs) })}</div>
      <div>{t('reports_cost', { amount: costStr })}</div>
      {report.switches.switch_count !== null && (
        <div>{t('reports_switches', { count: String(report.switches.switch_count) })}</div>
      )}
    </div>
  );
};

const ReportHeader: React.FC<{
  periodType: 'daily' | 'weekly';
  onPeriodChange: (p: 'daily' | 'weekly') => void;
  onExport: () => void;
}> = ({ periodType, onPeriodChange, onExport }) => (
  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 'var(--sp-16)' }}>
    <h2 style={{ margin: 0, fontSize: 'var(--fs-18)', fontWeight: 700, color: 'var(--fg)' }}>{t('reports_title')}</h2>
    <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--sp-8)' }}>
      <div style={{ display: 'inline-flex', borderRadius: 'var(--radius-4)', border: '1px solid var(--ln)', padding: 2, fontSize: 'var(--fs-12)' }}>
        <Button
          type="button"
          size="sm"
          variant={periodType === 'daily' ? 'primary' : 'ghost'}
          onClick={() => onPeriodChange('daily')}
        >
          {t('reports_daily')}
        </Button>
        <Button
          type="button"
          size="sm"
          variant={periodType === 'weekly' ? 'primary' : 'ghost'}
          onClick={() => onPeriodChange('weekly')}
        >
          {t('reports_weekly')}
        </Button>
      </div>
      <Button type="button" variant="secondary" size="sm" onClick={onExport}>
        {t('reports_export_btn')}
      </Button>
    </div>
  </div>
);

export const ReportsPanel: React.FC<{ workspaceId?: string; defaultPeriod?: 'daily' | 'weekly' }> = ({
  workspaceId = 'default',
  defaultPeriod = 'daily',
}) => {
  const { periodType, setPeriodType, report, markdown, loading } = useReports(workspaceId, defaultPeriod);
  const [modalOpen, setModalOpen] = useState(false);

  if (loading && !report) {
    return <div style={{ padding: 'var(--sp-16)', fontSize: 'var(--fs-12)', color: 'var(--mu)' }}>{t('reports_loading')}</div>;
  }

  return (
    <div style={{ padding: 'var(--sp-16)', backgroundColor: 'var(--sf)', borderRadius: 'var(--radius-8)', border: '1px solid var(--ln)' }}>
      <ReportHeader periodType={periodType} onPeriodChange={setPeriodType} onExport={() => setModalOpen(true)} />
      {report ? (
        <>
          <div style={{ marginBottom: 'var(--sp-16)' }}>
            <VoiceReportControl report={report} />
          </div>
          {periodType === 'weekly' && <ReportWeeklyChart />}
          <ReportCountsCards report={report} />
          <ReportMetaSection report={report} />
        </>
      ) : (
        <p style={{ fontSize: 'var(--fs-12)', color: 'var(--mu)', padding: 'var(--sp-16) 0', textAlign: 'center' }}>{t('reports_no_data')}</p>
      )}
      <ReportExportModal isOpen={modalOpen} markdown={markdown} onClose={() => setModalOpen(false)} />
    </div>
  );
};

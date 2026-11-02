import React, { useState } from 'react';
import { t } from '../../core/i18n';
import { ReportExportModal } from './ReportExportModal';
import { VoiceReportControl } from '../voice-report';
import type { ReportData } from './types';
import { useReports } from './useReports';

const ReportCountsCards: React.FC<{ report: ReportData }> = ({ report }) => (
  <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-4">
    <div className="p-3 bg-green-50 dark:bg-green-950/30 rounded border border-green-200">
      <div className="text-xs text-green-700">{t('reports_stat_completed')}</div>
      <div className="text-xl font-bold text-green-800">{report.counts.completed}</div>
    </div>
    <div className="p-3 bg-red-50 dark:bg-red-950/30 rounded border border-red-200">
      <div className="text-xs text-red-700">{t('reports_stat_failed')}</div>
      <div className="text-xl font-bold text-red-800">{report.counts.failed}</div>
    </div>
    <div className="p-3 bg-amber-50 dark:bg-amber-950/30 rounded border border-amber-200">
      <div className="text-xs text-amber-700">{t('reports_stat_needs_human')}</div>
      <div className="text-xl font-bold text-amber-800">{report.counts.needs_human}</div>
    </div>
    <div className="p-3 bg-blue-50 dark:bg-blue-950/30 rounded border border-blue-200">
      <div className="text-xs text-blue-700">{t('reports_stat_running')}</div>
      <div className="text-xl font-bold text-blue-800">{report.counts.running}</div>
    </div>
  </div>
);

const ReportMetaSection: React.FC<{ report: ReportData }> = ({ report }) => {
  const costStr = report.cost.has_data && report.cost.total_cost !== null
    ? `${report.cost.total_cost.toFixed(4)} ${report.cost.currency ?? 'USD'}`
    : (report.cost.missing_reason ?? '—');

  return (
    <div className="p-3 bg-slate-50 dark:bg-slate-800 rounded text-xs space-y-1 mb-4">
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
  <div className="flex items-center justify-between mb-4">
    <h2 className="text-lg font-bold">{t('reports_title')}</h2>
    <div className="flex items-center gap-2">
      <div className="inline-flex rounded border p-0.5 text-xs">
        <button
          type="button"
          onClick={() => onPeriodChange('daily')}
          className={`px-2.5 py-1 rounded ${periodType === 'daily' ? 'bg-blue-600 text-white' : ''}`}
        >
          {t('reports_daily')}
        </button>
        <button
          type="button"
          onClick={() => onPeriodChange('weekly')}
          className={`px-2.5 py-1 rounded ${periodType === 'weekly' ? 'bg-blue-600 text-white' : ''}`}
        >
          {t('reports_weekly')}
        </button>
      </div>
      <button type="button" onClick={onExport} className="px-3 py-1 bg-slate-700 text-white rounded text-xs">
        {t('reports_export_btn')}
      </button>
    </div>
  </div>
);

export const ReportsPanel: React.FC<{ workspaceId?: string }> = ({ workspaceId = 'default' }) => {
  const { periodType, setPeriodType, report, markdown, loading } = useReports(workspaceId);
  const [modalOpen, setModalOpen] = useState(false);

  if (loading && !report) {
    return <div className="p-4 text-xs">{t('reports_loading')}</div>;
  }

  return (
    <div className="p-4 bg-white dark:bg-slate-900 rounded-lg shadow-sm">
      <ReportHeader periodType={periodType} onPeriodChange={setPeriodType} onExport={() => setModalOpen(true)} />
      {report ? (
        <>
          <div className="mb-4">
            <VoiceReportControl report={report} />
          </div>
          <ReportCountsCards report={report} />
          <ReportMetaSection report={report} />
        </>
      ) : (
        <p className="text-xs text-slate-500 py-4 text-center">{t('reports_no_data')}</p>
      )}
      <ReportExportModal isOpen={modalOpen} markdown={markdown} onClose={() => setModalOpen(false)} />
    </div>
  );
};

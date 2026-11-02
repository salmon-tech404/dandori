import { t } from '../../../core/i18n';
import type { RunSummary } from '../types';
import { RunTableRow } from './RunTableRow';

interface TableProps {
  runs: RunSummary[];
  onOpenReplay: (id: string) => void;
  onOpenNote: (id: string) => void;
  onExport: (id: string, format: 'markdown' | 'json') => void;
}

export const RunHistoryTable = ({ runs, onOpenReplay, onOpenNote, onExport }: TableProps) => (
  <div style={{ overflowX: 'auto', border: '1px solid var(--ln)', borderRadius: '8px' }}>
    <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '13px' }}>
      <thead>
        <tr style={{ backgroundColor: 'var(--sf2)', borderBottom: '1px solid var(--ln)' }}>
          <th style={{ padding: '10px 12px', textAlign: 'left' }}>{t('run_history_col_id')}</th>
          <th style={{ padding: '10px 12px', textAlign: 'left' }}>{t('run_history_col_agent')}</th>
          <th style={{ padding: '10px 12px', textAlign: 'left' }}>{t('run_history_col_status')}</th>
          <th style={{ padding: '10px 12px', textAlign: 'left' }}>{t('run_history_col_started')}</th>
          <th style={{ padding: '10px 12px', textAlign: 'left' }}>{t('run_history_col_duration')}</th>
          <th style={{ padding: '10px 12px', textAlign: 'left' }}>{t('run_history_col_exit_code')}</th>
          <th style={{ padding: '10px 12px', textAlign: 'right' }}>{t('tech_profile_col_actions')}</th>
        </tr>
      </thead>
      <tbody>
        {runs.map((r) => (
          <RunTableRow
            key={r.id}
            run={r}
            onOpenReplay={onOpenReplay}
            onOpenNote={onOpenNote}
            onExport={onExport}
          />
        ))}
      </tbody>
    </table>
  </div>
);

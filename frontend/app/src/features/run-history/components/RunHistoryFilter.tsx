import { t } from '../../../core/i18n';

interface FilterProps {
  statusFilter: string;
  agentFilter: string;
  onStatusChange: (val: string) => void;
  onAgentChange: (val: string) => void;
}

const selectStyle: React.CSSProperties = {
  padding: '6px 10px',
  minHeight: '44px',
  borderRadius: '8px',
  border: '1px solid var(--ln)',
  backgroundColor: 'var(--sf)',
  color: 'var(--fg)',
};

export const RunHistoryFilter = ({
  statusFilter,
  agentFilter,
  onStatusChange,
  onAgentChange,
}: FilterProps) => (
  <div style={{ display: 'flex', gap: '8px' }}>
    <select value={statusFilter} onChange={(e) => onStatusChange(e.target.value)} style={selectStyle}>
      <option value="">{t('run_history_filter_all_status')}</option>
      <option value="success">{t('status_done')}</option>
      <option value="failed">{t('status_failed')}</option>
      <option value="running">{t('status_running')}</option>
      <option value="cancelled">{t('status_cancelled')}</option>
    </select>
    <select value={agentFilter} onChange={(e) => onAgentChange(e.target.value)} style={selectStyle}>
      <option value="">{t('run_history_filter_all_agents')}</option>
      <option value="claude">Claude</option>
      <option value="gemini">Gemini</option>
      <option value="codex">Codex</option>
    </select>
  </div>
);

import { t } from '../../../core/i18n';
import { Select } from '../../../ui/components/Select';

interface FilterProps {
  statusFilter: string;
  agentFilter: string;
  onStatusChange: (val: string) => void;
  onAgentChange: (val: string) => void;
}

const statusOptions = [
  { value: "", label: t('run_history_filter_all_status') },
  { value: "success", label: t('status_done') },
  { value: "failed", label: t('status_failed') },
  { value: "running", label: t('status_running') },
  { value: "cancelled", label: t('status_cancelled') },
];

export const RunHistoryFilter = ({
  statusFilter,
  agentFilter,
  onStatusChange,
  onAgentChange,
}: FilterProps) => {
  const agentOptions = [
    { value: "", label: t('run_history_filter_all_agents') },
    { value: "claude", label: t("provider_claude") },
    { value: "gemini", label: t("provider_gemini") },
    { value: "codex", label: t("provider_codex") },
  ];

  return (
    <div style={{ display: 'flex', gap: '8px' }}>
      <Select
        value={statusFilter}
        onChange={(e) => onStatusChange(e.target.value)}
        options={statusOptions}
      />
      <Select
        value={agentFilter}
        onChange={(e) => onAgentChange(e.target.value)}
        options={agentOptions}
      />
    </div>
  );
};

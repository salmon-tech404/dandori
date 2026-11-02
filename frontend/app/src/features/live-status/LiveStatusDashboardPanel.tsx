import React from 'react';
import { t } from '../../core/i18n';
import { ActiveAgentsTable } from './ActiveAgentsTable';
import { ConnectionIndicator } from './ConnectionIndicator';
import { LiveLogViewer } from './LiveLogViewer';
import { useLiveStatus } from './useLiveStatus';

interface Props {
  workspaceId?: string;
}

export const LiveStatusDashboardPanel: React.FC<Props> = ({ workspaceId }) => {
  const {
    connectionState,
    activeAgents,
    logsByRunId,
    selectedRunId,
    setSelectedRunId,
  } = useLiveStatus(workspaceId);

  const selectedLogs = selectedRunId ? logsByRunId[selectedRunId] || [] : [];

  return (
    <div
      data-testid="live-status-dashboard-panel"
      className="space-y-4 rounded-xl border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-950"
    >
      <div className="flex items-center justify-between border-b border-neutral-100 pb-3 dark:border-neutral-800/60">
        <div>
          <h3 className="font-semibold text-neutral-900 dark:text-neutral-100">
            {t('live_status_title')}
          </h3>
          <p className="text-xs text-neutral-500">{t('live_status_desc')}</p>
        </div>
        <ConnectionIndicator state={connectionState} />
      </div>

      <ActiveAgentsTable
        agents={activeAgents}
        selectedRunId={selectedRunId}
        onSelectRun={setSelectedRunId}
      />

      <LiveLogViewer logs={selectedLogs} selectedRunId={selectedRunId} />
    </div>
  );
};

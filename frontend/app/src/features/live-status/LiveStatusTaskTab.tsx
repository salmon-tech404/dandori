import React from 'react';
import { t } from '../../core/i18n';
import { ConnectionIndicator } from './ConnectionIndicator';
import { LiveLogViewer } from './LiveLogViewer';
import { useLiveStatus } from './useLiveStatus';

interface Props {
  workspaceId?: string;
  taskId?: string;
}

export const LiveStatusTaskTab: React.FC<Props> = ({
  workspaceId,
  taskId,
}) => {
  const { connectionState, activeAgents, logsByRunId } =
    useLiveStatus(workspaceId);

  const taskAgent = activeAgents.find((a) => a.taskId === taskId);
  const runId = taskAgent ? taskAgent.runId : null;
  const logs = runId ? logsByRunId[runId] || [] : [];

  return (
    <div data-testid="live-status-task-tab" className="space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-xs font-medium text-neutral-500">
          {t('live_status_tab_label')}
        </span>
        <ConnectionIndicator state={connectionState} />
      </div>

      <LiveLogViewer logs={logs} selectedRunId={runId || 'task-run'} />
    </div>
  );
};

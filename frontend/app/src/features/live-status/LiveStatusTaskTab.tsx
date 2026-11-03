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
    <div data-testid="live-status-task-tab" style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)" }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <span style={{ fontSize: "var(--fs-12)", fontWeight: 500, color: "var(--mu)" }}>
          {t('live_status_tab_label')}
        </span>
        <ConnectionIndicator state={connectionState} />
      </div>

      <LiveLogViewer logs={logs} selectedRunId={runId || 'task-run'} />
    </div>
  );
};

import React from 'react';
import { t } from '../../core/i18n';
import { ActiveAgentsTable } from './ActiveAgentsTable';
import { ConnectionIndicator } from './ConnectionIndicator';
import { LiveLogViewer } from './LiveLogViewer';
import { useLiveStatus } from './useLiveStatus';
import type { ActiveAgent, ConnectionState } from './types';

interface Props {
  workspaceId?: string;
}

function getAgentBadge(agentStr: string) {
  const p = agentStr.toLowerCase();
  if (p.includes('gemini')) return { letter: 'G', bg: 'rgba(37, 99, 235, 0.15)', fg: 'var(--bl)' };
  if (p.includes('codex') || p.includes('openai')) return { letter: 'K', bg: 'rgba(13, 148, 136, 0.15)', fg: 'var(--tl)' };
  if (p.includes('fake')) return { letter: 'F', bg: 'rgba(100, 116, 139, 0.15)', fg: 'var(--mu)' };
  return { letter: 'C', bg: 'rgba(217, 119, 6, 0.15)', fg: 'var(--or)' };
}

function AgentLiveCard({ agent }: { agent: ActiveAgent }) {
  const badge = getAgentBadge(agent.agent);
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 6, padding: '10px 12px', borderRadius: 'var(--radius-6)', backgroundColor: 'var(--sf2)' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <span style={{ display: 'inline-flex', alignItems: 'center', justifyContent: 'center', width: 22, height: 22, borderRadius: '50%', backgroundColor: badge.bg, color: badge.fg, fontWeight: 700, fontSize: 11 }}>
          {badge.letter}
        </span>
        <b style={{ fontSize: 'var(--fs-13)', color: 'var(--fg)' }}>{agent.taskTitle}</b>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: '11px', color: 'var(--mu)' }}>
        <span style={{ fontFamily: 'var(--font-mono)' }}>{agent.agent} ({agent.account})</span>
        <div style={{ flex: 1, height: 4, borderRadius: 'var(--radius-pill)', backgroundColor: 'var(--ln)', overflow: 'hidden' }}>
          <div style={{ height: '100%', borderRadius: 'var(--radius-pill)', backgroundColor: 'var(--bl)', width: '65%' }} />
        </div>
        <span>65%</span>
      </div>
    </div>
  );
}

function LiveStatusPanelHeader({ state }: { state: ConnectionState }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', borderBottom: '1px solid var(--ln)', paddingBottom: 'var(--sp-12)' }}>
      <div>
        <h3 style={{ margin: 0, fontWeight: 600, color: 'var(--fg)' }}>
          {t('live_status_title')}
        </h3>
        <p style={{ margin: '4px 0 0', fontSize: 'var(--fs-12)', color: 'var(--mu)' }}>{t('live_status_desc')}</p>
      </div>
      <ConnectionIndicator state={state} />
    </div>
  );
}

function ActiveAgentCards({ agents }: { agents: ActiveAgent[] }) {
  if (agents.length === 0) return null;
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
      {agents.map((a) => (
        <AgentLiveCard key={a.runId} agent={a} />
      ))}
    </div>
  );
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
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 'var(--sp-16)',
        borderRadius: 'var(--radius-10)',
        border: '1px solid var(--ln)',
        backgroundColor: 'var(--sf)',
        padding: 'var(--sp-16)',
      }}
    >
      <LiveStatusPanelHeader state={connectionState} />
      <ActiveAgentCards agents={activeAgents} />
      <ActiveAgentsTable
        agents={activeAgents}
        selectedRunId={selectedRunId}
        onSelectRun={setSelectedRunId}
      />
      <LiveLogViewer logs={selectedLogs} selectedRunId={selectedRunId} />
    </div>
  );
};

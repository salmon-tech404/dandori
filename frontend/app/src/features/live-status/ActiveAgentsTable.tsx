import { Bot, Clock } from 'lucide-react';
import React from 'react';
import { t } from '../../core/i18n';
import { ActiveAgent } from './types';

interface Props {
  agents: ActiveAgent[];
  selectedRunId: string | null;
  onSelectRun: (runId: string) => void;
}

function formatElapsed(startedAt: string): string {
  const start = new Date(startedAt).getTime();
  if (isNaN(start)) return '0s';
  const diffSec = Math.max(0, Math.floor((Date.now() - start) / 1000));
  const m = Math.floor(diffSec / 60);
  const s = diffSec % 60;
  return m > 0 ? `${m}m ${s}s` : `${s}s`;
}

const TableRow: React.FC<{
  agent: ActiveAgent;
  isSelected: boolean;
  onSelect: () => void;
}> = ({ agent, isSelected, onSelect }) => (
  <tr
    onClick={onSelect}
    style={{
      cursor: "pointer",
      borderBottom: "1px solid var(--ln)",
      backgroundColor: isSelected ? "var(--blb)" : "transparent",
    }}
  >
    <td style={{ padding: "10px 12px" }}>
      <div style={{ display: "flex", alignItems: "center", gap: 6, fontWeight: 500, color: "var(--fg)" }}>
        <Bot size={14} style={{ color: "var(--bl)" }} />
        <span>{agent.agent}</span>
        <span style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>({agent.account})</span>
      </div>
    </td>
    <td style={{ padding: "10px 12px", color: "var(--fg)" }}>{agent.taskTitle}</td>
    <td style={{ padding: "10px 12px", color: "var(--mu)" }}>{agent.assignedBy}</td>
    <td style={{ padding: "10px 12px", fontFamily: "var(--font-mono)", fontSize: "var(--fs-12)", color: "var(--fg)" }}>
      <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
        <Clock size={12} style={{ color: "var(--mu)" }} />
        {formatElapsed(agent.startedAt)}
      </div>
    </td>
    <td style={{ padding: "10px 12px", fontSize: "var(--fs-12)", color: "var(--mu)", fontStyle: "italic" }}>
      {t('live_status_eta_placeholder')}
    </td>
    <td style={{ padding: "10px 12px" }}>
      <span style={{ borderRadius: "var(--radius-4)", backgroundColor: "var(--grb)", padding: "2px 8px", fontSize: "var(--fs-12)", fontWeight: 500, color: "var(--gr)" }}>
        {agent.status}
      </span>
    </td>
  </tr>
);

const AgentsGrid: React.FC<Props> = ({ agents, selectedRunId, onSelectRun }) => (
  <div style={{ overflowX: "auto", borderRadius: "var(--radius-8)", border: "1px solid var(--ln)", backgroundColor: "var(--sf)", fontSize: "var(--fs-14)" }}>
    <table style={{ width: "100%", textAlign: "left", borderCollapse: "collapse" }}>
      <thead style={{ borderBottom: "1px solid var(--ln)", backgroundColor: "var(--sf2)", fontSize: "var(--fs-12)", fontWeight: 600, color: "var(--mu)" }}>
        <tr>
          <th style={{ padding: "8px 12px" }}>{t('live_status_col_agent')}</th>
          <th style={{ padding: "8px 12px" }}>{t('live_status_col_task')}</th>
          <th style={{ padding: "8px 12px" }}>{t('live_status_col_assigned_by')}</th>
          <th style={{ padding: "8px 12px" }}>{t('live_status_col_elapsed')}</th>
          <th style={{ padding: "8px 12px" }}>{t('live_status_col_eta')}</th>
          <th style={{ padding: "8px 12px" }}>{t('live_status_col_status')}</th>
        </tr>
      </thead>
      <tbody>
        {agents.map((a) => (
          <TableRow
            key={a.runId}
            agent={a}
            isSelected={selectedRunId === a.runId}
            onSelect={() => onSelectRun(a.runId)}
          />
        ))}
      </tbody>
    </table>
  </div>
);

export const ActiveAgentsTable: React.FC<Props> = (props) => {
  if (props.agents.length === 0) {
    return (
      <div
        data-testid="active-agents-empty"
        style={{
          borderRadius: "var(--radius-8)",
          border: "1px dashed var(--ln)",
          padding: "var(--sp-24)",
          textAlign: "center",
          fontSize: "var(--fs-14)",
          color: "var(--mu)",
        }}
      >
        {t('live_status_no_active_agents')}
      </div>
    );
  }

  return <AgentsGrid {...props} />;
};

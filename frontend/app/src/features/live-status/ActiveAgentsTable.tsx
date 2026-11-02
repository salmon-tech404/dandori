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
    className={`cursor-pointer border-b border-neutral-100 transition-colors hover:bg-neutral-50 dark:border-neutral-800/50 dark:hover:bg-neutral-800/40 ${
      isSelected ? 'bg-indigo-50/60 dark:bg-indigo-950/20' : ''
    }`}
  >
    <td className="px-3 py-2.5">
      <div className="flex items-center gap-1.5 font-medium text-neutral-800 dark:text-neutral-200">
        <Bot size={14} className="text-indigo-500" />
        <span>{agent.agent}</span>
        <span className="text-xs text-neutral-400">({agent.account})</span>
      </div>
    </td>
    <td className="px-3 py-2.5 text-neutral-600 dark:text-neutral-300">{agent.taskTitle}</td>
    <td className="px-3 py-2.5 text-neutral-500">{agent.assignedBy}</td>
    <td className="px-3 py-2.5 font-mono text-xs text-neutral-600 dark:text-neutral-300">
      <div className="flex items-center gap-1">
        <Clock size={12} className="text-neutral-400" />
        {formatElapsed(agent.startedAt)}
      </div>
    </td>
    <td className="px-3 py-2.5 text-xs text-neutral-400 italic">
      {t('live_status_eta_placeholder')}
    </td>
    <td className="px-3 py-2.5">
      <span className="rounded bg-emerald-50 px-2 py-0.5 text-xs font-medium text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-400">
        {agent.status}
      </span>
    </td>
  </tr>
);

export const ActiveAgentsTable: React.FC<Props> = ({ agents, selectedRunId, onSelectRun }) => {
  if (agents.length === 0) {
    return (
      <div
        data-testid="active-agents-empty"
        className="rounded-lg border border-dashed border-neutral-200 p-6 text-center text-sm text-neutral-400 dark:border-neutral-800"
      >
        {t('live_status_no_active_agents')}
      </div>
    );
  }

  return (
    <div className="overflow-x-auto rounded-lg border border-neutral-200 bg-white text-sm dark:border-neutral-800 dark:bg-neutral-900">
      <table className="w-full text-left">
        <thead className="border-b border-neutral-200 bg-neutral-50 text-xs font-semibold text-neutral-500 dark:border-neutral-800 dark:bg-neutral-800/50">
          <tr>
            <th className="px-3 py-2">{t('live_status_col_agent')}</th>
            <th className="px-3 py-2">{t('live_status_col_task')}</th>
            <th className="px-3 py-2">{t('live_status_col_assigned_by')}</th>
            <th className="px-3 py-2">{t('live_status_col_elapsed')}</th>
            <th className="px-3 py-2">{t('live_status_col_eta')}</th>
            <th className="px-3 py-2">{t('live_status_col_status')}</th>
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
};

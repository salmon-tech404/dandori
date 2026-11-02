import { ChevronDown, ChevronUp, Terminal } from 'lucide-react';
import React, { useState } from 'react';
import { t } from '../../core/i18n';
import { LogEntry, MAX_LOG_ENTRIES } from './types';

interface Props {
  logs: LogEntry[];
  selectedRunId: string | null;
}

const LogLine: React.FC<{ entry: LogEntry }> = ({ entry }) => (
  <div className="flex gap-2 font-mono text-xs leading-relaxed">
    <span className="select-none text-neutral-500">[{entry.seq}]</span>
    <span
      className={
        entry.stream === 'stderr'
          ? 'text-rose-400'
          : 'text-neutral-200 dark:text-neutral-300'
      }
    >
      {entry.chunk}
    </span>
  </div>
);

const LogHeader: React.FC<{
  count: number;
  isExpanded: boolean;
  onToggle: () => void;
}> = ({ count, isExpanded, onToggle }) => (
  <div className="flex items-center justify-between border-b border-neutral-800 px-3 py-2 text-xs">
    <div className="flex items-center gap-1.5 font-medium text-neutral-300">
      <Terminal size={13} className="text-emerald-400" />
      <span>{t('live_status_log_title')}</span>
    </div>
    <div className="flex items-center gap-3">
      <span className="text-neutral-400">
        {t('live_status_log_lines_count', { count, max: MAX_LOG_ENTRIES })}
      </span>
      <button
        type="button"
        onClick={onToggle}
        className="flex items-center gap-1 text-neutral-300 hover:text-white"
      >
        {isExpanded ? (
          <>
            <ChevronUp size={12} />
            <span>{t('live_status_log_collapse')}</span>
          </>
        ) : (
          <>
            <ChevronDown size={12} />
            <span>{t('live_status_log_expand')}</span>
          </>
        )}
      </button>
    </div>
  </div>
);

export const LiveLogViewer: React.FC<Props> = ({ logs, selectedRunId }) => {
  const [isExpanded, setIsExpanded] = useState(false);

  if (!selectedRunId) {
    return (
      <div
        data-testid="live-log-prompt"
        className="rounded-lg border border-neutral-200 bg-neutral-50 p-4 text-center text-xs text-neutral-400 dark:border-neutral-800 dark:bg-neutral-900/50"
      >
        {t('live_status_log_select_prompt')}
      </div>
    );
  }

  return (
    <div className="rounded-lg border border-neutral-200 bg-neutral-900 text-neutral-100 dark:border-neutral-800">
      <LogHeader
        count={logs.length}
        isExpanded={isExpanded}
        onToggle={() => setIsExpanded((prev) => !prev)}
      />
      <div
        data-testid="live-log-container"
        className={`overflow-y-auto p-3 ${isExpanded ? 'h-96' : 'h-48'}`}
      >
        {logs.length === 0 ? (
          <div className="text-xs text-neutral-500 italic">
            {t('live_status_log_empty')}
          </div>
        ) : (
          logs.map((entry) => <LogLine key={entry.id} entry={entry} />)
        )}
      </div>
    </div>
  );
};

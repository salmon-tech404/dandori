import { ChevronDown, ChevronUp, Terminal } from 'lucide-react';
import React, { useState } from 'react';
import { t } from '../../core/i18n';
import { Button } from '../../ui/components/Button';
import { LogEntry, MAX_LOG_ENTRIES } from './types';

interface Props {
  logs: LogEntry[];
  selectedRunId: string | null;
}

const LogLine: React.FC<{ entry: LogEntry }> = ({ entry }) => (
  <div style={{ display: "flex", gap: "var(--sp-8)", fontFamily: "var(--font-mono)", fontSize: "var(--fs-12)", lineHeight: 1.6 }}>
    <span style={{ userSelect: "none", color: "var(--mu)" }}>[{entry.seq}]</span>
    <span style={{ color: entry.stream === 'stderr' ? 'var(--rd)' : 'var(--fg)' }}>
      {entry.chunk}
    </span>
  </div>
);

const LogHeader: React.FC<{
  count: number;
  isExpanded: boolean;
  onToggle: () => void;
}> = ({ count, isExpanded, onToggle }) => (
  <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", borderBottom: "1px solid var(--ln)", padding: "8px 12px", fontSize: "var(--fs-12)" }}>
    <div style={{ display: "flex", alignItems: "center", gap: 6, fontWeight: 500, color: "var(--fg)" }}>
      <Terminal size={13} style={{ color: "var(--gr)" }} />
      <span>{t('live_status_log_title')}</span>
    </div>
    <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-12)" }}>
      <span style={{ color: "var(--mu)" }}>
        {t('live_status_log_lines_count', { count, max: MAX_LOG_ENTRIES })}
      </span>
      <Button
        variant="ghost"
        size="sm"
        type="button"
        onClick={onToggle}
        style={{ display: "flex", alignItems: "center", gap: 4 }}
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
      </Button>
    </div>
  </div>
);

const LogContainer: React.FC<{ logs: LogEntry[]; isExpanded: boolean }> = ({ logs, isExpanded }) => (
  <div
    data-testid="live-log-container"
    style={{
      overflowY: "auto",
      padding: "var(--sp-12)",
      height: isExpanded ? 384 : 192,
    }}
  >
    {logs.length === 0 ? (
      <div style={{ fontSize: "var(--fs-12)", color: "var(--mu)", fontStyle: "italic" }}>
        {t('live_status_log_empty')}
      </div>
    ) : (
      logs.map((entry) => <LogLine key={entry.id} entry={entry} />)
    )}
  </div>
);

export const LiveLogViewer: React.FC<Props> = ({ logs, selectedRunId }) => {
  const [isExpanded, setIsExpanded] = useState(false);

  if (!selectedRunId) {
    return (
      <div
        data-testid="live-log-prompt"
        style={{
          borderRadius: "var(--radius-8)",
          border: "1px solid var(--ln)",
          backgroundColor: "var(--sf)",
          padding: "var(--sp-16)",
          textAlign: "center",
          fontSize: "var(--fs-12)",
          color: "var(--mu)",
        }}
      >
        {t('live_status_log_select_prompt')}
      </div>
    );
  }

  return (
    <div style={{ borderRadius: "var(--radius-8)", border: "1px solid var(--ln)", backgroundColor: "var(--sf)", color: "var(--fg)" }}>
      <LogHeader
        count={logs.length}
        isExpanded={isExpanded}
        onToggle={() => setIsExpanded((prev) => !prev)}
      />
      <LogContainer logs={logs} isExpanded={isExpanded} />
    </div>
  );
};

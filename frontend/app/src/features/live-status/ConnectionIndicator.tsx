import { Radio } from 'lucide-react';
import React from 'react';
import { t } from '../../core/i18n';
import { ConnectionState } from './types';

interface Props {
  state: ConnectionState;
}

const STATE_CONFIG: Record<ConnectionState, { dot: string; textKey: string }> = {
  connected: { dot: 'bg-emerald-500', textKey: 'live_status_conn_connected' },
  connecting: { dot: 'bg-amber-400 animate-pulse', textKey: 'live_status_conn_connecting' },
  reconnecting: { dot: 'bg-amber-500 animate-pulse', textKey: 'live_status_conn_reconnecting' },
  disconnected: { dot: 'bg-neutral-400', textKey: 'live_status_conn_disconnected' },
  error: { dot: 'bg-rose-500', textKey: 'live_status_conn_error' },
};

export const ConnectionIndicator: React.FC<Props> = ({ state }) => {
  const config = STATE_CONFIG[state] || STATE_CONFIG.disconnected;

  return (
    <div
      data-testid="connection-indicator"
      className="inline-flex items-center gap-2 rounded-full border border-neutral-200 bg-neutral-50 px-2.5 py-1 text-xs text-neutral-600 dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-300"
    >
      <Radio size={12} className="text-neutral-400" />
      <span className={`inline-block h-2 w-2 rounded-full ${config.dot}`} />
      <span>{t(config.textKey)}</span>
    </div>
  );
};

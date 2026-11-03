import { Radio } from 'lucide-react';
import React from 'react';
import { t } from '../../core/i18n';
import { ConnectionState } from './types';

interface Props {
  state: ConnectionState;
}

const STATE_CONFIG: Record<ConnectionState, { dotColor: string; pulse?: boolean; textKey: string }> = {
  connected: { dotColor: 'var(--gr)', textKey: 'live_status_conn_connected' },
  connecting: { dotColor: 'var(--or)', pulse: true, textKey: 'live_status_conn_connecting' },
  reconnecting: { dotColor: 'var(--or)', pulse: true, textKey: 'live_status_conn_reconnecting' },
  disconnected: { dotColor: 'var(--mu)', textKey: 'live_status_conn_disconnected' },
  error: { dotColor: 'var(--rd)', textKey: 'live_status_conn_error' },
};

export const ConnectionIndicator: React.FC<Props> = ({ state }) => {
  const config = STATE_CONFIG[state] || STATE_CONFIG.disconnected;

  return (
    <div
      data-testid="connection-indicator"
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: "var(--sp-8)",
        borderRadius: "var(--radius-pill)",
        border: "1px solid var(--ln)",
        backgroundColor: "var(--sf)",
        padding: "4px 10px",
        fontSize: "var(--fs-12)",
        color: "var(--fg)",
      }}
    >
      <Radio size={12} style={{ color: "var(--mu)" }} />
      <span
        style={{
          display: "inline-block",
          width: 8,
          height: 8,
          borderRadius: "var(--radius-pill)",
          backgroundColor: config.dotColor,
          animation: config.pulse ? "dandori-pulse 1.5s infinite" : "none",
        }}
      />
      <span>{t(config.textKey)}</span>
    </div>
  );
};

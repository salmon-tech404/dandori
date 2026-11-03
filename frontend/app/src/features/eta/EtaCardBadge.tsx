import React from 'react';
import { Clock, AlertTriangle } from 'lucide-react';
import type { EtaResult, EtaConfidence } from './types';
import { t } from '../../core/i18n';

export interface EtaCardBadgeProps {
  eta?: EtaResult | null;
  className?: string;
}

function getConfidenceLabel(confidence?: EtaConfidence | null): string {
  if (confidence === 'high') return t('eta_confidence_high');
  if (confidence === 'medium') return t('eta_confidence_medium');
  return t('eta_confidence_low');
}

const badgeStyle: React.CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  gap: 4,
  padding: "2px 6px",
  borderRadius: "var(--radius-4)",
  fontSize: "var(--fs-12)",
  fontWeight: 500,
  backgroundColor: "var(--sf2)",
  color: "var(--fg)",
  border: "1px solid var(--ln)",
};

const ActiveBadge: React.FC<{ eta: EtaResult }> = ({ eta }) => {
  const confLabel = getConfidenceLabel(eta.confidence);
  return (
    <div
      data-testid="eta-card-badge"
      title={eta.confidence_downgraded ? t('eta_downgraded_notice') : undefined}
      style={badgeStyle}
    >
      <Clock style={{ width: 12, height: 12, color: "var(--bl)" }} />
      <span>{eta.eta_range_text}</span>
      <span style={{ fontSize: "10px", color: "var(--mu)" }}>({confLabel})</span>
      {eta.confidence_downgraded && (
        <AlertTriangle data-testid="eta-downgraded-icon" style={{ width: 12, height: 12, color: "var(--or)" }} />
      )}
    </div>
  );
};

export const EtaCardBadge: React.FC<EtaCardBadgeProps> = ({ eta }) => {
  if (!eta || !eta.has_estimate || !eta.eta_range_text) {
    return (
      <span
        data-testid="eta-badge-empty"
        style={{ display: "inline-flex", alignItems: "center", fontSize: "var(--fs-12)", color: "var(--mu)" }}
      >
        {t('eta_not_available')}
      </span>
    );
  }
  return <ActiveBadge eta={eta} />;
};

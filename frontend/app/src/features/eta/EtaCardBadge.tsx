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

export const EtaCardBadge: React.FC<EtaCardBadgeProps> = ({ eta, className = '' }) => {
  if (!eta || !eta.has_estimate || !eta.eta_range_text) {
    return (
      <span
        data-testid="eta-badge-empty"
        className={`inline-flex items-center text-xs text-muted-foreground ${className}`}
      >
        {t('eta_not_available')}
      </span>
    );
  }

  const confLabel = getConfidenceLabel(eta.confidence);

  return (
    <div
      data-testid="eta-card-badge"
      title={eta.confidence_downgraded ? t('eta_downgraded_notice') : undefined}
      className={`inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-xs font-medium bg-secondary text-secondary-foreground border border-border/50 ${className}`}
    >
      <Clock className="w-3 h-3 text-primary" />
      <span>{eta.eta_range_text}</span>
      <span className="text-[10px] text-muted-foreground">({confLabel})</span>
      {eta.confidence_downgraded && (
        <AlertTriangle
          data-testid="eta-downgraded-icon"
          className="w-3 h-3 text-amber-500"
        />
      )}
    </div>
  );
};

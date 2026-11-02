import React from 'react';
import { Clock, AlertTriangle, Info } from 'lucide-react';
import type { EtaResult, EtaConfidence } from './types';
import { t } from '../../core/i18n';

export interface EtaTaskSectionProps {
  eta?: EtaResult | null;
  className?: string;
}

function getConfidenceText(confidence?: EtaConfidence | null): string {
  if (confidence === 'high') return t('eta_confidence_high');
  if (confidence === 'medium') return t('eta_confidence_medium');
  return t('eta_confidence_low');
}

const EmptyEtaNotice: React.FC<{ reason?: string | null }> = ({ reason }) => (
  <div
    data-testid="eta-section-empty"
    className="flex items-center gap-2 p-3 text-xs text-muted-foreground bg-muted/40 rounded border border-dashed"
  >
    <Info className="w-4 h-4 flex-shrink-0" />
    <span>{reason || t('eta_no_data')}</span>
  </div>
);

const EtaHeader: React.FC = () => (
  <div className="flex items-center gap-1.5 font-medium text-sm text-foreground">
    <Clock className="w-4 h-4 text-primary" />
    <span>{t('eta_title')}</span>
  </div>
);

const EtaMetrics: React.FC<{ eta: EtaResult }> = ({ eta }) => (
  <div className="grid grid-cols-2 gap-2 text-xs">
    <div className="p-2 rounded bg-muted/30">
      <span className="text-muted-foreground block">{t('eta_median')}:</span>
      <span className="font-semibold">{eta.median_secs}{t("unit_second_short")}</span>
    </div>
    <div className="p-2 rounded bg-muted/30">
      <span className="text-muted-foreground block">{t('eta_p80')}:</span>
      <span className="font-semibold">{eta.p80_secs}{t("unit_second_short")}</span>
    </div>
    <div className="p-2 rounded bg-muted/30">
      <span className="text-muted-foreground block">{t('eta_samples')}:</span>
      <span className="font-semibold">{eta.sample_count}</span>
    </div>
    <div className="p-2 rounded bg-muted/30">
      <span className="text-muted-foreground block">{t('eta_confidence_label')}:</span>
      <span className="font-semibold capitalize">{getConfidenceText(eta.confidence)}</span>
    </div>
  </div>
);

export const EtaTaskSection: React.FC<EtaTaskSectionProps> = ({ eta, className = '' }) => {
  if (!eta || !eta.has_estimate) {
    return (
      <div className={`space-y-2 ${className}`}>
        <EtaHeader />
        <EmptyEtaNotice reason={eta?.reason} />
      </div>
    );
  }

  return (
    <div
      data-testid="eta-task-section"
      className={`space-y-3 p-3 rounded-lg border bg-card text-card-foreground shadow-sm ${className}`}
    >
      <EtaHeader />
      <div className="text-base font-semibold text-primary">{eta.eta_range_text}</div>
      <EtaMetrics eta={eta} />
      {eta.confidence_downgraded && (
        <div
          data-testid="eta-downgrade-warning"
          className="flex items-center gap-1.5 p-2 text-xs text-amber-700 bg-amber-50 rounded border border-amber-200 dark:bg-amber-950 dark:text-amber-300 dark:border-amber-800"
        >
          <AlertTriangle className="w-4 h-4 flex-shrink-0" />
          <span>{t('eta_downgraded_notice')}</span>
        </div>
      )}
    </div>
  );
};

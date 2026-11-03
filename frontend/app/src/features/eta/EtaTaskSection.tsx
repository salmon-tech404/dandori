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
    style={{
      display: "flex",
      alignItems: "center",
      gap: "var(--sp-8)",
      padding: "var(--sp-12)",
      fontSize: "var(--fs-12)",
      color: "var(--mu)",
      backgroundColor: "var(--sf2)",
      borderRadius: "var(--radius-4)",
      border: "1px dashed var(--ln)",
    }}
  >
    <Info style={{ width: 16, height: 16, flexShrink: 0 }} />
    <span>{reason || t('eta_no_data')}</span>
  </div>
);

const EtaHeader: React.FC = () => (
  <div style={{ display: "flex", alignItems: "center", gap: 6, fontWeight: 500, fontSize: "var(--fs-14)", color: "var(--fg)" }}>
    <Clock style={{ width: 16, height: 16, color: "var(--bl)" }} />
    <span>{t('eta_title')}</span>
  </div>
);

const EtaMetrics: React.FC<{ eta: EtaResult }> = ({ eta }) => (
  <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--sp-8)", fontSize: "var(--fs-12)" }}>
    <div style={{ padding: "var(--sp-8)", borderRadius: "var(--radius-4)", backgroundColor: "var(--sf2)" }}>
      <span style={{ color: "var(--mu)", display: "block" }}>{t('eta_median')}:</span>
      <span style={{ fontWeight: 600, color: "var(--fg)" }}>{eta.median_secs}{t("unit_second_short")}</span>
    </div>
    <div style={{ padding: "var(--sp-8)", borderRadius: "var(--radius-4)", backgroundColor: "var(--sf2)" }}>
      <span style={{ color: "var(--mu)", display: "block" }}>{t('eta_p80')}:</span>
      <span style={{ fontWeight: 600, color: "var(--fg)" }}>{eta.p80_secs}{t("unit_second_short")}</span>
    </div>
    <div style={{ padding: "var(--sp-8)", borderRadius: "var(--radius-4)", backgroundColor: "var(--sf2)" }}>
      <span style={{ color: "var(--mu)", display: "block" }}>{t('eta_samples')}:</span>
      <span style={{ fontWeight: 600, color: "var(--fg)" }}>{eta.sample_count}</span>
    </div>
    <div style={{ padding: "var(--sp-8)", borderRadius: "var(--radius-4)", backgroundColor: "var(--sf2)" }}>
      <span style={{ color: "var(--mu)", display: "block" }}>{t('eta_confidence_label')}:</span>
      <span style={{ fontWeight: 600, textTransform: "capitalize", color: "var(--fg)" }}>{getConfidenceText(eta.confidence)}</span>
    </div>
  </div>
);

const DowngradeWarning: React.FC = () => (
  <div
    data-testid="eta-downgrade-warning"
    style={{
      display: "flex",
      alignItems: "center",
      gap: 6,
      padding: "var(--sp-8)",
      fontSize: "var(--fs-12)",
      color: "var(--or)",
      backgroundColor: "var(--orb)",
      borderRadius: "var(--radius-4)",
      border: "1px solid var(--or)",
    }}
  >
    <AlertTriangle style={{ width: 16, height: 16, flexShrink: 0 }} />
    <span>{t('eta_downgraded_notice')}</span>
  </div>
);

const ActiveEtaSection: React.FC<{ eta: EtaResult }> = ({ eta }) => (
  <div
    data-testid="eta-task-section"
    style={{
      display: "flex",
      flexDirection: "column",
      gap: "var(--sp-12)",
      padding: "var(--sp-12)",
      borderRadius: "var(--radius-8)",
      border: "1px solid var(--ln)",
      backgroundColor: "var(--sf)",
    }}
  >
    <EtaHeader />
    <div style={{ fontSize: "var(--fs-16)", fontWeight: 600, color: "var(--bl)" }}>{eta.eta_range_text}</div>
    <EtaMetrics eta={eta} />
    {eta.confidence_downgraded && <DowngradeWarning />}
  </div>
);

export const EtaTaskSection: React.FC<EtaTaskSectionProps> = ({ eta }) => {
  if (!eta || !eta.has_estimate) {
    return (
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
        <EtaHeader />
        <EmptyEtaNotice reason={eta?.reason} />
      </div>
    );
  }
  return <ActiveEtaSection eta={eta} />;
};

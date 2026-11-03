import { useState } from "react";
import { Settings, AlertCircle } from "lucide-react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import type { QuotaStatusResponse, AccountQuotaConfigPayload, QuotaSource } from "./types";
import { QuotaConfigModal } from "./QuotaConfigModal";

interface QuotaStatusBarProps {
  status: QuotaStatusResponse;
  onUpdateConfig?: (payload: AccountQuotaConfigPayload) => Promise<void>;
}

function SourceChip({ source }: { source: QuotaSource }) {
  if (source === "reported") {
    return (
      <span style={{ borderRadius: "var(--radius-4)", backgroundColor: "var(--grb)", padding: "2px 6px", fontSize: "10px", fontWeight: 500, color: "var(--gr)" }}>
        {t("quota_source_reported")}
      </span>
    );
  }
  if (source === "estimated") {
    return (
      <span style={{ borderRadius: "var(--radius-4)", backgroundColor: "var(--blb)", padding: "2px 6px", fontSize: "10px", fontWeight: 500, color: "var(--bl)" }}>
        {t("quota_source_estimated")}
      </span>
    );
  }
  return (
    <span style={{ borderRadius: "var(--radius-4)", backgroundColor: "var(--sf2)", padding: "2px 6px", fontSize: "10px", fontWeight: 500, color: "var(--mu)" }}>
      {t("quota_source_no_data")}
    </span>
  );
}

interface UsageRowProps {
  label: string;
  used: number;
  limit: number | null;
  percentage: number | null;
  threshold: number;
}

function UsageRow({ label, used, limit, percentage, threshold }: UsageRowProps) {
  const isOver = percentage !== null && percentage >= threshold;
  const barColor = isOver ? "var(--or)" : "var(--bl)";
  const widthPercent = percentage !== null ? Math.min(percentage, 100) : 0;

  return (
    <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)", fontSize: "var(--fs-12)" }}>
      <span style={{ width: 48, fontWeight: 500, color: "var(--mu)" }}>{label}</span>
      <div style={{ height: 8, flex: 1, overflow: "hidden", borderRadius: "var(--radius-pill)", backgroundColor: "var(--sf2)" }}>
        <div style={{ height: "100%", backgroundColor: barColor, width: `${widthPercent}%`, transition: "all 0.2s" }} />
      </div>
      <span style={{ width: 96, textAlign: "right", fontSize: "11px", color: "var(--mu)" }}>
        {limit !== null ? `${used}/${limit} (${percentage}%)` : t("quota_source_no_data")}
      </span>
    </div>
  );
}

function ExhaustedAlert({ exhaustedUntil }: { exhaustedUntil: string }) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 6, borderRadius: "var(--radius-4)", backgroundColor: "var(--rdb)", padding: "4px 8px", fontSize: "11px", color: "var(--rd)" }}>
      <AlertCircle style={{ width: 14, height: 14, flexShrink: 0 }} />
      <span>{t("quota_exhausted_until")}: {exhaustedUntil}</span>
    </div>
  );
}

interface HeaderProps {
  source: QuotaSource;
  onOpenConfig?: () => void;
}

function HeaderRow({ source, onOpenConfig }: HeaderProps) {
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
        <span style={{ fontSize: "var(--fs-12)", fontWeight: 600, color: "var(--fg)" }}>
          {t("quota_tracking_title")}
        </span>
        <SourceChip source={source} />
      </div>
      {onOpenConfig && (
        <Button
          type="button"
          variant="ghost"
          size="sm"
          onClick={onOpenConfig}
          icon={<Settings style={{ width: 12, height: 12 }} />}
        >
          {t("quota_config_btn")}
        </Button>
      )}
    </div>
  );
}

export function QuotaStatusBar({ status, onUpdateConfig }: QuotaStatusBarProps) {
  const [isModalOpen, setIsModalOpen] = useState(false);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)", borderRadius: "var(--radius-6)", border: "1px solid var(--ln)", backgroundColor: "var(--sf)", padding: "10px" }}>
      <HeaderRow source={status.source} onOpenConfig={onUpdateConfig ? () => setIsModalOpen(true) : undefined} />
      {status.exhausted_until && <ExhaustedAlert exhaustedUntil={status.exhausted_until} />}
      <div style={{ display: "flex", flexDirection: "column", gap: 6, paddingTop: 2 }}>
        <UsageRow
          label={t("quota_tracking_5h")}
          used={status.used_5h}
          limit={status.limit_5h}
          percentage={status.percentage_5h}
          threshold={status.alert_threshold_percent}
        />
        <UsageRow
          label={t("quota_tracking_weekly")}
          used={status.used_weekly}
          limit={status.limit_weekly}
          percentage={status.percentage_weekly}
          threshold={status.alert_threshold_percent}
        />
      </div>
      {onUpdateConfig && (
        <QuotaConfigModal
          isOpen={isModalOpen}
          onClose={() => setIsModalOpen(false)}
          onSave={onUpdateConfig}
          initial5h={status.limit_5h}
          initialWeekly={status.limit_weekly}
          initialThreshold={status.alert_threshold_percent}
        />
      )}
    </div>
  );
}

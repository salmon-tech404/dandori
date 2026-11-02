import { useState } from "react";
import { Settings, AlertCircle } from "lucide-react";
import { t } from "../../core/i18n";
import type { QuotaStatusResponse, AccountQuotaConfigPayload, QuotaSource } from "./types";
import { QuotaConfigModal } from "./QuotaConfigModal";

interface QuotaStatusBarProps {
  status: QuotaStatusResponse;
  onUpdateConfig?: (payload: AccountQuotaConfigPayload) => Promise<void>;
}

function SourceChip({ source }: { source: QuotaSource }) {
  if (source === "reported") {
    return (
      <span className="rounded bg-emerald-100 px-1.5 py-0.5 text-[10px] font-medium text-emerald-800 dark:bg-emerald-950/60 dark:text-emerald-300">
        {t("quota_source_reported")}
      </span>
    );
  }
  if (source === "estimated") {
    return (
      <span className="rounded bg-sky-100 px-1.5 py-0.5 text-[10px] font-medium text-sky-800 dark:bg-sky-950/60 dark:text-sky-300">
        {t("quota_source_estimated")}
      </span>
    );
  }
  return (
    <span className="rounded bg-slate-100 px-1.5 py-0.5 text-[10px] font-medium text-slate-600 dark:bg-slate-800 dark:text-slate-400">
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
  const barClass = isOver ? "bg-amber-500" : "bg-blue-500";
  const widthPercent = percentage !== null ? Math.min(percentage, 100) : 0;

  return (
    <div className="flex items-center gap-2 text-xs">
      <span className="w-12 font-medium text-slate-600 dark:text-slate-400">{label}</span>
      <div className="h-2 flex-1 overflow-hidden rounded-full bg-slate-200 dark:bg-slate-700">
        <div className={`h-full transition-all ${barClass}`} style={{ width: `${widthPercent}%` }} />
      </div>
      <span className="w-24 text-right text-[11px] text-slate-500 dark:text-slate-400">
        {limit !== null ? `${used}/${limit} (${percentage}%)` : t("quota_source_no_data")}
      </span>
    </div>
  );
}

function ExhaustedAlert({ exhaustedUntil }: { exhaustedUntil: string }) {
  return (
    <div className="flex items-center gap-1.5 rounded bg-rose-50 px-2 py-1 text-[11px] text-rose-700 dark:bg-rose-950/40 dark:text-rose-300">
      <AlertCircle className="h-3.5 w-3.5 shrink-0" />
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
    <div className="flex items-center justify-between">
      <div className="flex items-center gap-2">
        <span className="text-xs font-semibold text-slate-800 dark:text-slate-200">
          {t("quota_tracking_title")}
        </span>
        <SourceChip source={source} />
      </div>
      {onOpenConfig && (
        <button
          type="button"
          onClick={onOpenConfig}
          className="flex items-center gap-1 text-[11px] font-medium text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"
        >
          <Settings className="h-3 w-3" />
          <span>{t("quota_config_btn")}</span>
        </button>
      )}
    </div>
  );
}

export function QuotaStatusBar({ status, onUpdateConfig }: QuotaStatusBarProps) {
  const [isModalOpen, setIsModalOpen] = useState(false);

  return (
    <div className="space-y-2 rounded border border-slate-200 bg-slate-50/50 p-2.5 dark:border-slate-800 dark:bg-slate-900/50">
      <HeaderRow source={status.source} onOpenConfig={onUpdateConfig ? () => setIsModalOpen(true) : undefined} />
      {status.exhausted_until && <ExhaustedAlert exhaustedUntil={status.exhausted_until} />}
      <div className="space-y-1.5 pt-0.5">
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

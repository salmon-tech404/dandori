import React, { useEffect, useState } from "react";
import { t } from "../../core/i18n";
import { fetchQuietStatus } from "./api";
import type { QuietStatus } from "./types";

interface Props {
  workspaceId?: string;
  projectId?: string;
  onOpenSettings?: () => void;
}

function formatRetryTime(isoStr: string | null): string {
  if (!isoStr) return "";
  try {
    const d = new Date(isoStr);
    return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
  } catch {
    return "";
  }
}

const BadgeContent: React.FC<{ status: QuietStatus }> = ({ status }) => {
  if (status.in_quiet_window) {
    return (
      <span className="flex items-center gap-1.5 text-amber-700 dark:text-amber-400 font-medium">
        <span className="w-2 h-2 rounded-full bg-amber-500 animate-pulse" />
        <span>{t("quiet_hours_badge_active")}</span>
        {status.retry_after && (
          <span className="text-[11px] opacity-80">
            ({t("quiet_hours_retry_after", { time: formatRetryTime(status.retry_after) })})
          </span>
        )}
      </span>
    );
  }
  return (
    <span className="flex items-center gap-1.5 text-emerald-700 dark:text-emerald-400 font-medium">
      <span className="w-2 h-2 rounded-full bg-emerald-500" />
      <span>{t("quiet_hours_badge_inactive")}</span>
    </span>
  );
};

export const QuietHoursToolbarBadge: React.FC<Props> = ({
  workspaceId = "default",
  projectId,
  onOpenSettings,
}) => {
  const [status, setStatus] = useState<QuietStatus | null>(null);

  useEffect(() => {
    fetchQuietStatus(workspaceId, projectId)
      .then(setStatus)
      .catch(() => setStatus(null));
  }, [workspaceId, projectId]);

  if (!status) return null;

  return (
    <div className="flex items-center gap-2 px-2 py-1 rounded text-xs border" data-testid="quiet-hours-toolbar-badge">
      <BadgeContent status={status} />
      {onOpenSettings && (
        <button
          type="button"
          onClick={onOpenSettings}
          className="ml-1 text-slate-500 hover:text-slate-700 dark:hover:text-slate-300 underline"
        >
          {t("quiet_hours_settings_btn")}
        </button>
      )}
    </div>
  );
};

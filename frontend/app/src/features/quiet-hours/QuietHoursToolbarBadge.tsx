import React, { useEffect, useState } from "react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
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
      <span style={{ display: "flex", alignItems: "center", gap: 6, color: "var(--or)", fontWeight: 500 }}>
        <span style={{ width: 8, height: 8, borderRadius: "var(--radius-pill)", backgroundColor: "var(--or)", animation: "dandori-pulse 1.5s infinite" }} />
        <span>{t("quiet_hours_badge_active")}</span>
        {status.retry_after && (
          <span style={{ fontSize: "11px", opacity: 0.8 }}>
            ({t("quiet_hours_retry_after", { time: formatRetryTime(status.retry_after) })})
          </span>
        )}
      </span>
    );
  }
  return (
    <span style={{ display: "flex", alignItems: "center", gap: 6, color: "var(--gr)", fontWeight: 500 }}>
      <span style={{ width: 8, height: 8, borderRadius: "var(--radius-pill)", backgroundColor: "var(--gr)" }} />
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
    <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)", padding: "4px 8px", borderRadius: "var(--radius-4)", fontSize: "var(--fs-12)", border: "1px solid var(--ln)", backgroundColor: "var(--sf)" }} data-testid="quiet-hours-toolbar-badge">
      <BadgeContent status={status} />
      {onOpenSettings && (
        <Button variant="ghost" size="sm" type="button" onClick={onOpenSettings} style={{ padding: 0, height: "auto", minHeight: "unset", textDecoration: "underline" }}>{t("quiet_hours_settings_btn")}</Button>
      )}
    </div>
  );
};

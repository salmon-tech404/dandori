import React, { useEffect, useState } from "react";
import { AlertTriangle } from "lucide-react";
import { t } from "../../../core/i18n";
import { testAgentApi } from "../api/testAgentApi";
import type { TestAgentRun } from "../types";

export interface TestResultBadgeProps {
  taskId?: string;
  initialRuns?: TestAgentRun[];
}

function getBadgeColors(status: string): { bg: string; fg: string; border: string } {
  if (status === "passed") return { bg: "var(--grb)", fg: "var(--gr)", border: "1px solid var(--gr)" };
  if (status === "failed") return { bg: "var(--rdb)", fg: "var(--rd)", border: "1px solid var(--rd)" };
  return { bg: "var(--blb)", fg: "var(--bl)", border: "1px solid var(--bl)" };
}

function getStatusLabel(status: string): string {
  if (status === "passed") return t("test_agent_status_passed");
  if (status === "failed") return t("test_agent_status_failed");
  return t("test_agent_status_running");
}

export const TestResultBadge: React.FC<TestResultBadgeProps> = ({ taskId, initialRuns }) => {
  const [latestRun, setLatestRun] = useState<TestAgentRun | null>(initialRuns?.[0] ?? null);

  useEffect(() => {
    if (initialRuns?.[0]) {
      setLatestRun(initialRuns[0]);
    } else if (taskId) {
      testAgentApi.listResults(taskId).then((runs) => {
        if (runs.length > 0) setLatestRun(runs[0]);
      }).catch(() => {});
    }
  }, [taskId, initialRuns]);

  if (!latestRun) return null;

  const flagged = latestRun.flagged_files && latestRun.flagged_files !== "[]";
  const { bg, fg, border } = getBadgeColors(latestRun.status);

  return (
    <span
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: "4px",
        padding: "2px 8px",
        borderRadius: "var(--radius-pill)",
        fontSize: "var(--fs-12)",
        fontWeight: 600,
        backgroundColor: bg,
        color: fg,
        border,
      }}
      title={latestRun.summary ?? undefined}
    >
      <span>{t("test_agent_badge_label")}: {getStatusLabel(latestRun.status)}</span>
      {flagged && (
        <span style={{ color: "var(--or)", fontWeight: "bold", display: "inline-flex", alignItems: "center" }} title={t("test_agent_flagged_warning")}>
          <AlertTriangle size={14} data-testid="warning-icon" />
        </span>
      )}
    </span>
  );
};

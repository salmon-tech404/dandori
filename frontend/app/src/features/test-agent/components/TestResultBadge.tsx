import React, { useEffect, useState } from "react";
import { AlertTriangle } from "lucide-react";
import { t } from "../../../core/i18n";
import { testAgentApi } from "../api/testAgentApi";
import type { TestAgentRun } from "../types";

export interface TestResultBadgeProps {
  taskId?: string;
  initialRuns?: TestAgentRun[];
}

function getBadgeStyle(status: string): string {
  if (status === "passed") return "bg-green-100 text-green-800 border-green-300";
  if (status === "failed") return "bg-red-100 text-red-800 border-red-300";
  return "bg-blue-100 text-blue-800 border-blue-300";
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
  const badgeClass = `inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs border font-medium ${getBadgeStyle(latestRun.status)}`;

  return (
    <span className={badgeClass} title={latestRun.summary ?? undefined}>
      <span>{t("test_agent_badge_label")}: {getStatusLabel(latestRun.status)}</span>
      {flagged && (
        <span className="text-amber-600 font-bold" title={t("test_agent_flagged_warning")}>
          <AlertTriangle className="w-3.5 h-3.5 inline" data-testid="warning-icon" />
        </span>
      )}
    </span>
  );
};

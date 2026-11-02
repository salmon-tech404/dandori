import React, { useEffect, useState } from "react";
import { AlertTriangle } from "lucide-react";
import { t } from "../../../core/i18n";
import { testAgentApi } from "../api/testAgentApi";
import type { TestAgentRun } from "../types";

export interface TestTaskSectionProps {
  taskId: string;
}

const FlaggedFilesList: React.FC<{ filesJson: string }> = ({ filesJson }) => {
  let files: string[] = [];
  try {
    files = JSON.parse(filesJson) as string[];
  } catch {
    files = [];
  }
  if (!files || files.length === 0) return null;

  return (
    <div className="mt-2 p-2 bg-amber-50 border border-amber-200 rounded text-xs">
      <div className="font-semibold text-amber-800 flex items-center gap-1">
        <AlertTriangle className="w-3.5 h-3.5" />
        <span>{t("test_agent_flagged_warning")}</span>
      </div>
      <ul className="list-disc list-inside mt-1 text-amber-700">
        {files.map((f) => (
          <li key={f}>{f}</li>
        ))}
      </ul>
    </div>
  );
};

const TestRunCard: React.FC<{ run: TestAgentRun }> = ({ run }) => {
  const isPassed = run.status === "passed";
  const badgeClass = isPassed ? "text-green-700 bg-green-100" : "text-red-700 bg-red-100";
  const label = isPassed ? t("test_agent_status_passed") : t("test_agent_status_failed");

  return (
    <div className="p-3 border rounded mb-2 bg-white text-sm">
      <div className="flex items-center justify-between">
        <span className={`px-2 py-0.5 rounded text-xs font-semibold ${badgeClass}`}>
          {label}
        </span>
        <span className="text-xs text-neutral-500">
          {t("test_agent_exit_code")}: {run.exit_code ?? "-"}
        </span>
      </div>
      {run.summary && <p className="mt-2 text-neutral-700">{run.summary}</p>}
      <FlaggedFilesList filesJson={run.flagged_files} />
    </div>
  );
};

export const TestTaskSection: React.FC<TestTaskSectionProps> = ({ taskId }) => {
  const [runs, setRuns] = useState<TestAgentRun[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    testAgentApi
      .listResults(taskId)
      .then((data) => setRuns(data))
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [taskId]);

  if (loading) return <div className="text-xs text-neutral-500">{t("test_agent_loading")}</div>;
  if (runs.length === 0) {
    return <div className="text-xs text-neutral-500">{t("test_agent_no_runs")}</div>;
  }

  return (
    <div className="space-y-2">
      <h4 className="font-semibold text-sm mb-2">{t("test_agent_title")}</h4>
      {runs.map((r) => (
        <TestRunCard key={r.id} run={r} />
      ))}
    </div>
  );
};

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
    <div style={{ marginTop: "var(--sp-8)", padding: "var(--sp-8)", backgroundColor: "var(--orb)", border: "1px solid var(--or)", borderRadius: "var(--radius-6)", fontSize: "var(--fs-12)" }}>
      <div style={{ fontWeight: 600, color: "var(--or)", display: "flex", alignItems: "center", gap: "var(--sp-4)" }}>
        <AlertTriangle size={14} />
        <span>{t("test_agent_flagged_warning")}</span>
      </div>
      <ul style={{ listStyle: "disc", paddingLeft: "var(--sp-16)", margin: "var(--sp-4) 0 0 0", color: "var(--or)" }}>
        {files.map((f) => (
          <li key={f}>{f}</li>
        ))}
      </ul>
    </div>
  );
};

const TestRunCard: React.FC<{ run: TestAgentRun }> = ({ run }) => {
  const isPassed = run.status === "passed";
  const badgeBg = isPassed ? "var(--grb)" : "var(--rdb)";
  const badgeFg = isPassed ? "var(--gr)" : "var(--rd)";
  const label = isPassed ? t("test_agent_status_passed") : t("test_agent_status_failed");

  return (
    <div style={{ padding: "var(--sp-12)", border: "1px solid var(--ln)", borderRadius: "var(--radius-8)", marginBottom: "var(--sp-8)", backgroundColor: "var(--sf)", fontSize: "var(--fs-13)" }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <span style={{ padding: "2px 8px", borderRadius: "var(--radius-pill)", fontSize: "var(--fs-12)", fontWeight: 600, backgroundColor: badgeBg, color: badgeFg }}>
          {label}
        </span>
        <span style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>
          {t("test_agent_exit_code")}: {run.exit_code ?? "-"}
        </span>
      </div>
      {run.summary && <p style={{ margin: "var(--sp-8) 0 0 0", color: "var(--fg)" }}>{run.summary}</p>}
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

  if (loading) return <div style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t("test_agent_loading")}</div>;
  if (runs.length === 0) {
    return <div style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t("test_agent_no_runs")}</div>;
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
      <h4 style={{ margin: "0 0 var(--sp-8) 0", fontSize: "var(--fs-14)", fontWeight: 600, color: "var(--fg)" }}>{t("test_agent_title")}</h4>
      {runs.map((r) => (
        <TestRunCard key={r.id} run={r} />
      ))}
    </div>
  );
};

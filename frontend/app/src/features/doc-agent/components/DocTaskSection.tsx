import React, { useEffect, useState } from "react";
import { AlertTriangle, FileText } from "lucide-react";
import { t } from "../../../core/i18n";
import { docAgentApi } from "../api/docAgentApi";
import type { DocAgentRun } from "../types";

export interface DocTaskSectionProps {
  taskId: string;
}

function parseJsonFiles(json: string): string[] {
  try {
    return JSON.parse(json) as string[];
  } catch {
    return [];
  }
}

const FlaggedFilesNotice: React.FC<{ filesJson: string }> = ({ filesJson }) => {
  const files = parseJsonFiles(filesJson);
  if (files.length === 0) return null;

  return (
    <div style={{ marginTop: "var(--sp-8)", padding: "var(--sp-8)", backgroundColor: "var(--orb)", border: "1px solid var(--or)", borderRadius: "var(--radius-6)", fontSize: "var(--fs-12)" }}>
      <div style={{ fontWeight: 600, color: "var(--or)", display: "flex", alignItems: "center", gap: "var(--sp-4)" }}>
        <AlertTriangle size={14} />
        <span>{t("doc_agent_flagged_warning")}</span>
      </div>
      <ul style={{ listStyle: "disc", paddingLeft: "var(--sp-16)", margin: "var(--sp-4) 0 0 0", color: "var(--or)" }}>
        {files.map((f) => (
          <li key={f}>{f}</li>
        ))}
      </ul>
    </div>
  );
};

const ChangedDocsList: React.FC<{ filesJson: string }> = ({ filesJson }) => {
  const files = parseJsonFiles(filesJson);
  if (files.length === 0) return null;

  return (
    <div style={{ marginTop: "var(--sp-8)", fontSize: "var(--fs-12)" }}>
      <div style={{ fontWeight: 500, color: "var(--mu)", display: "flex", alignItems: "center", gap: "var(--sp-4)" }}>
        <FileText size={14} />
        <span>{t("doc_agent_changed_files_title")}</span>
      </div>
      <ul style={{ listStyle: "disc", paddingLeft: "var(--sp-16)", margin: "var(--sp-4) 0 0 0", color: "var(--fg)" }}>
        {files.map((f) => (
          <li key={f}>{f}</li>
        ))}
      </ul>
    </div>
  );
};

const DocRunCard: React.FC<{ run: DocAgentRun }> = ({ run }) => {
  const isCompleted = run.status === "completed";
  const badgeBg = isCompleted ? "var(--grb)" : "var(--orb)";
  const badgeFg = isCompleted ? "var(--gr)" : "var(--or)";
  const label = isCompleted
    ? t("doc_agent_status_completed")
    : t("doc_agent_status_running");

  return (
    <div style={{ padding: "var(--sp-12)", border: "1px solid var(--ln)", borderRadius: "var(--radius-8)", marginBottom: "var(--sp-8)", backgroundColor: "var(--sf)", fontSize: "var(--fs-13)" }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <span style={{ padding: "2px 8px", borderRadius: "var(--radius-pill)", fontSize: "var(--fs-12)", fontWeight: 600, backgroundColor: badgeBg, color: badgeFg }}>
          {label}
        </span>
      </div>
      {run.summary && <p style={{ margin: "var(--sp-8) 0 0 0", color: "var(--fg)" }}>{run.summary}</p>}
      <ChangedDocsList filesJson={run.changed_doc_files} />
      <FlaggedFilesNotice filesJson={run.flagged_files} />
    </div>
  );
};

export const DocTaskSection: React.FC<DocTaskSectionProps> = ({ taskId }) => {
  const [runs, setRuns] = useState<DocAgentRun[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    docAgentApi
      .listResults(taskId)
      .then((data) => setRuns(data))
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [taskId]);

  if (loading) return <div style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t("doc_agent_loading")}</div>;
  if (runs.length === 0) {
    return <div style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t("doc_agent_no_runs")}</div>;
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
      <h4 style={{ margin: "0 0 var(--sp-8) 0", fontSize: "var(--fs-14)", fontWeight: 600, color: "var(--fg)" }}>{t("doc_agent_title")}</h4>
      {runs.map((r) => (
        <DocRunCard key={r.id} run={r} />
      ))}
    </div>
  );
};

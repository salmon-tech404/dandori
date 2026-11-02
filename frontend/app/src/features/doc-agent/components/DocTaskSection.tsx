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
    <div className="mt-2 p-2 bg-amber-50 border border-amber-200 rounded text-xs">
      <div className="font-semibold text-amber-800 flex items-center gap-1">
        <AlertTriangle className="w-3.5 h-3.5" />
        <span>{t("doc_agent_flagged_warning")}</span>
      </div>
      <ul className="list-disc list-inside mt-1 text-amber-700">
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
    <div className="mt-2 text-xs">
      <div className="font-medium text-neutral-700 flex items-center gap-1">
        <FileText className="w-3.5 h-3.5" />
        <span>{t("doc_agent_changed_files_title")}</span>
      </div>
      <ul className="list-disc list-inside mt-1 text-neutral-600">
        {files.map((f) => (
          <li key={f}>{f}</li>
        ))}
      </ul>
    </div>
  );
};

const DocRunCard: React.FC<{ run: DocAgentRun }> = ({ run }) => {
  const isCompleted = run.status === "completed";
  const badgeClass = isCompleted
    ? "text-green-700 bg-green-100"
    : "text-amber-700 bg-amber-100";
  const label = isCompleted
    ? t("doc_agent_status_completed")
    : t("doc_agent_status_running");

  return (
    <div className="p-3 border rounded mb-2 bg-white text-sm">
      <div className="flex items-center justify-between">
        <span className={`px-2 py-0.5 rounded text-xs font-semibold ${badgeClass}`}>
          {label}
        </span>
      </div>
      {run.summary && <p className="mt-2 text-neutral-700">{run.summary}</p>}
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

  if (loading) return <div className="text-xs text-neutral-500">{t("doc_agent_loading")}</div>;
  if (runs.length === 0) {
    return <div className="text-xs text-neutral-500">{t("doc_agent_no_runs")}</div>;
  }

  return (
    <div className="space-y-2">
      <h4 className="font-semibold text-sm mb-2">{t("doc_agent_title")}</h4>
      {runs.map((r) => (
        <DocRunCard key={r.id} run={r} />
      ))}
    </div>
  );
};

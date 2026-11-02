import { useEffect, useState } from "react";
import { t } from "../../core/i18n";
import { approveJob, fetchJobs, fireJobNow, rejectJob, syncJobs } from "./api";
import { JobCard } from "./JobCard";
import { JobDetailModal } from "./JobDetailModal";
import type { MdJobDto } from "./types";

interface MdJobsPanelProps {
  projectId?: string;
}

function useMdJobs(projectId?: string) {
  const [jobs, setJobs] = useState<MdJobDto[]>([]);
  const [syncing, setSyncing] = useState(false);

  const loadJobs = async () => {
    if (!projectId) return;
    try {
      setJobs(await fetchJobs(projectId));
    } catch {
      // Bỏ qua lỗi kết nối
    }
  };

  useEffect(() => {
    loadJobs();
  }, [projectId]);

  const handleSync = async () => {
    if (!projectId) return;
    setSyncing(true);
    try {
      await syncJobs(projectId);
      await loadJobs();
    } finally {
      setSyncing(false);
    }
  };

  const handleAction = async (action: () => Promise<void>) => {
    await action();
    await loadJobs();
  };

  return { jobs, syncing, handleSync, handleAction };
}

function renderPanelHeader(onSync: () => void, syncing: boolean) {
  return (
    <div className="flex flex-wrap items-center justify-between gap-3 pb-3 border-b border-border">
      <div className="flex items-center gap-2">
        <h3 className="font-bold text-text-primary">{t("md_jobs_title")}</h3>
        <span className="font-mono text-xs px-2 py-0.5 rounded bg-surface-muted text-text-secondary border border-border">
          {t("md_jobs_path_badge")}
        </span>
      </div>
      <button
        type="button"
        onClick={onSync}
        disabled={syncing}
        className="px-3 py-1.5 rounded bg-surface-muted hover:bg-surface-muted/80 text-text-primary text-xs font-medium border border-border disabled:opacity-50"
      >
        {syncing ? t("md_jobs_syncing") : t("md_jobs_sync")}
      </button>
    </div>
  );
}

function renderJobsList(
  jobs: MdJobDto[],
  onView: (j: MdJobDto) => void,
  onApprove: (id: string) => void,
  onReject: (id: string) => void,
  onFireNow: (id: string) => void
) {
  if (jobs.length === 0) {
    return <div className="py-8 text-center text-sm text-text-muted">{t("md_jobs_no_jobs")}</div>;
  }
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
      {jobs.map((job) => (
        <JobCard
          key={job.id}
          job={job}
          onView={onView}
          onApprove={onApprove}
          onReject={onReject}
          onFireNow={onFireNow}
        />
      ))}
    </div>
  );
}

export function MdJobsPanel({ projectId }: MdJobsPanelProps) {
  const { jobs, syncing, handleSync, handleAction } = useMdJobs(projectId);
  const [selectedJob, setSelectedJob] = useState<MdJobDto | null>(null);

  return (
    <div className="rounded-lg border border-border bg-surface p-4 shadow-sm space-y-4">
      {renderPanelHeader(handleSync, syncing)}
      {renderJobsList(
        jobs,
        setSelectedJob,
        (id) => handleAction(() => approveJob(id)),
        (id) => handleAction(() => rejectJob(id)),
        (id) => handleAction(() => fireJobNow(id))
      )}
      {selectedJob && <JobDetailModal job={selectedJob} onClose={() => setSelectedJob(null)} />}
    </div>
  );
}

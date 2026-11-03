import { useEffect, useState } from "react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
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
    void loadJobs();
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
    <div style={{ display: "flex", flexWrap: "wrap", alignItems: "center", justifyContent: "space-between", gap: "var(--sp-12)", paddingBottom: "var(--sp-12)", borderBottom: "1px solid var(--ln)" }}>
      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
        <h3 style={{ margin: 0, fontWeight: 700, color: "var(--fg)" }}>{t("md_jobs_title")}</h3>
        <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--fs-12)", padding: "2px 8px", borderRadius: "var(--radius-4)", backgroundColor: "var(--sf2)", color: "var(--mu)", border: "1px solid var(--ln)" }}>
          {t("md_jobs_path_badge")}
        </span>
      </div>
      <Button
        variant="secondary"
        size="sm"
        type="button"
        onClick={onSync}
        disabled={syncing}
        loading={syncing}
      >
        {syncing ? t("md_jobs_syncing") : t("md_jobs_sync")}
      </Button>
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
    return <div style={{ padding: "var(--sp-32) 0", textAlign: "center", fontSize: "var(--fs-14)", color: "var(--mu)" }}>{t("md_jobs_no_jobs")}</div>;
  }
  return (
    <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))", gap: "var(--sp-16)" }}>
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
    <div style={{ borderRadius: "var(--radius-8)", border: "1px solid var(--ln)", backgroundColor: "var(--sf)", padding: "var(--sp-16)", display: "flex", flexDirection: "column", gap: "var(--sp-16)" }}>
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

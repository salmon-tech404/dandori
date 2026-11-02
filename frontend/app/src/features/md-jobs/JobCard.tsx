import { t } from "../../core/i18n";
import type { MdJobDto } from "./types";

interface JobCardProps {
  job: MdJobDto;
  onView: (job: MdJobDto) => void;
  onApprove: (id: string) => void;
  onReject: (id: string) => void;
  onFireNow: (id: string) => void;
}

function renderStatusBadge(status: MdJobDto["status"]) {
  const configs = {
    pending_approval: { label: t("md_jobs_status_pending"), color: "bg-warning/10 text-warning border-warning/20" },
    approved: { label: t("md_jobs_status_approved"), color: "bg-success/10 text-success border-success/20" },
    rejected: { label: t("md_jobs_status_rejected"), color: "bg-danger/10 text-danger border-danger/20" },
    syntax_error: { label: t("md_jobs_status_syntax_error"), color: "bg-danger/20 text-danger border-danger/40" },
  };
  const cfg = configs[status] || configs.pending_approval;
  return <span className={`px-2 py-0.5 text-xs font-medium rounded border ${cfg.color}`}>{cfg.label}</span>;
}

function renderCardActions(
  job: MdJobDto,
  onView: () => void,
  onApprove: () => void,
  onReject: () => void,
  onFireNow: () => void
) {
  return (
    <div className="flex flex-wrap items-center gap-2 pt-2 border-t border-border">
      <button type="button" onClick={onView} className="px-2.5 py-1 text-xs rounded bg-surface-muted hover:bg-surface-muted/80 text-text-primary">
        {t("md_jobs_view_content")}
      </button>
      {job.status === "pending_approval" && (
        <>
          <button type="button" onClick={onApprove} className="px-2.5 py-1 text-xs rounded bg-success text-white hover:bg-success/90 font-medium">
            {t("md_jobs_approve")}
          </button>
          <button type="button" onClick={onReject} className="px-2.5 py-1 text-xs rounded bg-danger/10 text-danger hover:bg-danger/20">
            {t("md_jobs_reject")}
          </button>
        </>
      )}
      {job.status === "approved" && (
        <button type="button" onClick={onFireNow} className="px-2.5 py-1 text-xs rounded bg-primary text-white hover:bg-primary/90 font-medium">
          {t("md_jobs_fire_now")}
        </button>
      )}
    </div>
  );
}

export function JobCard({ job, onView, onApprove, onReject, onFireNow }: JobCardProps) {
  return (
    <div className="rounded-lg border border-border bg-surface p-4 shadow-sm space-y-3">
      <div className="flex items-start justify-between gap-2">
        <div>
          <h4 className="font-semibold text-text-primary">{job.job_name}</h4>
          <span className="font-mono text-xs text-text-muted">{job.file_path}</span>
        </div>
        {renderStatusBadge(job.status)}
      </div>
      <div className="grid grid-cols-2 gap-2 text-xs text-text-secondary">
        <div>
          <span className="text-text-muted">{t("md_jobs_schedule")} </span>
          <code className="font-mono bg-surface-muted px-1 rounded">{job.schedule_cron}</code>
        </div>
        <div>
          <span className="text-text-muted">{t("md_jobs_agent")} </span>
          <span className="font-medium text-text-primary">{job.agent_role}</span>
        </div>
      </div>
      {job.next_fire && (
        <div className="text-xs text-text-muted">
          <span>{t("md_jobs_next_run")} </span>
          <span className="font-medium text-text-secondary">{new Date(job.next_fire).toLocaleString("vi-VN")}</span>
        </div>
      )}
      {renderCardActions(
        job,
        () => onView(job),
        () => onApprove(job.id),
        () => onReject(job.id),
        () => onFireNow(job.id)
      )}
    </div>
  );
}

import { CSSProperties } from "react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import type { MdJobDto } from "./types";

interface JobCardProps {
  job: MdJobDto;
  onView: (job: MdJobDto) => void;
  onApprove: (id: string) => void;
  onReject: (id: string) => void;
  onFireNow: (id: string) => void;
}

function renderStatusBadge(status: MdJobDto["status"]) {
  const configs: Record<string, { label: string; style: CSSProperties }> = {
    pending_approval: { label: t("md_jobs_status_pending"), style: { backgroundColor: "var(--orb)", color: "var(--or)", border: "1px solid var(--or)" } },
    approved: { label: t("md_jobs_status_approved"), style: { backgroundColor: "var(--grb)", color: "var(--gr)", border: "1px solid var(--gr)" } },
    rejected: { label: t("md_jobs_status_rejected"), style: { backgroundColor: "var(--rdb)", color: "var(--rd)", border: "1px solid var(--rd)" } },
    syntax_error: { label: t("md_jobs_status_syntax_error"), style: { backgroundColor: "var(--rdb)", color: "var(--rd)", border: "1px solid var(--rd)" } },
  };
  const cfg = configs[status] || configs.pending_approval;
  return <span style={{ padding: "2px 8px", fontSize: "var(--fs-12)", fontWeight: 500, borderRadius: "var(--radius-4)", ...cfg.style }}>{cfg.label}</span>;
}

function renderCardActions(
  job: MdJobDto,
  onView: () => void,
  onApprove: () => void,
  onReject: () => void,
  onFireNow: () => void
) {
  return (
    <div style={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: "var(--sp-8)", paddingTop: "var(--sp-8)", borderTop: "1px solid var(--ln)" }}>
      <Button variant="ghost" size="sm" type="button" onClick={onView}>
        {t("md_jobs_view_content")}
      </Button>
      {job.status === "pending_approval" && (
        <>
          <Button variant="primary" size="sm" type="button" onClick={onApprove}>
            {t("md_jobs_approve")}
          </Button>
          <Button variant="danger" size="sm" type="button" onClick={onReject}>
            {t("md_jobs_reject")}
          </Button>
        </>
      )}
      {job.status === "approved" && (
        <Button variant="primary" size="sm" type="button" onClick={onFireNow}>
          {t("md_jobs_fire_now")}
        </Button>
      )}
    </div>
  );
}

export function JobCard({ job, onView, onApprove, onReject, onFireNow }: JobCardProps) {
  return (
    <div style={{ borderRadius: "var(--radius-8)", border: "1px solid var(--ln)", backgroundColor: "var(--sf)", padding: "var(--sp-16)", display: "flex", flexDirection: "column", gap: "var(--sp-12)" }}>
      <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: "var(--sp-8)" }}>
        <div>
          <h4 style={{ margin: 0, fontWeight: 600, color: "var(--fg)" }}>{job.job_name}</h4>
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--fs-12)", color: "var(--mu)" }}>{job.file_path}</span>
        </div>
        {renderStatusBadge(job.status)}
      </div>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--sp-8)", fontSize: "var(--fs-12)", color: "var(--mu)" }}>
        <div>
          <span style={{ color: "var(--mu)" }}>{t("md_jobs_schedule")} </span>
          <code style={{ fontFamily: "var(--font-mono)", backgroundColor: "var(--sf2)", padding: "2px 4px", borderRadius: "var(--radius-4)", color: "var(--fg)" }}>{job.schedule_cron}</code>
        </div>
        <div>
          <span style={{ color: "var(--mu)" }}>{t("md_jobs_agent")} </span>
          <span style={{ fontWeight: 500, color: "var(--fg)" }}>{job.agent_role}</span>
        </div>
      </div>
      {job.next_fire && (
        <div style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>
          <span>{t("md_jobs_next_run")} </span>
          <span style={{ fontWeight: 500, color: "var(--fg)" }}>{new Date(job.next_fire).toLocaleString("vi-VN")}</span>
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

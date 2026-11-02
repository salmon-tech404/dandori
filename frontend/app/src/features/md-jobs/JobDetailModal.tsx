import { t } from "../../core/i18n";
import type { MdJobDto } from "./types";

interface JobDetailModalProps {
  job: MdJobDto;
  onClose: () => void;
}

function renderModalBody(job: MdJobDto) {
  return (
    <div className="mt-4 space-y-4 max-h-[60vh] overflow-y-auto">
      <div>
        <h4 className="text-xs font-semibold text-text-muted uppercase">{t("md_jobs_frontmatter")}</h4>
        <div className="mt-1 p-3 rounded bg-surface-muted font-mono text-xs text-text-secondary space-y-1">
          <div>{t("job_field_name")} {job.job_name}</div>
          <div>{t("job_field_schedule")} {job.schedule_cron}</div>
          <div>{t("job_field_agent")} {job.agent_role}</div>
          {job.account_id && <div>{t("job_field_account")} {job.account_id}</div>}
        </div>
      </div>
      <div>
        <h4 className="text-xs font-semibold text-text-muted uppercase">{t("md_jobs_prompt_body")}</h4>
        <pre className="mt-1 p-3 rounded bg-surface-muted font-mono text-xs text-text-primary whitespace-pre-wrap">
          {job.prompt}
        </pre>
      </div>
    </div>
  );
}

export function JobDetailModal({ job, onClose }: JobDetailModalProps) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div className="w-full max-w-2xl rounded-lg bg-surface p-6 shadow-xl border border-border">
        <div className="flex items-center justify-between pb-4 border-b border-border">
          <h3 className="text-lg font-bold text-text-primary">{job.job_name}</h3>
          <span className="font-mono text-xs px-2 py-1 rounded bg-surface-muted text-text-secondary">
            {job.file_path}
          </span>
        </div>
        {renderModalBody(job)}
        <div className="mt-6 flex justify-end">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 rounded bg-surface-muted hover:bg-surface-muted/80 text-text-primary text-sm font-medium"
          >
            {t("md_jobs_close")}
          </button>
        </div>
      </div>
    </div>
  );
}

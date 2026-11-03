import { t } from "../../core/i18n";
import { Modal } from "../../ui/components/Modal";
import { Button } from "../../ui/components/Button";
import type { MdJobDto } from "./types";

interface JobDetailModalProps {
  job: MdJobDto;
  onClose: () => void;
}

function renderModalBody(job: MdJobDto) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-16)" }}>
      <div>
        <h4 style={{ margin: "0 0 var(--sp-4) 0", fontSize: "var(--fs-12)", fontWeight: 600, color: "var(--mu)", textTransform: "uppercase" }}>
          {t("md_jobs_frontmatter")}
        </h4>
        <div style={{ padding: "var(--sp-12)", borderRadius: "var(--radius-8)", backgroundColor: "var(--sf2)", fontFamily: "var(--font-mono)", fontSize: "var(--fs-12)", color: "var(--fg)", display: "flex", flexDirection: "column", gap: "var(--sp-4)" }}>
          <div>{t("job_field_name")} {job.job_name}</div>
          <div>{t("job_field_schedule")} {job.schedule_cron}</div>
          <div>{t("job_field_agent")} {job.agent_role}</div>
          {job.account_id && <div>{t("job_field_account")} {job.account_id}</div>}
        </div>
      </div>
      <div>
        <h4 style={{ margin: "0 0 var(--sp-4) 0", fontSize: "var(--fs-12)", fontWeight: 600, color: "var(--mu)", textTransform: "uppercase" }}>
          {t("md_jobs_prompt_body")}
        </h4>
        <pre style={{ margin: 0, padding: "var(--sp-12)", borderRadius: "var(--radius-8)", backgroundColor: "var(--sf2)", fontFamily: "var(--font-mono)", fontSize: "var(--fs-12)", color: "var(--fg)", whiteSpace: "pre-wrap" }}>
          {job.prompt}
        </pre>
      </div>
    </div>
  );
}

export function JobDetailModal({ job, onClose }: JobDetailModalProps) {
  const title = (
    <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-12)" }}>
      <span>{job.job_name}</span>
      <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--fs-12)", padding: "2px 8px", borderRadius: "var(--radius-pill)", backgroundColor: "var(--sf2)", color: "var(--mu)" }}>
        {job.file_path}
      </span>
    </div>
  );

  return (
    <Modal
      open={true}
      onClose={onClose}
      title={title}
      size="lg"
      footer={
        <Button variant="secondary" onClick={onClose}>
          {t("md_jobs_close")}
        </Button>
      }
    >
      {renderModalBody(job)}
    </Modal>
  );
}

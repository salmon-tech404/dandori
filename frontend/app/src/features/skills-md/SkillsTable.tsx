import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import type { SkillSummary } from "./types";

interface SkillsTableProps {
  skills: SkillSummary[];
  onViewDetail: (id: string) => void;
  onApprove: (id: string) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
}

const SkillsTableHeader = () => (
  <thead
    style={{
      borderBottom: "1px solid var(--ln)",
      backgroundColor: "var(--sf2)",
      fontSize: "var(--fs-12)",
      textTransform: "uppercase",
      color: "var(--mu)",
    }}
  >
    <tr>
      <th style={{ padding: "var(--sp-8) var(--sp-16)" }}>{t("skills_col_name_desc")}</th>
      <th style={{ padding: "var(--sp-8) var(--sp-16)" }}>{t("skills_col_status")}</th>
      <th style={{ padding: "var(--sp-8) var(--sp-16)" }}>{t("skills_col_script")}</th>
      <th style={{ padding: "var(--sp-8) var(--sp-16)" }}>{t("skills_col_bindings")}</th>
      <th style={{ padding: "var(--sp-8) var(--sp-16)", textAlign: "right" }}>{t("skills_col_actions")}</th>
    </tr>
  </thead>
);

export function SkillsTable({ skills, onViewDetail, onApprove, onDelete }: SkillsTableProps) {
  return (
    <div
      style={{
        overflowX: "auto",
        borderRadius: "var(--radius-8)",
        border: "1px solid var(--ln)",
        backgroundColor: "var(--sf)",
      }}
    >
      <table style={{ width: "100%", textAlign: "left", fontSize: "var(--fs-13)", borderCollapse: "collapse" }}>
        <SkillsTableHeader />
        <tbody>
          {skills.map((s) => (
            <SkillTableRow
              key={s.id}
              skill={s}
              onViewDetail={onViewDetail}
              onApprove={onApprove}
              onDelete={onDelete}
            />
          ))}
        </tbody>
      </table>
    </div>
  );
}

interface SkillTableRowProps {
  skill: SkillSummary;
  onViewDetail: (id: string) => void;
  onApprove: (id: string) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
}

function SkillTableRow({ skill, onViewDetail, onApprove, onDelete }: SkillTableRowProps) {
  const isApproved = skill.status === "approved" && skill.is_hash_matching_approved;

  return (
    <tr style={{ borderBottom: "1px solid var(--ln)" }}>
      <td style={{ padding: "var(--sp-8) var(--sp-16)" }}>
        <div style={{ fontWeight: 600, color: "var(--fg)" }}>{skill.name}</div>
        <div style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{skill.description}</div>
      </td>
      <td style={{ padding: "var(--sp-8) var(--sp-16)" }}>
        <StatusBadge isApproved={isApproved} />
      </td>
      <td style={{ padding: "var(--sp-8) var(--sp-16)" }}>
        <ScriptBadge hasScripts={skill.has_scripts} />
      </td>
      <td style={{ padding: "var(--sp-8) var(--sp-16)", fontSize: "var(--fs-12)", color: "var(--mu)" }}>
        {t("skills_binding_count", { count: skill.binding_count })}
      </td>
      <td style={{ padding: "var(--sp-8) var(--sp-16)", textAlign: "right" }}>
        <RowActions
          id={skill.id}
          isApproved={isApproved}
          onViewDetail={onViewDetail}
          onApprove={onApprove}
          onDelete={onDelete}
        />
      </td>
    </tr>
  );
}

function StatusBadge({ isApproved }: { isApproved: boolean }) {
  if (isApproved) {
    return (
      <span
        style={{
          borderRadius: "var(--radius-6)",
          backgroundColor: "rgba(34, 197, 94, 0.15)",
          padding: "2px 8px",
          fontSize: "var(--fs-12)",
          fontWeight: 500,
          color: "var(--gr)",
        }}
      >
        {t("skills_status_approved")}
      </span>
    );
  }
  return (
    <span
      style={{
        borderRadius: "var(--radius-6)",
        backgroundColor: "rgba(234, 179, 8, 0.15)",
        padding: "2px 8px",
        fontSize: "var(--fs-12)",
        fontWeight: 500,
        color: "var(--or)",
      }}
    >
      {t("skills_status_pending")}
    </span>
  );
}

function ScriptBadge({ hasScripts }: { hasScripts: boolean }) {
  if (hasScripts) {
    return (
      <span
        style={{
          borderRadius: "var(--radius-6)",
          backgroundColor: "rgba(249, 115, 22, 0.15)",
          padding: "2px 8px",
          fontSize: "var(--fs-12)",
          fontWeight: 500,
          color: "#ea580c",
        }}
      >
        {t("skills_has_scripts_badge")}
      </span>
    );
  }
  return <span style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t("skills_no_scripts_badge")}</span>;
}

interface RowActionsProps {
  id: string;
  isApproved: boolean;
  onViewDetail: (id: string) => void;
  onApprove: (id: string) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
}

function RowActions({ id, isApproved, onViewDetail, onApprove, onDelete }: RowActionsProps) {
  return (
    <div style={{ display: "inline-flex", gap: "var(--sp-4)", justifyContent: "flex-end" }}>
      <Button
        type="button"
        variant="ghost"
        size="sm"
        onClick={() => onViewDetail(id)}
      >
        {t("skills_view_detail")}
      </Button>
      {!isApproved && (
        <Button
          type="button"
          variant="ghost"
          size="sm"
          onClick={() => void onApprove(id)}
        >
          {t("skills_approve")}
        </Button>
      )}
      <Button
        type="button"
        variant="danger"
        size="sm"
        onClick={() => void onDelete(id)}
      >
        {t("skills_delete")}
      </Button>
    </div>
  );
}

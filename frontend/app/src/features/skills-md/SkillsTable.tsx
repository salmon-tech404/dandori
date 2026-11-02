import { t } from "../../core/i18n";
import type { SkillSummary } from "./types";

interface SkillsTableProps {
  skills: SkillSummary[];
  onViewDetail: (id: string) => void;
  onApprove: (id: string) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
}

export function SkillsTable({ skills, onViewDetail, onApprove, onDelete }: SkillsTableProps) {
  return (
    <div className="overflow-x-auto rounded border bg-white">
      <table className="w-full text-left text-sm">
        <thead className="border-b bg-gray-50 text-xs uppercase text-gray-600">
          <tr>
            <th className="px-4 py-3">{t("skills_col_name_desc")}</th>
            <th className="px-4 py-3">{t("skills_col_status")}</th>
            <th className="px-4 py-3">{t("skills_col_script")}</th>
            <th className="px-4 py-3">{t("skills_col_bindings")}</th>
            <th className="px-4 py-3 text-right">{t("skills_col_actions")}</th>
          </tr>
        </thead>
        <tbody className="divide-y">
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
    <tr className="hover:bg-gray-50">
      <td className="px-4 py-3">
        <div className="font-semibold text-gray-900">{skill.name}</div>
        <div className="text-xs text-gray-500">{skill.description}</div>
      </td>
      <td className="px-4 py-3">
        <StatusBadge isApproved={isApproved} />
      </td>
      <td className="px-4 py-3">
        <ScriptBadge hasScripts={skill.has_scripts} />
      </td>
      <td className="px-4 py-3 text-xs text-gray-600">
        {t("skills_binding_count", { count: skill.binding_count })}
      </td>
      <td className="px-4 py-3 text-right space-x-2">
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
      <span className="rounded bg-green-100 px-2 py-1 text-xs font-medium text-green-800">
        {t("skills_status_approved")}
      </span>
    );
  }
  return (
    <span className="rounded bg-yellow-100 px-2 py-1 text-xs font-medium text-yellow-800">
      {t("skills_status_pending")}
    </span>
  );
}

function ScriptBadge({ hasScripts }: { hasScripts: boolean }) {
  if (hasScripts) {
    return (
      <span className="rounded bg-orange-100 px-2 py-1 text-xs font-medium text-orange-800">
        {t("skills_has_scripts_badge")}
      </span>
    );
  }
  return <span className="text-xs text-gray-400">{t("skills_no_scripts_badge")}</span>;
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
    <>
      <button
        type="button"
        onClick={() => onViewDetail(id)}
        className="text-xs text-blue-600 hover:underline"
      >
        {t("skills_view_detail")}
      </button>
      {!isApproved && (
        <button
          type="button"
          onClick={() => void onApprove(id)}
          className="text-xs font-medium text-green-600 hover:underline"
        >
          {t("skills_approve")}
        </button>
      )}
      <button
        type="button"
        onClick={() => void onDelete(id)}
        className="text-xs text-red-600 hover:underline"
      >
        {t("skills_delete")}
      </button>
    </>
  );
}

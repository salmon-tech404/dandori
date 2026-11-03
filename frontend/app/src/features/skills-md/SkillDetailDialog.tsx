import { useState } from "react";
import { t } from "../../core/i18n";
import { Modal } from "../../ui/components/Modal";
import { Button } from "../../ui/components/Button";
import { Input } from "../../ui/components/Input";
import { Select } from "../../ui/components/Select";
import { Checkbox } from "../../ui/components/Checkbox";
import { createBinding, deleteBinding } from "./api";
import type { SkillDetail } from "./types";

interface SkillDetailDialogProps {
  workspaceId: string;
  skill: SkillDetail;
  onClose: () => void;
  onReload: () => void;
}

function useBindingActions(workspaceId: string, skillId: string, onReload: () => void) {
  const [projectId, setProjectId] = useState("");
  const [role, setRole] = useState("worker");
  const [embedFull, setEmbedFull] = useState(false);

  const handleAdd = async () => {
    try {
      await createBinding(workspaceId, skillId, {
        project_id: projectId.trim() ? projectId.trim() : null,
        role: role.trim() ? role.trim() : null,
        embed_full: embedFull,
      });
      onReload();
    } catch {
      // ignore
    }
  };

  const handleDelete = async (bindingId: string) => {
    try {
      await deleteBinding(workspaceId, skillId, bindingId);
      onReload();
    } catch {
      // ignore
    }
  };

  return { projectId, setProjectId, role, setRole, embedFull, setEmbedFull, handleAdd, handleDelete };
}

export function SkillDetailDialog({ workspaceId, skill, onClose, onReload }: SkillDetailDialogProps) {
  const actions = useBindingActions(workspaceId, skill.id, onReload);

  return (
    <Modal open onClose={onClose} title={skill.name} size="lg">
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-16)" }}>
        {skill.description && <p style={{ margin: 0, fontSize: "var(--fs-12)", color: "var(--mu)" }}>{skill.description}</p>}
        <SkillMetaInfo skill={skill} />
        <BindingSection
          projectId={actions.projectId}
          role={actions.role}
          embedFull={actions.embedFull}
          onProjectChange={actions.setProjectId}
          onRoleChange={actions.setRole}
          onEmbedFullChange={actions.setEmbedFull}
          onAdd={() => void actions.handleAdd()}
          bindings={skill.bindings}
          onDeleteBinding={(id) => void actions.handleDelete(id)}
        />
        <ContentPreview content={skill.content} />
      </div>
    </Modal>
  );
}

function SkillMetaInfo({ skill }: { skill: SkillDetail }) {
  return (
    <div style={{ fontSize: "var(--fs-12)", display: "flex", flexDirection: "column", gap: 4 }}>
      <div>
        <span style={{ fontWeight: 500, color: "var(--fg)" }}>{t("skills_hash_label")}: </span>
        <code style={{ backgroundColor: "var(--sf2)", padding: "2px 4px", borderRadius: "var(--radius-4)", fontFamily: "var(--font-mono)", color: "var(--fg)" }}>{skill.content_hash}</code>
      </div>
      {skill.has_scripts && (
        <div style={{ borderRadius: "var(--radius-4)", backgroundColor: "var(--orb)", padding: "var(--sp-8)", color: "var(--or)", border: "1px solid var(--or)" }}>
          <span style={{ fontWeight: 600 }}>{t("skills_has_scripts_badge")}: </span>
          {skill.script_files.join(", ")}
        </div>
      )}
    </div>
  );
}

interface BindingSectionProps {
  projectId: string;
  role: string;
  embedFull: boolean;
  onProjectChange: (v: string) => void;
  onRoleChange: (v: string) => void;
  onEmbedFullChange: (v: boolean) => void;
  onAdd: () => void;
  bindings: SkillDetail["bindings"];
  onDeleteBinding: (id: string) => void;
}

function BindingSection(props: BindingSectionProps) {
  return (
    <div>
      <h4 style={{ margin: "0 0 var(--sp-8)", fontSize: "var(--fs-14)", fontWeight: 600, color: "var(--fg)" }}>{t("skill_binding_title")}</h4>
      <BindingInputBar
        projectId={props.projectId}
        role={props.role}
        embedFull={props.embedFull}
        onProjectChange={props.onProjectChange}
        onRoleChange={props.onRoleChange}
        onEmbedFullChange={props.onEmbedFullChange}
        onAdd={props.onAdd}
      />
      <BindingList bindings={props.bindings} onDeleteBinding={props.onDeleteBinding} />
    </div>
  );
}

interface BindingInputBarProps {
  projectId: string;
  role: string;
  embedFull: boolean;
  onProjectChange: (v: string) => void;
  onRoleChange: (v: string) => void;
  onEmbedFullChange: (v: boolean) => void;
  onAdd: () => void;
}

const roleOptions = [
  { value: "worker", label: t("role_worker") },
  { value: "lead", label: t("role_lead") },
  { value: "test", label: t("role_test") },
  { value: "doc", label: t("role_doc") },
];

function BindingInputBar(props: BindingInputBarProps) {
  return (
    <div style={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: "var(--sp-8)", fontSize: "var(--fs-12)" }}>
      <Input
        type="text"
        placeholder={t("skill_project_id_placeholder")}
        value={props.projectId}
        onChange={(e) => props.onProjectChange(e.target.value)}
        style={{ width: "auto" }}
      />
      <Select
        value={props.role}
        onChange={(e) => props.onRoleChange(e.target.value)}
        options={roleOptions}
        style={{ width: "auto" }}
      />
      <label style={{ display: "flex", alignItems: "center", gap: 4, color: "var(--fg)" }}>
        <Checkbox checked={props.embedFull} onChange={(e) => props.onEmbedFullChange(e.target.checked)} />
        <span>{t("skills_embed_full_label")}</span>
      </label>
      <Button
        type="button"
        size="sm"
        variant="primary"
        onClick={props.onAdd}
      >
        {t("skills_add_binding")}
      </Button>
    </div>
  );
}

function BindingList({
  bindings,
  onDeleteBinding,
}: {
  bindings: SkillDetail["bindings"];
  onDeleteBinding: (id: string) => void;
}) {
  return (
    <div style={{ marginTop: "var(--sp-8)", borderRadius: "var(--radius-6)", border: "1px solid var(--ln)", fontSize: "var(--fs-12)" }}>
      {bindings.map((b) => (
        <div key={b.id} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "var(--sp-8)", borderBottom: "1px solid var(--ln)" }}>
          <span style={{ color: "var(--fg)" }}>
            {t("skill_project_prefix")} {b.project_id || t("filter_all")} {t("skill_role_prefix")} {b.role || t("filter_all")} |{" "}
            {b.embed_full ? t("skills_embed_full_label") : t("skills_embed_default_label")}
          </span>
          <Button type="button" variant="ghost" size="sm" onClick={() => onDeleteBinding(b.id)} style={{ color: "var(--rd)" }}>
            {t("skills_remove_binding")}
          </Button>
        </div>
      ))}
    </div>
  );
}

function ContentPreview({ content }: { content: string }) {
  return (
    <div>
      <h4 style={{ margin: "0 0 4px", fontSize: "var(--fs-14)", fontWeight: 600, color: "var(--fg)" }}>{t("skills_content_markdown")}</h4>
      <pre style={{ margin: 0, maxHeight: 192, overflowY: "auto", borderRadius: "var(--radius-6)", backgroundColor: "var(--sf2)", padding: "var(--sp-12)", fontSize: "var(--fs-12)", fontFamily: "var(--font-mono)", color: "var(--fg)" }}>{content}</pre>
    </div>
  );
}

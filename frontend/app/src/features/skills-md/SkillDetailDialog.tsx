import { X } from "lucide-react";
import { useState } from "react";
import { t } from "../../core/i18n";
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
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
      <div className="max-h-[85vh] w-full max-w-2xl overflow-y-auto rounded bg-white p-6 shadow-lg space-y-4">
        <DialogHeader name={skill.name} desc={skill.description} onClose={onClose} />
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
    </div>
  );
}

function DialogHeader({ name, desc, onClose }: { name: string; desc: string; onClose: () => void }) {
  return (
    <div className="flex items-start justify-between border-b pb-3">
      <div>
        <h3 className="text-lg font-bold">{name}</h3>
        <p className="text-xs text-gray-500">{desc}</p>
      </div>
      <button type="button" onClick={onClose} className="text-gray-400 hover:text-gray-600">
        <X className="h-4 w-4" />
      </button>
    </div>
  );
}

function SkillMetaInfo({ skill }: { skill: SkillDetail }) {
  return (
    <div className="text-xs space-y-1">
      <div>
        <span className="font-medium">{t("skills_hash_label")}: </span>
        <code className="bg-gray-100 p-1">{skill.content_hash}</code>
      </div>
      {skill.has_scripts && (
        <div className="rounded bg-orange-50 p-2 text-orange-800">
          <span className="font-semibold">{t("skills_has_scripts_badge")}: </span>
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
      <h4 className="text-sm font-semibold mb-2">Gắn kết dự án & vai trò</h4>
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

function BindingInputBar(props: BindingInputBarProps) {
  return (
    <div className="flex flex-wrap items-center gap-2 text-xs">
      <input
        type="text"
        placeholder="Project ID (để trống = toàn bộ)"
        value={props.projectId}
        onChange={(e) => props.onProjectChange(e.target.value)}
        className="rounded border p-1"
      />
      <select value={props.role} onChange={(e) => props.onRoleChange(e.target.value)} className="rounded border p-1">
        <option value="worker">Worker</option>
        <option value="lead">Lead</option>
        <option value="test">Test</option>
        <option value="doc">Doc</option>
      </select>
      <label className="flex items-center space-x-1">
        <input type="checkbox" checked={props.embedFull} onChange={(e) => props.onEmbedFullChange(e.target.checked)} />
        <span>{t("skills_embed_full_label")}</span>
      </label>
      <button
        type="button"
        onClick={props.onAdd}
        className="rounded bg-blue-600 px-3 py-1 font-medium text-white hover:bg-blue-700"
      >
        {t("skills_add_binding")}
      </button>
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
    <div className="mt-2 divide-y rounded border text-xs">
      {bindings.map((b) => (
        <div key={b.id} className="flex items-center justify-between p-2">
          <span>
            Dự án: {b.project_id || "Tất cả"} | Vai trò: {b.role || "Tất cả"} |{" "}
            {b.embed_full ? t("skills_embed_full_label") : t("skills_embed_default_label")}
          </span>
          <button type="button" onClick={() => onDeleteBinding(b.id)} className="text-red-600 hover:underline">
            {t("skills_remove_binding")}
          </button>
        </div>
      ))}
    </div>
  );
}

function ContentPreview({ content }: { content: string }) {
  return (
    <div>
      <h4 className="text-sm font-semibold mb-1">{t("skills_content_markdown")}</h4>
      <pre className="max-h-48 overflow-y-auto rounded bg-gray-50 p-3 text-xs">{content}</pre>
    </div>
  );
}

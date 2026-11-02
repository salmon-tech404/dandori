import { useState } from "react";
import { t } from "../../core/i18n";
import type { CreateMemoryPayload, MemoryCategory } from "./types";

interface CreateModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (payload: CreateMemoryPayload) => Promise<void>;
  projectId?: string;
}

export function CreateMemoryModal({ isOpen, onClose, onSubmit, projectId }: CreateModalProps) {
  const [category, setCategory] = useState<MemoryCategory>("fact");
  const [content, setContent] = useState("");
  const [saving, setSaving] = useState(false);

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!content.trim() || saving) return;
    setSaving(true);
    try {
      await onSubmit({
        project_id: projectId || null,
        category,
        content: content.trim(),
      });
      setContent("");
      onClose();
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
      <div className="bg-slate-900 border border-slate-800 rounded-xl max-w-lg w-full p-6 shadow-2xl">
        <h2 className="text-lg font-semibold text-white mb-4">
          {t("shared_memory_modal_title")}
        </h2>
        <form onSubmit={handleSubmit} className="space-y-4">
          <CategoryField value={category} onChange={setCategory} />
          <ContentField value={content} onChange={setContent} />
          <p className="text-xs text-slate-500 italic">
            {t("shared_memory_notice_b10")}
          </p>
          <ModalActions saving={saving} onClose={onClose} />
        </form>
      </div>
    </div>
  );
}

function CategoryField({
  value,
  onChange,
}: {
  value: MemoryCategory;
  onChange: (c: MemoryCategory) => void;
}) {
  return (
    <div>
      <label className="block text-xs font-medium text-slate-400 mb-1">
        {t("shared_memory_col_category")}
      </label>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value as MemoryCategory)}
        className="w-full bg-slate-800 border border-slate-700 text-slate-200 text-sm rounded px-3 py-2"
      >
        <option value="fact">{t("shared_memory_category_fact")}</option>
        <option value="decision">{t("shared_memory_category_decision")}</option>
        <option value="convention">{t("shared_memory_category_convention")}</option>
        <option value="lesson">{t("shared_memory_category_lesson")}</option>
      </select>
    </div>
  );
}

function ContentField({
  value,
  onChange,
}: {
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <div>
      <label className="block text-xs font-medium text-slate-400 mb-1">
        {t("shared_memory_col_content")}
      </label>
      <textarea
        value={value}
        onChange={(e) => onChange(e.target.value)}
        rows={4}
        placeholder={t("shared_memory_content_placeholder")}
        className="w-full bg-slate-800 border border-slate-700 text-slate-200 text-sm rounded p-3 focus:outline-none focus:border-indigo-500 resize-none"
        required
      />
    </div>
  );
}

function ModalActions({ saving, onClose }: { saving: boolean; onClose: () => void }) {
  return (
    <div className="flex justify-end gap-3 pt-2">
      <button
        type="button"
        onClick={onClose}
        disabled={saving}
        className="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 text-sm rounded transition"
      >
        {t("shared_memory_cancel")}
      </button>
      <button
        type="submit"
        disabled={saving}
        className="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white text-sm font-medium rounded transition disabled:opacity-50"
      >
        {saving ? "..." : t("shared_memory_save")}
      </button>
    </div>
  );
}

import { useState } from "react";
import { t } from "../../core/i18n";
import { Modal } from "../../ui/components/Modal";
import { Button } from "../../ui/components/Button";
import { Select } from "../../ui/components/Select";
import { Textarea } from "../../ui/components/Textarea";
import type { CreateMemoryPayload, MemoryCategory } from "./types";

interface CreateModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (payload: CreateMemoryPayload) => Promise<void>;
  projectId?: string;
}

const categoryOptions = [
  { value: "fact", label: t("shared_memory_category_fact") },
  { value: "decision", label: t("shared_memory_category_decision") },
  { value: "convention", label: t("shared_memory_category_convention") },
  { value: "lesson", label: t("shared_memory_category_lesson") },
];

function CategoryField({
  value,
  onChange,
}: {
  value: MemoryCategory;
  onChange: (c: MemoryCategory) => void;
}) {
  return (
    <div>
      <label style={{ display: "block", fontSize: "var(--fs-12)", fontWeight: 500, color: "var(--fg)", marginBottom: 4 }}>
        {t("shared_memory_col_category")}
      </label>
      <Select
        value={value}
        onChange={(e) => onChange(e.target.value as MemoryCategory)}
        options={categoryOptions}
      />
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
      <label style={{ display: "block", fontSize: "var(--fs-12)", fontWeight: 500, color: "var(--fg)", marginBottom: 4 }}>
        {t("shared_memory_col_content")}
      </label>
      <Textarea
        value={value}
        onChange={(e) => onChange(e.target.value)}
        rows={4}
        placeholder={t("shared_memory_content_placeholder")}
        required
      />
    </div>
  );
}

function ModalActions({ saving, onClose }: { saving: boolean; onClose: () => void }) {
  return (
    <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--sp-12)", paddingTop: "var(--sp-8)" }}>
      <Button
        type="button"
        variant="secondary"
        onClick={onClose}
        disabled={saving}
      >
        {t("shared_memory_cancel")}
      </Button>
      <Button
        type="submit"
        variant="primary"
        disabled={saving}
      >
        {saving ? "..." : t("shared_memory_save")}
      </Button>
    </div>
  );
}

export function CreateMemoryModal({ isOpen, onClose, onSubmit, projectId }: CreateModalProps) {
  const [category, setCategory] = useState<MemoryCategory>("fact");
  const [content, setContent] = useState("");
  const [saving, setSaving] = useState(false);

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
    <Modal open={isOpen} onClose={onClose} title={t("shared_memory_modal_title")} size="md">
      <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: "var(--sp-16)" }}>
        <CategoryField value={category} onChange={setCategory} />
        <ContentField value={content} onChange={setContent} />
        <p style={{ margin: 0, fontSize: "var(--fs-12)", color: "var(--mu)", fontStyle: "italic" }}>
          {t("shared_memory_notice_b10")}
        </p>
        <ModalActions saving={saving} onClose={onClose} />
      </form>
    </Modal>
  );
}

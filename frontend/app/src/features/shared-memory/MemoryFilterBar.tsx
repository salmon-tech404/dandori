import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { Select } from "../../ui/components/Select";
import type { MemoryFilters } from "./types";

interface FilterBarProps {
  filters: MemoryFilters;
  onFilterChange: (filters: MemoryFilters) => void;
  onOpenCreate: () => void;
  onBatchApprove: () => void;
  onBatchReject: () => void;
  selectedCount: number;
}

const statusOptions = [
  { value: "", label: t("shared_memory_status_all") },
  { value: "pending", label: t("shared_memory_status_pending") },
  { value: "approved", label: t("shared_memory_status_approved") },
  { value: "rejected", label: t("shared_memory_status_rejected") },
];

const categoryOptions = [
  { value: "", label: t("shared_memory_category_all") },
  { value: "fact", label: t("shared_memory_category_fact") },
  { value: "decision", label: t("shared_memory_category_decision") },
  { value: "convention", label: t("shared_memory_category_convention") },
  { value: "lesson", label: t("shared_memory_category_lesson") },
];

function StatusSelect({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  return (
    <Select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      options={statusOptions}
    />
  );
}

function CategorySelect({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  return (
    <Select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      options={categoryOptions}
    />
  );
}

function BatchActions({
  selectedCount,
  onBatchApprove,
  onBatchReject,
}: {
  selectedCount: number;
  onBatchApprove: () => void;
  onBatchReject: () => void;
}) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
      <span style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>
        ({selectedCount})
      </span>
      <Button
        type="button"
        variant="secondary"
        size="sm"
        onClick={onBatchApprove}
      >
        {t("shared_memory_btn_batch_approve")}
      </Button>
      <Button
        type="button"
        variant="danger"
        size="sm"
        onClick={onBatchReject}
      >
        {t("shared_memory_btn_batch_reject")}
      </Button>
    </div>
  );
}

export function MemoryFilterBar({
  filters,
  onFilterChange,
  onOpenCreate,
  onBatchApprove,
  onBatchReject,
  selectedCount,
}: FilterBarProps) {
  return (
    <div style={{ display: "flex", flexWrap: "wrap", alignItems: "center", justifyContent: "space-between", gap: "var(--sp-16)", backgroundColor: "var(--sf)", padding: "var(--sp-16)", borderRadius: "var(--radius-8)", border: "1px solid var(--ln)" }}>
      <div style={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: "var(--sp-12)" }}>
        <StatusSelect
          value={filters.status || ""}
          onChange={(s) => onFilterChange({ ...filters, status: s || undefined })}
        />
        <CategorySelect
          value={filters.category || ""}
          onChange={(c) => onFilterChange({ ...filters, category: c || undefined })}
        />
        {selectedCount > 0 && (
          <BatchActions
            selectedCount={selectedCount}
            onBatchApprove={onBatchApprove}
            onBatchReject={onBatchReject}
          />
        )}
      </div>
      <Button
        type="button"
        variant="primary"
        onClick={onOpenCreate}
      >
        {t("shared_memory_btn_create")}
      </Button>
    </div>
  );
}

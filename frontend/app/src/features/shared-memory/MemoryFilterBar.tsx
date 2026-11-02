import { t } from "../../core/i18n";
import type { MemoryFilters } from "./types";

interface FilterBarProps {
  filters: MemoryFilters;
  onFilterChange: (filters: MemoryFilters) => void;
  onOpenCreate: () => void;
  onBatchApprove: () => void;
  onBatchReject: () => void;
  selectedCount: number;
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
    <div className="flex flex-wrap items-center justify-between gap-4 bg-slate-900/50 p-4 rounded-lg border border-slate-800">
      <div className="flex flex-wrap items-center gap-3">
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
      <button
        onClick={onOpenCreate}
        className="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded text-sm font-medium transition"
      >
        {t("shared_memory_btn_create")}
      </button>
    </div>
  );
}

function StatusSelect({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="bg-slate-800 border border-slate-700 text-slate-200 text-sm rounded px-3 py-1.5 focus:outline-none focus:border-indigo-500"
    >
      <option value="">{t("shared_memory_status_all")}</option>
      <option value="pending">{t("shared_memory_status_pending")}</option>
      <option value="approved">{t("shared_memory_status_approved")}</option>
      <option value="rejected">{t("shared_memory_status_rejected")}</option>
    </select>
  );
}

function CategorySelect({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="bg-slate-800 border border-slate-700 text-slate-200 text-sm rounded px-3 py-1.5 focus:outline-none focus:border-indigo-500"
    >
      <option value="">{t("shared_memory_category_all")}</option>
      <option value="fact">{t("shared_memory_category_fact")}</option>
      <option value="decision">{t("shared_memory_category_decision")}</option>
      <option value="convention">{t("shared_memory_category_convention")}</option>
      <option value="lesson">{t("shared_memory_category_lesson")}</option>
    </select>
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
    <div className="flex items-center gap-2 pl-3 border-l border-slate-700">
      <span className="text-xs text-slate-400">({selectedCount})</span>
      <button
        onClick={onBatchApprove}
        className="px-2.5 py-1 bg-emerald-600/80 hover:bg-emerald-600 text-white text-xs rounded transition"
      >
        {t("shared_memory_btn_batch_approve")}
      </button>
      <button
        onClick={onBatchReject}
        className="px-2.5 py-1 bg-rose-600/80 hover:bg-rose-600 text-white text-xs rounded transition"
      >
        {t("shared_memory_btn_batch_reject")}
      </button>
    </div>
  );
}

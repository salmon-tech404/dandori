import { t } from "../../core/i18n";
import type { MemoryEntry } from "./types";

interface TableProps {
  memories: MemoryEntry[];
  selectedIds: string[];
  onToggleSelect: (id: string) => void;
  onSelectAll: (ids: string[]) => void;
  onApprove: (id: string) => void;
  onReject: (id: string) => void;
  onDelete: (id: string) => void;
}

export function MemoryTable({
  memories,
  selectedIds,
  onToggleSelect,
  onSelectAll,
  onApprove,
  onReject,
  onDelete,
}: TableProps) {
  if (memories.length === 0) {
    return (
      <div className="text-center py-12 text-slate-500 text-sm">
        {t("shared_memory_empty")}
      </div>
    );
  }

  const allSelected = memories.length > 0 && selectedIds.length === memories.length;

  return (
    <div className="overflow-x-auto rounded-lg border border-slate-800">
      <table className="w-full text-left text-sm text-slate-300">
        <TableHeader
          allSelected={allSelected}
          onSelectAll={() => onSelectAll(allSelected ? [] : memories.map((m) => m.id))}
        />
        <TableBody
          memories={memories}
          selectedIds={selectedIds}
          onToggleSelect={onToggleSelect}
          onApprove={onApprove}
          onReject={onReject}
          onDelete={onDelete}
        />
      </table>
    </div>
  );
}

function TableHeader({
  allSelected,
  onSelectAll,
}: {
  allSelected: boolean;
  onSelectAll: () => void;
}) {
  return (
    <thead className="bg-slate-800/80 text-xs uppercase text-slate-400">
      <tr>
        <th className="p-3 w-10">
          <input
            type="checkbox"
            checked={allSelected}
            onChange={onSelectAll}
            className="rounded border-slate-700 bg-slate-800 text-indigo-600 focus:ring-0"
          />
        </th>
        <th className="p-3 w-28">{t("shared_memory_col_category")}</th>
        <th className="p-3">{t("shared_memory_col_content")}</th>
        <th className="p-3 w-40">{t("shared_memory_col_source")}</th>
        <th className="p-3 w-28">{t("shared_memory_col_status")}</th>
        <th className="p-3 w-44 text-right">{t("shared_memory_col_actions")}</th>
      </tr>
    </thead>
  );
}

function TableBody({
  memories,
  selectedIds,
  onToggleSelect,
  onApprove,
  onReject,
  onDelete,
}: {
  memories: MemoryEntry[];
  selectedIds: string[];
  onToggleSelect: (id: string) => void;
  onApprove: (id: string) => void;
  onReject: (id: string) => void;
  onDelete: (id: string) => void;
}) {
  return (
    <tbody className="divide-y divide-slate-800">
      {memories.map((entry) => (
        <Row
          key={entry.id}
          entry={entry}
          isSelected={selectedIds.includes(entry.id)}
          onToggle={() => onToggleSelect(entry.id)}
          onApprove={() => onApprove(entry.id)}
          onReject={() => onReject(entry.id)}
          onDelete={() => onDelete(entry.id)}
        />
      ))}
    </tbody>
  );
}

function Row({
  entry,
  isSelected,
  onToggle,
  onApprove,
  onReject,
  onDelete,
}: {
  entry: MemoryEntry;
  isSelected: boolean;
  onToggle: () => void;
  onApprove: () => void;
  onReject: () => void;
  onDelete: () => void;
}) {
  return (
    <tr className="hover:bg-slate-800/40 transition">
      <td className="p-3">
        <input
          type="checkbox"
          checked={isSelected}
          onChange={onToggle}
          className="rounded border-slate-700 bg-slate-800 text-indigo-600 focus:ring-0"
        />
      </td>
      <td className="p-3"><CategoryBadge category={entry.category} /></td>
      <td className="p-3 font-mono text-xs whitespace-pre-wrap max-w-md">{entry.content}</td>
      <td className="p-3 text-xs text-slate-400">
        <span className="font-semibold text-slate-300">{entry.source_type}:</span> {entry.source_ref}
      </td>
      <td className="p-3"><StatusBadge status={entry.status} /></td>
      <td className="p-3 text-right">
        <RowActions status={entry.status} onApprove={onApprove} onReject={onReject} onDelete={onDelete} />
      </td>
    </tr>
  );
}

function CategoryBadge({ category }: { category: string }) {
  const colors: Record<string, string> = {
    fact: "bg-blue-500/10 text-blue-400 border-blue-500/20",
    decision: "bg-purple-500/10 text-purple-400 border-purple-500/20",
    convention: "bg-amber-500/10 text-amber-400 border-amber-500/20",
    lesson: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20",
  };
  const color = colors[category.toLowerCase()] || "bg-slate-500/10 text-slate-400 border-slate-500/20";
  return (
    <span className={`inline-block px-2 py-0.5 text-xs rounded border ${color}`}>
      {category.toUpperCase()}
    </span>
  );
}

function StatusBadge({ status }: { status: string }) {
  const badges: Record<string, { label: string; cls: string }> = {
    pending: { label: t("shared_memory_status_pending"), cls: "bg-amber-500/10 text-amber-400 border-amber-500/20" },
    approved: { label: t("shared_memory_status_approved"), cls: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20" },
    rejected: { label: t("shared_memory_status_rejected"), cls: "bg-rose-500/10 text-rose-400 border-rose-500/20" },
  };
  const b = badges[status.toLowerCase()] || { label: status, cls: "bg-slate-500/10 text-slate-400 border-slate-500/20" };
  return (
    <span className={`inline-block px-2 py-0.5 text-xs rounded border ${b.cls}`}>
      {b.label}
    </span>
  );
}

function RowActions({
  status,
  onApprove,
  onReject,
  onDelete,
}: {
  status: string;
  onApprove: () => void;
  onReject: () => void;
  onDelete: () => void;
}) {
  return (
    <div className="flex items-center justify-end gap-2 text-xs">
      {status !== "approved" && (
        <button type="button" onClick={onApprove} className="text-emerald-400 hover:underline font-medium">
          {t("shared_memory_btn_approve")}
        </button>
      )}
      {status !== "rejected" && (
        <button type="button" onClick={onReject} className="text-rose-400 hover:underline font-medium">
          {t("shared_memory_btn_reject")}
        </button>
      )}
      <button type="button" onClick={onDelete} className="text-slate-400 hover:text-red-400 hover:underline">
        {t("shared_memory_btn_delete")}
      </button>
    </div>
  );
}

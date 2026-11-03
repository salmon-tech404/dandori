import { CSSProperties } from "react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { Checkbox } from "../../ui/components/Checkbox";
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
      <div style={{ textAlign: "center", padding: "var(--sp-32) 0", color: "var(--mu)", fontSize: "var(--fs-14)" }}>
        {t("shared_memory_empty")}
      </div>
    );
  }

  const allSelected = memories.length > 0 && selectedIds.length === memories.length;

  return (
    <div style={{ overflowX: "auto", borderRadius: "var(--radius-8)", border: "1px solid var(--ln)", backgroundColor: "var(--sf)" }}>
      <table style={{ width: "100%", textAlign: "left", fontSize: "var(--fs-14)", borderCollapse: "collapse" }}>
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
    <thead style={{ backgroundColor: "var(--sf2)", fontSize: "var(--fs-12)", textTransform: "uppercase", color: "var(--mu)", borderBottom: "1px solid var(--ln)" }}>
      <tr>
        <th style={{ padding: "10px 12px", width: 40 }}>
          <Checkbox
            checked={allSelected}
            onChange={onSelectAll}
          />
        </th>
        <th style={{ padding: "10px 12px", width: 112 }}>{t("shared_memory_col_category")}</th>
        <th style={{ padding: "10px 12px" }}>{t("shared_memory_col_content")}</th>
        <th style={{ padding: "10px 12px", width: 160 }}>{t("shared_memory_col_source")}</th>
        <th style={{ padding: "10px 12px", width: 112 }}>{t("shared_memory_col_status")}</th>
        <th style={{ padding: "10px 12px", width: 176, textAlign: "right" }}>{t("shared_memory_col_actions")}</th>
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
    <tbody>
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
    <tr style={{ borderBottom: "1px solid var(--ln)", backgroundColor: isSelected ? "var(--blb)" : "transparent" }}>
      <td style={{ padding: "10px 12px" }}>
        <Checkbox
          checked={isSelected}
          onChange={onToggle}
        />
      </td>
      <td style={{ padding: "10px 12px" }}><CategoryBadge category={entry.category} /></td>
      <td style={{ padding: "10px 12px", fontFamily: "var(--font-mono)", fontSize: "var(--fs-12)", whiteSpace: "pre-wrap", maxWidth: 448, color: "var(--fg)" }}>{entry.content}</td>
      <td style={{ padding: "10px 12px", fontSize: "var(--fs-12)", color: "var(--mu)" }}>
        <span style={{ fontWeight: 600, color: "var(--fg)" }}>{entry.source_type}:</span> {entry.source_ref}
      </td>
      <td style={{ padding: "10px 12px" }}><StatusBadge status={entry.status} /></td>
      <td style={{ padding: "10px 12px", textAlign: "right" }}>
        <RowActions status={entry.status} onApprove={onApprove} onReject={onReject} onDelete={onDelete} />
      </td>
    </tr>
  );
}

function CategoryBadge({ category }: { category: string }) {
  const styles: Record<string, CSSProperties> = {
    fact: { backgroundColor: "var(--blb)", color: "var(--bl)", border: "1px solid var(--bl)" },
    decision: { backgroundColor: "var(--tlb)", color: "var(--tl)", border: "1px solid var(--tl)" },
    convention: { backgroundColor: "var(--orb)", color: "var(--or)", border: "1px solid var(--or)" },
    lesson: { backgroundColor: "var(--grb)", color: "var(--gr)", border: "1px solid var(--gr)" },
  };
  const style = styles[category.toLowerCase()] || { backgroundColor: "var(--sf2)", color: "var(--mu)", border: "1px solid var(--ln)" };
  return (
    <span style={{ display: "inline-block", padding: "2px 8px", fontSize: "var(--fs-12)", borderRadius: "var(--radius-4)", ...style }}>
      {category.toUpperCase()}
    </span>
  );
}

function StatusBadge({ status }: { status: string }) {
  const badges: Record<string, { label: string; style: CSSProperties }> = {
    pending: { label: t("shared_memory_status_pending"), style: { backgroundColor: "var(--orb)", color: "var(--or)", border: "1px solid var(--or)" } },
    approved: { label: t("shared_memory_status_approved"), style: { backgroundColor: "var(--grb)", color: "var(--gr)", border: "1px solid var(--gr)" } },
    rejected: { label: t("shared_memory_status_rejected"), style: { backgroundColor: "var(--rdb)", color: "var(--rd)", border: "1px solid var(--rd)" } },
  };
  const b = badges[status.toLowerCase()] || { label: status, style: { backgroundColor: "var(--sf2)", color: "var(--mu)", border: "1px solid var(--ln)" } };
  return (
    <span style={{ display: "inline-block", padding: "2px 8px", fontSize: "var(--fs-12)", borderRadius: "var(--radius-4)", ...b.style }}>
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
    <div style={{ display: "flex", alignItems: "center", justifyContent: "flex-end", gap: "var(--sp-8)", fontSize: "var(--fs-12)" }}>
      {status !== "approved" && (
        <Button type="button" variant="ghost" size="sm" onClick={onApprove} style={{ color: "var(--gr)", fontWeight: 500 }}>
          {t("shared_memory_btn_approve")}
        </Button>
      )}
      {status !== "rejected" && (
        <Button type="button" variant="ghost" size="sm" onClick={onReject} style={{ color: "var(--rd)", fontWeight: 500 }}>
          {t("shared_memory_btn_reject")}
        </Button>
      )}
      <Button type="button" variant="ghost" size="sm" onClick={onDelete} style={{ color: "var(--mu)" }}>
        {t("shared_memory_btn_delete")}
      </Button>
    </div>
  );
}

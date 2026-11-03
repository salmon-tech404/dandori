import { useState } from "react";
import { t } from "../../core/i18n";
import { CreateMemoryModal } from "./CreateMemoryModal";
import { MemoryFilterBar } from "./MemoryFilterBar";
import { MemoryTable } from "./MemoryTable";
import type { MemoryEntry, MemoryFilters } from "./types";
import { useMemoryActions, useMemoryData, useMemorySelection } from "./useSharedMemory";

interface PanelProps {
  workspaceId: string;
  projectId?: string;
}

export function SharedMemoryPanel({ workspaceId, projectId }: PanelProps) {
  const [filters, setFilters] = useState<MemoryFilters>({ projectId });
  const [isModalOpen, setIsModalOpen] = useState(false);

  const { memories, loading, error, loadMemories } = useMemoryData(workspaceId, filters);
  const selection = useMemorySelection();
  const actions = useMemoryActions(workspaceId, loadMemories, selection.clearSelection);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-24)", padding: "var(--sp-24)" }}>
      <PanelHeader />
      {error && <div style={{ padding: "var(--sp-12)", backgroundColor: "var(--rdb)", border: "1px solid var(--rd)", color: "var(--rd)", fontSize: "var(--fs-14)", borderRadius: "var(--radius-6)" }}>{error}</div>}
      <MemoryFilterBar
        filters={filters}
        onFilterChange={setFilters}
        onOpenCreate={() => setIsModalOpen(true)}
        onBatchApprove={() => actions.batchUpdate(selection.selectedIds, "approved")}
        onBatchReject={() => actions.batchUpdate(selection.selectedIds, "rejected")}
        selectedCount={selection.selectedIds.length}
      />
      <TableSection
        loading={loading}
        memories={memories}
        selectedIds={selection.selectedIds}
        toggleSelect={selection.toggleSelect}
        selectAll={selection.selectAll}
        onApprove={(id) => actions.updateStatus(id, "approved")}
        onReject={(id) => actions.updateStatus(id, "rejected")}
        onDelete={actions.deleteItem}
      />
      <CreateMemoryModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        onSubmit={actions.createItem}
        projectId={projectId}
      />
    </div>
  );
}

function PanelHeader() {
  return (
    <div>
      <h1 style={{ margin: 0, fontSize: "var(--fs-18)", fontWeight: 700, color: "var(--fg)" }}>{t("shared_memory_title")}</h1>
      <p style={{ margin: "4px 0 0", fontSize: "var(--fs-14)", color: "var(--mu)" }}>{t("shared_memory_desc")}</p>
    </div>
  );
}

function TableSection({
  loading,
  memories,
  selectedIds,
  toggleSelect,
  selectAll,
  onApprove,
  onReject,
  onDelete,
}: {
  loading: boolean;
  memories: MemoryEntry[];
  selectedIds: string[];
  toggleSelect: (id: string) => void;
  selectAll: (ids: string[]) => void;
  onApprove: (id: string) => void;
  onReject: (id: string) => void;
  onDelete: (id: string) => void;
}) {
  if (loading) {
    return <div style={{ textAlign: "center", padding: "var(--sp-32) 0", color: "var(--mu)" }}>{t("loading")}</div>;
  }

  return (
    <MemoryTable
      memories={memories}
      selectedIds={selectedIds}
      onToggleSelect={toggleSelect}
      onSelectAll={selectAll}
      onApprove={onApprove}
      onReject={onReject}
      onDelete={onDelete}
    />
  );
}

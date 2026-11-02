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
    <div className="space-y-6 p-6">
      <PanelHeader />
      {error && <div className="p-3 bg-rose-500/10 border border-rose-500/20 text-rose-400 text-sm rounded">{error}</div>}
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
      <h1 className="text-xl font-bold text-white">{t("shared_memory_title")}</h1>
      <p className="text-sm text-slate-400 mt-1">{t("shared_memory_desc")}</p>
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
    return <div className="text-center py-12 text-slate-500 text-sm">...</div>;
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

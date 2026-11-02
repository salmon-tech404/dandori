import { t } from '../../../core/i18n';
import { exportRun } from '../api/runHistoryApi';
import { useNoteModal, useReplayModal, useRunList } from '../hooks/useRunHistory';
import { LogReplayModal } from './LogReplayModal';
import { NoteEditorModal } from './NoteEditorModal';
import { RunHistoryFilter } from './RunHistoryFilter';
import { RunHistoryTable } from './RunHistoryTable';

const handleExport = async (runId: string, format: 'markdown' | 'json') => {
  try {
    const content = await exportRun(runId, format);
    const mime = format === 'json' ? 'application/json' : 'text/markdown';
    const ext = format === 'json' ? 'json' : 'md';
    const blob = new Blob([content], { type: mime });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `run-${runId}.${ext}`;
    a.click();
    URL.revokeObjectURL(url);
  } catch (err) {
    alert(err instanceof Error ? err.message : 'Lỗi xuất dữ liệu');
  }
};

const PanelHeader = ({ list }: { list: ReturnType<typeof useRunList> }) => (
  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
    <div>
      <h3 style={{ margin: 0, fontSize: '16px', fontWeight: 600 }}>{t('run_history_title')}</h3>
      <p style={{ margin: '4px 0 0 0', fontSize: '13px', color: 'var(--mu)' }}>{t('run_history_desc')}</p>
    </div>
    <RunHistoryFilter
      statusFilter={list.statusFilter}
      agentFilter={list.agentFilter}
      onStatusChange={list.setStatusFilter}
      onAgentChange={list.setAgentFilter}
    />
  </div>
);

const PanelContent = ({
  list,
  onReplay,
  onNote,
}: {
  list: ReturnType<typeof useRunList>;
  onReplay: (id: string) => void;
  onNote: (id: string) => void;
}) => {
  if (list.loading) return <div style={{ textAlign: 'center', padding: '32px', color: 'var(--mu)' }}>{t('run_history_loading')}</div>;
  if (list.error) return <div style={{ padding: '16px', backgroundColor: 'var(--rdb)', border: '1px solid var(--rd)', borderRadius: '8px', color: 'var(--rd)' }}>{list.error}</div>;
  if (list.runs.length === 0) return <div style={{ textAlign: 'center', padding: '40px 16px', border: '1px dashed var(--ln)', borderRadius: '8px', color: 'var(--mu)' }}>{t('run_history_empty')}</div>;
  return <RunHistoryTable runs={list.runs} onOpenReplay={onReplay} onOpenNote={onNote} onExport={handleExport} />;
};

export const RunHistoryPanel = ({ context }: { context?: { taskId?: string } }) => {
  const list = useRunList(context?.taskId);
  const replay = useReplayModal();
  const note = useNoteModal(list.reload);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', padding: '16px' }}>
      <PanelHeader list={list} />
      <PanelContent list={list} onReplay={replay.openReplay} onNote={note.openNoteEditor} />
      {replay.replayRunId && (
        <LogReplayModal runId={replay.replayRunId} events={replay.events} loading={replay.loadingEvents} onClose={() => replay.setReplayRunId(null)} />
      )}
      {note.editingNoteRunId && (
        <NoteEditorModal
          noteText={note.noteText}
          tagsText={note.tagsText}
          saving={note.savingNote}
          statusMessage={note.noteStatusMessage}
          onNoteChange={note.setNoteText}
          onTagsChange={note.setTagsText}
          onSave={note.persistNote}
          onClose={() => note.setEditingNoteRunId(null)}
        />
      )}
    </div>
  );
};

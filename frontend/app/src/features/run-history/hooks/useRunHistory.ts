import { useEffect, useState } from 'react';
import { t } from '../../../core/i18n';
import { fetchNote, fetchRunEvents, fetchRuns, saveNote } from '../api/runHistoryApi';
import type { RunEventItem, RunSummary } from '../types';

export const useRunList = (taskId?: string) => {
  const [runs, setRuns] = useState<RunSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [statusFilter, setStatusFilter] = useState('');
  const [agentFilter, setAgentFilter] = useState('');

  const loadData = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await fetchRuns({ task_id: taskId, status: statusFilter || undefined, agent_provider: agentFilter || undefined });
      setRuns(data.runs || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : t('run_history_error'));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { void loadData(); }, [taskId, statusFilter, agentFilter]);

  return { runs, loading, error, statusFilter, agentFilter, setStatusFilter, setAgentFilter, reload: loadData };
};

export const useReplayModal = () => {
  const [replayRunId, setReplayRunId] = useState<string | null>(null);
  const [events, setEvents] = useState<RunEventItem[]>([]);
  const [loadingEvents, setLoadingEvents] = useState(false);

  const openReplay = async (runId: string) => {
    setReplayRunId(runId);
    setLoadingEvents(true);
    try {
      const data = await fetchRunEvents(runId);
      setEvents(data.events || []);
    } catch {
      setEvents([]);
    } finally {
      setLoadingEvents(false);
    }
  };

  return { replayRunId, events, loadingEvents, setReplayRunId, openReplay };
};

export const useNoteModal = (onSaved: () => void) => {
  const [editingNoteRunId, setEditingNoteRunId] = useState<string | null>(null);
  const [noteText, setNoteText] = useState('');
  const [tagsText, setTagsText] = useState('');
  const [savingNote, setSavingNote] = useState(false);
  const [noteStatusMessage, setNoteStatusMessage] = useState<string | null>(null);

  const openNoteEditor = async (runId: string) => {
    setEditingNoteRunId(runId);
    setNoteStatusMessage(null);
    try {
      const data = await fetchNote(runId);
      setNoteText(data.note || '');
      setTagsText((data.tags || []).join(', '));
    } catch {
      setNoteText('');
      setTagsText('');
    }
  };

  const persistNote = async () => {
    if (!editingNoteRunId) return;
    setSavingNote(true);
    setNoteStatusMessage(null);
    try {
      const tags = tagsText.split(',').map((s) => s.trim()).filter(Boolean);
      await saveNote(editingNoteRunId, noteText, tags);
      setNoteStatusMessage(t('run_history_note_saved'));
      onSaved();
      setTimeout(() => setEditingNoteRunId(null), 800);
    } catch (err) {
      setNoteStatusMessage(err instanceof Error ? err.message : 'Lỗi lưu ghi chú');
    } finally {
      setSavingNote(false);
    }
  };

  return { editingNoteRunId, noteText, tagsText, savingNote, noteStatusMessage, setNoteText, setTagsText, setEditingNoteRunId, openNoteEditor, persistNote };
};

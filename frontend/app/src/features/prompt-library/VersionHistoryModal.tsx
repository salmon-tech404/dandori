import { RotateCcw } from 'lucide-react';
import React, { useEffect, useState } from 'react';
import { t } from '../../core/i18n';
import { Modal } from '../../ui/components/Modal';
import { Button } from '../../ui/components/Button';
import { PromptSummary, PromptVersion } from './types';
import { fetchPromptVersions, restorePromptVersion } from './usePromptLibrary';

interface Props {
  prompt: PromptSummary;
  onClose: () => void;
  onRestored: () => void;
}

interface ListProps {
  versions: PromptVersion[];
  selectedVer: PromptVersion | null;
  onSelect: (v: PromptVersion) => void;
}

const VersionList: React.FC<ListProps> = ({ versions, selectedVer, onSelect }) => (
  <div style={{ borderRight: "1px solid var(--ln)", paddingRight: "var(--sp-12)", overflowY: "auto", display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
    {versions.map((v) => (
      <Button
        variant="ghost"
        type="button"
        key={v.id}
        onClick={() => onSelect(v)}
        style={{
          width: "100%",
          textAlign: "left",
          padding: "var(--sp-8)",
          borderRadius: "var(--radius-4)",
          fontSize: "var(--fs-12)",
          backgroundColor: selectedVer?.id === v.id ? "var(--blb)" : "transparent",
          border: selectedVer?.id === v.id ? "1px solid var(--bl)" : "1px solid transparent",
          color: selectedVer?.id === v.id ? "var(--bl)" : "var(--fg)",
          display: "flex",
          flexDirection: "column",
          alignItems: "flex-start",
        }}
      >
        <div style={{ fontWeight: 600 }}>{t("version_prefix")}{v.version}</div>
        <div style={{ fontSize: "10px", color: "var(--mu)" }}>{new Date(v.created_at).toLocaleString()}</div>
      </Button>
    ))}
  </div>
);

interface DetailProps {
  selectedVer: PromptVersion | null;
  isCurrent: boolean;
  restoring: boolean;
  onRestore: () => void;
}

const VersionDetail: React.FC<DetailProps> = ({ selectedVer, isCurrent, restoring, onRestore }) => {
  if (!selectedVer) return <div style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t('empty_state')}</div>;

  return (
    <div style={{ display: "flex", flexDirection: "column", justifyContent: "space-between", flex: 2 }}>
      <div style={{ flex: 1, overflowY: "auto" }}>
        <div style={{ fontSize: "var(--fs-12)", fontWeight: 600, color: "var(--fg)", marginBottom: "var(--sp-8)" }}>
          {t("version_label")}{selectedVer.version}
        </div>
        <pre style={{ margin: 0, padding: "var(--sp-12)", backgroundColor: "var(--sf2)", borderRadius: "var(--radius-6)", fontSize: "var(--fs-12)", fontFamily: "var(--font-mono)", whiteSpace: "pre-wrap", color: "var(--fg)", border: "1px solid var(--ln)" }}>
          {selectedVer.content}
        </pre>
      </div>
      <div style={{ paddingTop: "var(--sp-12)", borderTop: "1px solid var(--ln)", display: "flex", justifyContent: "flex-end" }}>
        <Button variant="primary" size="sm" type="button" disabled={restoring || isCurrent} loading={restoring} onClick={onRestore} icon={<RotateCcw size={14} />}>{t('prompt_history_restore')}</Button>
      </div>
    </div>
  );
};

function useVersionHistory(promptId: string, onRestored: () => void) {
  const [versions, setVersions] = useState<PromptVersion[]>([]);
  const [selectedVer, setSelectedVer] = useState<PromptVersion | null>(null);
  const [loading, setLoading] = useState(false);
  const [restoring, setRestoring] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    setLoading(true);
    fetchPromptVersions(promptId)
      .then((vers) => {
        setVersions(vers);
        if (vers.length > 0) setSelectedVer(vers[0]);
      })
      .catch((e: unknown) => setErr(e instanceof Error ? e.message : String(e)))
      .finally(() => setLoading(false));
  }, [promptId]);

  const handleRestore = async () => {
    if (!selectedVer) return;
    setRestoring(true);
    setErr(null);
    try {
      await restorePromptVersion(promptId, selectedVer.version);
      onRestored();
    } catch (e: unknown) {
      setErr(e instanceof Error ? e.message : String(e));
    } finally {
      setRestoring(false);
    }
  };

  return { versions, selectedVer, setSelectedVer, loading, restoring, err, handleRestore };
}

export const VersionHistoryModal: React.FC<Props> = ({ prompt, onClose, onRestored }) => {
  const h = useVersionHistory(prompt.id, onRestored);
  const isCurrent = h.selectedVer?.version === prompt.current_version;

  return (
    <Modal open onClose={onClose} title={t('prompt_history_title', { name: prompt.name })} size="lg">
      {h.loading && <p style={{ padding: "var(--sp-16) 0", fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t('loading')}</p>}
      {h.err && <p style={{ padding: "var(--sp-8) 0", fontSize: "var(--fs-12)", color: "var(--rd)" }}>{h.err}</p>}
      {!h.loading && (
        <div style={{ display: "grid", gridTemplateColumns: "1fr 2fr", gap: "var(--sp-16)", height: 320 }}>
          <VersionList versions={h.versions} selectedVer={h.selectedVer} onSelect={h.setSelectedVer} />
          <VersionDetail selectedVer={h.selectedVer} isCurrent={isCurrent} restoring={h.restoring} onRestore={h.handleRestore} />
        </div>
      )}
    </Modal>
  );
};

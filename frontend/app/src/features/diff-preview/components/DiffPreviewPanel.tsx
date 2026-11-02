import React, { useState } from "react";
import { t } from "../../../core/i18n";
import { useDiffPreview } from "../hooks/useDiffPreview";
import { DiffFileList } from "./DiffFileList";
import { DiffFileView } from "./DiffFileView";
import { PreviewControls } from "./PreviewControls";
import { PreviewLogs } from "./PreviewLogs";

interface Props {
  context?: { taskId?: string };
  task?: { id: string };
  taskId?: string;
}

export function DiffPreviewPanel({ context, task, taskId }: Props): React.JSX.Element {
  const effectiveTaskId = context?.taskId || task?.id || taskId || "";
  const [activeTab, setActiveTab] = useState<"diff" | "preview">("diff");
  const [selectedFile, setSelectedFile] = useState<string>("");
  const [confirmed, setConfirmed] = useState<boolean>(false);
  const [port, setPort] = useState<number>(3000);
  const { diff, status, loading, error, handleStart, handleStop } = useDiffPreview(effectiveTaskId);

  const currentFile = diff?.files.find((f) => f.path === (selectedFile || diff.files[0]?.path));

  if (loading) return <div style={{ padding: "16px", color: "var(--mu)" }}>...</div>;
  if (error) return <div style={{ padding: "16px", color: "var(--rd)" }}>{error}</div>;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "12px", padding: "8px" }}>
      <div style={{ display: "flex", gap: "8px", borderBottom: "1px solid var(--ln)", paddingBottom: "8px" }}>
        <button type="button" onClick={() => setActiveTab("diff")} style={{ minHeight: "44px", padding: "8px 16px", border: "none", background: activeTab === "diff" ? "var(--sf2)" : "transparent", fontWeight: 600, cursor: "pointer", borderRadius: "6px" }}>{t("diff_preview_tab_diff")}</button>
        <button type="button" onClick={() => setActiveTab("preview")} style={{ minHeight: "44px", padding: "8px 16px", border: "none", background: activeTab === "preview" ? "var(--sf2)" : "transparent", fontWeight: 600, cursor: "pointer", borderRadius: "6px" }}>{t("diff_preview_tab_preview")}</button>
      </div>
      {activeTab === "diff" ? (
        <div style={{ display: "grid", gridTemplateColumns: "240px 1fr", gap: "16px" }}>
          <DiffFileList files={diff?.files || []} selectedPath={currentFile?.path || ""} onSelect={setSelectedFile} />
          <DiffFileView file={currentFile} />
        </div>
      ) : (
        <div>
          <PreviewControls status={status} confirmed={confirmed} port={port} onConfirmChange={setConfirmed} onPortChange={setPort} onStart={() => void handleStart(port)} onStop={() => void handleStop()} />
          {status?.logs && <PreviewLogs logs={status.logs} />}
        </div>
      )}
    </div>
  );
}

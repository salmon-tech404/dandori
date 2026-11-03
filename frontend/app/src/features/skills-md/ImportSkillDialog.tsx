import React, { useState } from "react";
import { t } from "../../core/i18n";
import { Modal } from "../../ui/components/Modal";
import { Button } from "../../ui/components/Button";
import { Input } from "../../ui/components/Input";
import { importSkillDir, importSkillZip } from "./api";

interface ImportSkillDialogProps {
  workspaceId: string;
  onClose: () => void;
  onSuccess: () => void;
}

function useImportState(workspaceId: string, onSuccess: () => void) {
  const [tab, setTab] = useState<"dir" | "zip">("dir");
  const [dirPath, setDirPath] = useState("");
  const [zipData, setZipData] = useState({ name: "", b64: "" });
  const [importing, setImporting] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      const res = reader.result as string;
      const b64 = res.split(",")[1] || "";
      setZipData({ name: file.name, b64 });
    };
    reader.readAsDataURL(file);
  };

  const executeImport = async () => {
    setImporting(true);
    setErrorMsg(null);
    try {
      if (tab === "dir") {
        await importSkillDir(workspaceId, { dir_path: dirPath });
      } else {
        await importSkillZip(workspaceId, { file_name: zipData.name, base64_content: zipData.b64 });
      }
      onSuccess();
    } catch (err) {
      setErrorMsg(err instanceof Error ? err.message : String(err));
    } finally {
      setImporting(false);
    }
  };

  const canSubmit = !importing && (tab === "dir" ? dirPath.trim().length > 0 : zipData.b64.length > 0);

  return { tab, setTab, dirPath, setDirPath, handleFileChange, executeImport, errorMsg, canSubmit, importing };
}

export function ImportSkillDialog({ workspaceId, onClose, onSuccess }: ImportSkillDialogProps) {
  const state = useImportState(workspaceId, onSuccess);

  return (
    <Modal open onClose={onClose} title={t("skills_import_btn")} size="md">
      <ImportTabs tab={state.tab} onSelectTab={state.setTab} />
      <ImportInput
        tab={state.tab}
        dirPath={state.dirPath}
        onDirChange={state.setDirPath}
        onFileChange={state.handleFileChange}
      />
      {state.errorMsg && <p style={{ marginTop: "var(--sp-8)", fontSize: "var(--fs-12)", color: "var(--rd)" }}>{state.errorMsg}</p>}
      <ImportActions
        onClose={onClose}
        onSubmit={() => void state.executeImport()}
        canSubmit={state.canSubmit}
        importing={state.importing}
      />
    </Modal>
  );
}

function ImportTabs({ tab, onSelectTab }: { tab: "dir" | "zip"; onSelectTab: (t: "dir" | "zip") => void }) {
  return (
    <div style={{ marginTop: "var(--sp-16)", display: "flex", borderBottom: "1px solid var(--ln)" }}>
      <Button
        type="button"
        variant={tab === "dir" ? "primary" : "ghost"}
        size="sm"
        onClick={() => onSelectTab("dir")}
      >
        {t("skills_import_dir")}
      </Button>
      <Button
        type="button"
        variant={tab === "zip" ? "primary" : "ghost"}
        size="sm"
        onClick={() => onSelectTab("zip")}
      >
        {t("skills_import_zip")}
      </Button>
    </div>
  );
}

interface ImportInputProps {
  tab: "dir" | "zip";
  dirPath: string;
  onDirChange: (v: string) => void;
  onFileChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
}

function ImportInput({ tab, dirPath, onDirChange, onFileChange }: ImportInputProps) {
  return (
    <div style={{ marginTop: "var(--sp-16)" }}>
      {tab === "dir" ? (
        <Input
          type="text"
          value={dirPath}
          onChange={(e) => onDirChange(e.target.value)}
          placeholder={t("skills_dir_path_placeholder")}
        />
      ) : (
        <Input type="file" accept=".zip" onChange={onFileChange} />
      )}
    </div>
  );
}

interface ImportActionsProps {
  onClose: () => void;
  onSubmit: () => void;
  canSubmit: boolean;
  importing: boolean;
}

function ImportActions({ onClose, onSubmit, canSubmit, importing }: ImportActionsProps) {
  return (
    <div style={{ marginTop: "var(--sp-24)", display: "flex", justifyContent: "flex-end", gap: "var(--sp-8)" }}>
      <Button type="button" variant="secondary" onClick={onClose}>{t("btn_cancel")}</Button>
      <Button
        type="button"
        variant="primary"
        disabled={!canSubmit}
        onClick={onSubmit}
      >
        {importing ? "..." : t("skills_import_btn")}
      </Button>
    </div>
  );
}

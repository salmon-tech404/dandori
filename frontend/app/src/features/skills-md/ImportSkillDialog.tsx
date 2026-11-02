import React, { useState } from "react";
import { t } from "../../core/i18n";
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
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
      <div className="w-full max-w-md rounded bg-white p-6 shadow-lg">
        <h3 className="text-lg font-bold">{t("skills_import_btn")}</h3>
        <ImportTabs tab={state.tab} onSelectTab={state.setTab} />
        <ImportInput
          tab={state.tab}
          dirPath={state.dirPath}
          onDirChange={state.setDirPath}
          onFileChange={state.handleFileChange}
        />
        {state.errorMsg && <p className="mt-2 text-xs text-red-600">{state.errorMsg}</p>}
        <ImportActions
          onClose={onClose}
          onSubmit={() => void state.executeImport()}
          canSubmit={state.canSubmit}
          importing={state.importing}
        />
      </div>
    </div>
  );
}

function ImportTabs({ tab, onSelectTab }: { tab: "dir" | "zip"; onSelectTab: (t: "dir" | "zip") => void }) {
  return (
    <div className="mt-4 flex border-b">
      <button
        type="button"
        onClick={() => onSelectTab("dir")}
        className={`px-4 py-2 text-sm font-medium ${tab === "dir" ? "border-b-2 border-blue-600 text-blue-600" : "text-gray-500"}`}
      >
        {t("skills_import_dir")}
      </button>
      <button
        type="button"
        onClick={() => onSelectTab("zip")}
        className={`px-4 py-2 text-sm font-medium ${tab === "zip" ? "border-b-2 border-blue-600 text-blue-600" : "text-gray-500"}`}
      >
        {t("skills_import_zip")}
      </button>
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
    <div className="mt-4">
      {tab === "dir" ? (
        <input
          type="text"
          value={dirPath}
          onChange={(e) => onDirChange(e.target.value)}
          placeholder={t("skills_dir_path_placeholder")}
          className="w-full rounded border p-2 text-sm"
        />
      ) : (
        <input type="file" accept=".zip" onChange={onFileChange} className="w-full text-sm" />
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
    <div className="mt-6 flex justify-end space-x-2">
      <button type="button" onClick={onClose} className="rounded border px-4 py-2 text-sm hover:bg-gray-50">
        Hủy
      </button>
      <button
        type="button"
        disabled={!canSubmit}
        onClick={onSubmit}
        className="rounded bg-blue-600 px-4 py-2 text-sm text-white hover:bg-blue-700 disabled:opacity-50"
      >
        {importing ? "Đang nhập..." : "Nhập"}
      </button>
    </div>
  );
}

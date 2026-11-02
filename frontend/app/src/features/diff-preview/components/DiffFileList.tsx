import React from "react";
import type { DiffFile } from "../types";
import { t } from "../../../core/i18n";

interface ItemProps {
  file: DiffFile;
  isSelected: boolean;
  onSelect: (path: string) => void;
}

function DiffFileItem({ file, isSelected, onSelect }: ItemProps): React.JSX.Element {
  return (
    <button
      type="button"
      onClick={() => onSelect(file.path)}
      style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        padding: "8px 12px",
        minHeight: "44px",
        borderRadius: "6px",
        border: isSelected ? "1px solid var(--fg)" : "1px solid var(--ln)",
        background: isSelected ? "var(--sf2)" : "var(--sf)",
        cursor: "pointer",
        textAlign: "left",
      }}
    >
      <span style={{ fontFamily: "monospace", fontSize: "13px", color: "var(--fg)" }}>{file.path}</span>
      <span style={{ fontSize: "12px", display: "flex", gap: "8px" }}>
        {file.is_binary ? (
          <span style={{ color: "var(--mu)" }}>{t("diff_binary_badge")}</span>
        ) : (
          <>
            <span style={{ color: "var(--gr)" }}>+{file.insertions}</span>
            <span style={{ color: "var(--rd)" }}>-{file.deletions}</span>
          </>
        )}
      </span>
    </button>
  );
}

interface Props {
  files: DiffFile[];
  selectedPath: string;
  onSelect: (path: string) => void;
}

export function DiffFileList({ files, selectedPath, onSelect }: Props): React.JSX.Element {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
      {files.map((f) => (
        <DiffFileItem key={f.path} file={f} isSelected={f.path === selectedPath} onSelect={onSelect} />
      ))}
    </div>
  );
}

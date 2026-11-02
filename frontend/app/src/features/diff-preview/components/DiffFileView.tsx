import React from "react";
import type { DiffFile } from "../types";
import { t } from "../../../core/i18n";

interface Props {
  file: DiffFile | undefined;
}

export function DiffFileView({ file }: Props): React.JSX.Element {
  if (!file) {
    return <div style={{ color: "var(--mu)", padding: "16px" }}>{t("diff_no_changes")}</div>;
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
      {file.truncated && (
        <div style={{ padding: "8px 12px", background: "var(--orb)", color: "var(--or)", borderRadius: "6px" }}>
          {t("diff_truncated_banner")}
        </div>
      )}
      {file.is_binary ? (
        <div style={{ padding: "16px", color: "var(--mu)", background: "var(--sf2)", borderRadius: "6px" }}>
          {t("diff_binary_badge")}
        </div>
      ) : (
        <pre
          style={{
            margin: 0,
            padding: "12px",
            background: "var(--sf2)",
            borderRadius: "6px",
            overflowX: "auto",
            fontFamily: "monospace",
            fontSize: "12px",
            lineHeight: "1.5",
            whiteSpace: "pre-wrap",
            wordBreak: "break-all",
          }}
        >
          {file.diff}
        </pre>
      )}
    </div>
  );
}

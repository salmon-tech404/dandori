import React from "react";
import { t } from "../../../core/i18n";

interface Props {
  logs: string[];
}

export function PreviewLogs({ logs }: Props): React.JSX.Element {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "8px", marginTop: "12px" }}>
      <span style={{ fontSize: "13px", fontWeight: 600, color: "var(--fg)" }}>
        {t("diff_preview_logs_title")}
      </span>
      <div
        style={{
          background: "var(--sf2)",
          padding: "12px",
          borderRadius: "6px",
          fontFamily: "monospace",
          fontSize: "12px",
          maxHeight: "260px",
          overflowY: "auto",
          whiteSpace: "pre-wrap",
          wordBreak: "break-all",
        }}
      >
        {logs.length === 0 ? (
          <span style={{ color: "var(--mu)" }}>...</span>
        ) : (
          logs.map((line, idx) => (
            <div key={`${line}-${idx}`} style={{ color: "var(--fg)" }}>{line}</div>
          ))
        )}
      </div>
    </div>
  );
}

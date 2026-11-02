import React from "react";
import type { PreviewStatus } from "../types";
import { t } from "../../../core/i18n";

interface Props {
  status: PreviewStatus | null;
  confirmed: boolean;
  port: number;
  onConfirmChange: (val: boolean) => void;
  onPortChange: (val: number) => void;
  onStart: () => void;
  onStop: () => void;
}

export function PreviewControls(props: Props): React.JSX.Element {
  const isRunning = Boolean(props.status?.running);
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "12px", padding: "16px", background: "var(--sf)", borderRadius: "8px", border: "1px solid var(--ln)" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <span style={{ fontWeight: 600, color: isRunning ? "var(--gr)" : "var(--mu)" }}>
          {isRunning ? `${t("diff_preview_running")} ${props.status?.url}` : t("diff_preview_stopped")}
        </span>
        {isRunning ? (
          <button type="button" onClick={props.onStop} style={{ padding: "8px 16px", minHeight: "44px", borderRadius: "8px", background: "var(--rd)", color: "#fff", border: "none", cursor: "pointer" }}>
            {t("diff_btn_stop_preview")}
          </button>
        ) : (
          <button type="button" onClick={props.onStart} disabled={!props.confirmed} style={{ padding: "8px 16px", minHeight: "44px", borderRadius: "8px", background: props.confirmed ? "var(--ac)" : "var(--gy)", color: "var(--acf)", border: "none", cursor: props.confirmed ? "pointer" : "not-allowed" }}>
            {t("diff_btn_start_preview")}
          </button>
        )}
      </div>
      {!isRunning && (
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <input id="diff-confirm-cb" type="checkbox" checked={props.confirmed} onChange={(e) => props.onConfirmChange(e.target.checked)} style={{ width: "20px", height: "20px" }} />
          <label htmlFor="diff-confirm-cb" style={{ fontSize: "13px", color: "var(--fg)", cursor: "pointer" }}>{t("diff_confirm_checkbox")}</label>
        </div>
      )}
    </div>
  );
}

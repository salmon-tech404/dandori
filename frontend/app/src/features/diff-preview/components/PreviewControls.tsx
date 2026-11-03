import React from "react";
import type { PreviewStatus } from "../types";
import { t } from "../../../core/i18n";
import { Button } from "../../../ui/components/Button";
import { Checkbox } from "../../../ui/components/Checkbox";

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
          <Button variant="danger" type="button" onClick={props.onStop} style={{ padding: "8px 16px", minHeight: "44px", borderRadius: "8px", background: "var(--rd)", color: "#fff", border: "none" }}>
            {t("diff_btn_stop_preview")}
          </Button>
        ) : (
          <Button variant="primary" type="button" onClick={props.onStart} disabled={!props.confirmed} style={{ padding: "8px 16px", minHeight: "44px", borderRadius: "8px", background: props.confirmed ? "var(--ac)" : "var(--gy)", color: "var(--acf)", border: "none" }}>
            {t("diff_btn_start_preview")}
          </Button>
        )}
      </div>
      {!isRunning && (
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <Checkbox id="diff-confirm-cb" checked={props.confirmed} onChange={(e) => props.onConfirmChange(e.target.checked)} label={t("diff_confirm_checkbox")} />

        </div>
      )}
    </div>
  );
}

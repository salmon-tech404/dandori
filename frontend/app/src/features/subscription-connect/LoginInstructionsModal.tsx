import type { ReactElement } from "react";
import type { LoginInstructions } from "./types";
import { t } from "../../core/i18n";

interface LoginInstructionsModalProps {
  instructions: LoginInstructions;
  onClose: () => void;
}

export function LoginInstructionsModal({ instructions, onClose }: LoginInstructionsModalProps): ReactElement {
  return (
    <div className="modal-backdrop">
      <div className="modal-content" style={{ maxWidth: "600px" }}>
        <h3 style={{ margin: "0 0 8px 0", color: "var(--fg)" }}>
          {t("dialog_login_title", { label: instructions.provider.toUpperCase() })}
        </h3>
        <p style={{ fontSize: "13px", color: "var(--mu)", margin: "0 0 16px 0" }}>{t("dialog_login_desc")}</p>
        <div style={{ fontSize: "13px", marginBottom: "12px", color: "var(--fg)" }}>{instructions.instructions}</div>
        <pre
          style={{
            backgroundColor: "var(--bg)",
            border: "1px solid var(--ln)",
            borderRadius: "6px",
            padding: "12px",
            fontFamily: "var(--font-mono, monospace)",
            fontSize: "12px",
            overflowX: "auto",
            whiteSpace: "pre-wrap",
            wordBreak: "break-all",
            color: "var(--fg)",
          }}
        >
          {instructions.command}
        </pre>
        <div style={{ display: "flex", justifyContent: "flex-end", marginTop: "16px" }}>
          <button type="button" className="btn btn-primary" style={{ minHeight: "44px" }} onClick={onClose}>
            {t("btn_close")}
          </button>
        </div>
      </div>
    </div>
  );
}

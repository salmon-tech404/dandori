import type { ReactElement } from "react";
import type { LoginInstructions } from "./types";
import { t } from "../../core/i18n";
import { Modal } from "../../ui/components/Modal";
import { Button } from "../../ui/components/Button";

interface LoginInstructionsModalProps {
  instructions: LoginInstructions;
  onClose: () => void;
}

export function LoginInstructionsModal({ instructions, onClose }: LoginInstructionsModalProps): ReactElement {
  const footer = (
    <div style={{ display: "flex", justifyContent: "flex-end" }}>
      <Button type="button" variant="primary" onClick={onClose}>
        {t("btn_close")}
      </Button>
    </div>
  );

  return (
    <Modal
      open
      onClose={onClose}
      title={t("dialog_login_title", { label: instructions.provider.toUpperCase() })}
      footer={footer}
      size="md"
    >
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
    </Modal>
  );
}

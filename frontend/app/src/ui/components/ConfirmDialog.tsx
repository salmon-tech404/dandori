import { FC, useEffect, useRef, RefObject } from "react";
import { Button } from "./Button";
import { t } from "../../core/i18n";

export interface ConfirmDialogProps {
  open: boolean;
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  isDestructive?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

function useDialogEscape(open: boolean, onCancel: () => void) {
  useEffect(() => {
    if (!open) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onCancel();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [open, onCancel]);
}

interface DialogContentProps {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  isDestructive: boolean;
  confirmRef: RefObject<HTMLButtonElement | null>;
  onConfirm: () => void;
  onCancel: () => void;
}

function DialogContent({ title, message, confirmLabel, cancelLabel, isDestructive, confirmRef, onConfirm, onCancel }: DialogContentProps) {
  return (
    <div
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="dialog-title"
      aria-describedby="dialog-desc"
      style={{ backgroundColor: "var(--sf)", border: "1px solid var(--ln)", borderRadius: "var(--radius-10)", padding: "var(--sp-24)", maxWidth: "420px", width: "90%" }}
      onClick={(e) => e.stopPropagation()}
    >
      <h3 id="dialog-title" style={{ margin: "0 0 var(--sp-8) 0", fontSize: "var(--fs-16)", fontWeight: 600, color: "var(--fg)" }}>{title}</h3>
      <p id="dialog-desc" style={{ margin: "0 0 var(--sp-24) 0", fontSize: "var(--fs-14)", color: "var(--mu)" }}>{message}</p>
      <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--sp-8)" }}>
        <Button variant="secondary" onClick={onCancel}>{cancelLabel || t("btn_cancel")}</Button>
        <Button ref={confirmRef} variant={isDestructive ? "danger" : "primary"} onClick={onConfirm}>{confirmLabel || t("btn_confirm")}</Button>
      </div>
    </div>
  );
}

export const ConfirmDialog: FC<ConfirmDialogProps> = ({ open, title, message, confirmLabel, cancelLabel, isDestructive = false, onConfirm, onCancel }) => {
  const confirmBtnRef = useRef<HTMLButtonElement>(null);
  useDialogEscape(open, onCancel);

  useEffect(() => {
    if (open) confirmBtnRef.current?.focus();
  }, [open]);

  if (!open) return null;

  return (
    <div role="presentation" style={{ position: "fixed", inset: 0, backgroundColor: "rgba(0, 0, 0, 0.45)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000 }} onClick={onCancel}>
      <DialogContent title={title} message={message} confirmLabel={confirmLabel} cancelLabel={cancelLabel} isDestructive={isDestructive} confirmRef={confirmBtnRef} onConfirm={onConfirm} onCancel={onCancel} />
    </div>
  );
};

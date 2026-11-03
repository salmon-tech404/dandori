import { FC, ReactNode, useEffect, useRef, useId, CSSProperties, RefObject } from "react";
import { IconButton } from "./IconButton";
import { t } from "../../core/i18n";

export interface ModalProps {
  open: boolean;
  onClose: () => void;
  title?: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
  size?: "sm" | "md" | "lg" | "full";
  role?: "dialog" | "alertdialog";
  closeOnEscape?: boolean;
  closeOnBackdrop?: boolean;
  ariaLabel?: string;
  ariaLabelledBy?: string;
  className?: string;
  style?: CSSProperties;
  "data-testid"?: string;
}

const SIZE_WIDTHS: Record<NonNullable<ModalProps["size"]>, string> = {
  sm: "400px",
  md: "520px",
  lg: "720px",
  full: "94vw",
};

function useModalScrollLock(open: boolean) {
  useEffect(() => {
    if (!open) return;
    const prev = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => { document.body.style.overflow = prev; };
  }, [open]);
}

function handleTabTrap(e: KeyboardEvent, dialogRef: RefObject<HTMLDivElement | null>) {
  if (e.key !== "Tab" || !dialogRef.current) return;
  const focusables = dialogRef.current.querySelectorAll<HTMLElement>(
    'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
  );
  if (focusables.length === 0) return;
  const first = focusables[0];
  const last = focusables[focusables.length - 1];
  if (e.shiftKey && document.activeElement === first) {
    e.preventDefault();
    last.focus();
  } else if (!e.shiftKey && document.activeElement === last) {
    e.preventDefault();
    first.focus();
  }
}

function useModalFocusTrap(open: boolean, closeOnEscape: boolean, onClose: () => void, dialogRef: RefObject<HTMLDivElement | null>) {
  useEffect(() => {
    if (!open) return;
    const prevActive = document.activeElement as HTMLElement | null;
    const focusables = dialogRef.current?.querySelectorAll<HTMLElement>('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])');
    if (focusables && focusables.length > 0) focusables[0].focus();

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && closeOnEscape) {
        e.stopPropagation();
        onClose();
      } else {
        handleTabTrap(e, dialogRef);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      prevActive?.focus?.();
    };
  }, [open, closeOnEscape, onClose, dialogRef]);
}

function ModalHeader({ title, titleId, onClose }: { title?: ReactNode; titleId?: string; onClose: () => void }) {
  if (!title && !onClose) return null;
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "var(--sp-16) var(--sp-24)", borderBottom: "1px solid var(--ln)", gap: "var(--sp-12)" }}>
      {title && <h2 id={titleId} style={{ margin: 0, fontSize: "var(--fs-16)", fontWeight: 600, color: "var(--fg)" }}>{title}</h2>}
      <IconButton
        aria-label={t("btn_close")}
        variant="ghost"
        size="sm"
        icon={<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>}
        onClick={onClose}
      />
    </div>
  );
}

function ModalFooter({ footer }: { footer?: ReactNode }) {
  if (!footer) return null;
  return (
    <div style={{ padding: "var(--sp-12) var(--sp-24)", borderTop: "1px solid var(--ln)", backgroundColor: "var(--sf2)", borderRadius: "0 0 var(--radius-10) var(--radius-10)", display: "flex", justifyContent: "flex-end", gap: "var(--sp-8)" }}>
      {footer}
    </div>
  );
}

export const Modal: FC<ModalProps> = ({ open, onClose, title, children, footer, size = "md", role = "dialog", closeOnEscape = true, closeOnBackdrop = true, ariaLabel, ariaLabelledBy, className = "", style, "data-testid": testId }) => {
  const generatedTitleId = useId();
  const titleId = ariaLabelledBy || (title ? generatedTitleId : undefined);
  const dialogRef = useRef<HTMLDivElement>(null);

  useModalScrollLock(open);
  useModalFocusTrap(open, closeOnEscape, onClose, dialogRef);

  if (!open) return null;

  return (
    <div role="presentation" data-testid="modal-backdrop" style={{ position: "fixed", inset: 0, backgroundColor: "rgba(0, 0, 0, 0.45)", backdropFilter: "blur(2px)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000, padding: "var(--sp-16)" }} onClick={(e) => { if (closeOnBackdrop && e.target === e.currentTarget) onClose(); }}>
      <div ref={dialogRef} role={role} data-testid={testId} aria-modal="true" aria-label={ariaLabel} aria-labelledby={titleId} tabIndex={-1} style={{ backgroundColor: "var(--sf)", border: "1px solid var(--ln)", borderRadius: "var(--radius-10)", width: "100%", maxWidth: SIZE_WIDTHS[size], maxHeight: "90vh", display: "flex", flexDirection: "column", boxShadow: "0 10px 25px rgba(0, 0, 0, 0.2)", outline: "none", ...style }} className={`dandori-modal ${className}`} onClick={(e) => e.stopPropagation()}>
        <ModalHeader title={title} titleId={titleId} onClose={onClose} />
        <div style={{ padding: "var(--sp-24)", overflowY: "auto", color: "var(--fg)", fontSize: "var(--fs-14)" }}>{children}</div>
        <ModalFooter footer={footer} />
      </div>
    </div>
  );
};

import { FC, useRef, useEffect, RefObject } from "react";
import { Modal } from "./Modal";
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

function ConfirmFooter({
  cancelLabel,
  confirmLabel,
  isDestructive,
  onCancel,
  onConfirm,
  confirmRef,
}: {
  cancelLabel?: string;
  confirmLabel?: string;
  isDestructive: boolean;
  onCancel: () => void;
  onConfirm: () => void;
  confirmRef: RefObject<HTMLButtonElement | null>;
}) {
  return (
    <>
      <Button variant="secondary" onClick={onCancel}>
        {cancelLabel || t("btn_cancel")}
      </Button>
      <Button
        ref={confirmRef}
        variant={isDestructive ? "danger" : "primary"}
        onClick={onConfirm}
      >
        {confirmLabel || t("btn_confirm")}
      </Button>
    </>
  );
}

export const ConfirmDialog: FC<ConfirmDialogProps> = (props) => {
  const confirmRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (props.open) confirmRef.current?.focus();
  }, [props.open]);

  return (
    <Modal
      open={props.open}
      onClose={props.onCancel}
      title={props.title}
      size="sm"
      role="alertdialog"
      footer={
        <ConfirmFooter
          cancelLabel={props.cancelLabel}
          confirmLabel={props.confirmLabel}
          isDestructive={props.isDestructive ?? false}
          onCancel={props.onCancel}
          onConfirm={props.onConfirm}
          confirmRef={confirmRef}
        />
      }
    >
      <p style={{ margin: 0, color: "var(--fg)", fontSize: "var(--fs-14)", lineHeight: 1.5 }}>
        {props.message}
      </p>
    </Modal>
  );
};

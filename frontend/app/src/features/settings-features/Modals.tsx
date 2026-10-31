import { FC } from "react";
import { t } from "../../core/i18n";
import { ConfirmDialog } from "../../ui/components/ConfirmDialog";

export interface ModalState {
  type: "dependencies" | "dependents" | "risk" | "pack" | null;
  targetId?: string;
  names?: string[];
  note?: string;
  onConfirm?: () => void;
}

function renderModalByType(modal: ModalState, onClose: () => void) {
  if (modal.type === "dependencies") {
    return <ConfirmDialog open={true} title={t("modal_enable_dependencies_title")} message={t("modal_enable_dependencies_msg", { modules: (modal.names || []).join(", ") })} onConfirm={modal.onConfirm!} onCancel={onClose} />;
  }
  if (modal.type === "dependents") {
    return <ConfirmDialog open={true} isDestructive={true} title={t("modal_disable_dependents_title")} message={t("modal_disable_dependents_msg", { modules: (modal.names || []).join(", ") })} onConfirm={modal.onConfirm!} onCancel={onClose} />;
  }
  if (modal.type === "risk") {
    return <ConfirmDialog open={true} isDestructive={true} title={t("modal_risk_title")} message={t("modal_risk_msg", { note: modal.note || "" })} onConfirm={modal.onConfirm!} onCancel={onClose} />;
  }
  return <ConfirmDialog open={true} title={t("pack_confirm_title", { name: modal.targetId || "" })} message={t("pack_confirm_desc", { modules: (modal.names || []).join(", ") })} onConfirm={modal.onConfirm!} onCancel={onClose} />;
}

export const ActionModals: FC<{ modal: ModalState; onClose: () => void }> = ({ modal, onClose }) => {
  if (!modal.type || !modal.onConfirm) return null;
  return <>{renderModalByType(modal, onClose)}</>;
};

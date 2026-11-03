import { useState, type ReactElement } from "react";
import { t } from "../../core/i18n";
import { Modal } from "../../ui/components/Modal";
import { Button } from "../../ui/components/Button";
import { Input } from "../../ui/components/Input";
import { Select } from "../../ui/components/Select";

interface AddAccountModalProps {
  onClose: () => void;
  onSubmit: (provider: string, label: string, priority: number) => Promise<void>;
}

const providerOptions = [
  { value: "claude", label: t("account_provider_claude") },
  { value: "agy", label: t("account_provider_agy") },
  { value: "codex", label: t("account_provider_codex") },
];

export function AddAccountModal({ onClose, onSubmit }: AddAccountModalProps): ReactElement {
  const [provider, setProvider] = useState("claude");
  const [label, setLabel] = useState("");
  const [priority, setPriority] = useState(100);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!label.trim()) return;
    void onSubmit(provider, label.trim(), Number(priority));
  };

  const footer = (
    <div style={{ display: "flex", justifyContent: "flex-end", gap: "10px" }}>
      <Button type="button" variant="secondary" onClick={onClose}>{t("btn_cancel")}</Button>
      <Button type="submit" form="add-account-form" variant="primary">{t("btn_confirm")}</Button>
    </div>
  );

  return (
    <Modal open onClose={onClose} title={t("dialog_add_account_title")} footer={footer} size="sm">
      <form id="add-account-form" onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: "14px" }}>
        <div>
          <label style={{ display: "block", fontSize: "13px", marginBottom: "6px", color: "var(--fg)" }}>{t("account_provider")}</label>
          <Select value={provider} onChange={(e) => setProvider(e.target.value)} options={providerOptions} />
        </div>
        <div>
          <label style={{ display: "block", fontSize: "13px", marginBottom: "6px", color: "var(--fg)" }}>{t("account_label")}</label>
          <Input type="text" value={label} onChange={(e) => setLabel(e.target.value)} placeholder={t("account_label_placeholder")} required />
        </div>
        <div>
          <label style={{ display: "block", fontSize: "13px", marginBottom: "6px", color: "var(--fg)" }}>{t("account_priority")}</label>
          <Input type="number" value={priority} onChange={(e) => setPriority(Number(e.target.value))} min={1} max={999} />
        </div>
      </form>
    </Modal>
  );
}

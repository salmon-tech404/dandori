import { FC, useState } from "react";
import { t } from "../../core/i18n";
import { Modal } from "../../ui/components/Modal";
import { Button } from "../../ui/components/Button";
import { Input } from "../../ui/components/Input";
import { Select } from "../../ui/components/Select";
import type { CreateChannelRequest } from "./types";

interface Props {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (data: CreateChannelRequest) => Promise<void>;
}

function useChannelForm(onSubmit: (data: CreateChannelRequest) => Promise<void>, onClose: () => void) {
  const [name, setName] = useState("");
  const [type, setType] = useState("telegram");
  const [secret, setSecret] = useState("");
  const [chatId, setChatId] = useState("");
  const [busy, setBusy] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !secret.trim()) return;
    setBusy(true);
    try {
      await onSubmit({
        name: name.trim(),
        channel_type: type,
        secret: secret.trim(),
        chat_id: type === "telegram" ? chatId.trim() : undefined,
      });
      onClose();
    } finally {
      setBusy(false);
    }
  };

  return { name, setName, type, setType, secret, setSecret, chatId, setChatId, busy, handleSubmit };
}

export const ChannelFormModal: FC<Props> = ({ isOpen, onClose, onSubmit }) => {
  const f = useChannelForm(onSubmit, onClose);
  if (!isOpen) return null;

  const typeOptions = [
    { value: "telegram", label: t("notification_type_telegram") },
    { value: "discord", label: t("notification_type_discord") },
    { value: "slack", label: t("notification_type_slack") },
  ];

  return (
    <Modal
      open={isOpen}
      onClose={onClose}
      title={t("notifications_add_channel")}
      size="sm"
      footer={
        <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--sp-8)" }}>
          <Button variant="ghost" onClick={onClose}>{t("notifications_cancel")}</Button>
          <Button variant="primary" loading={f.busy} onClick={f.handleSubmit}>{t("notifications_save")}</Button>
        </div>
      }
    >
      <form onSubmit={f.handleSubmit} style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)" }}>
        <Input label={t("notifications_channel_name")} value={f.name} onChange={(e) => f.setName(e.target.value)} required />
        <Select label={t("notifications_channel_type")} value={f.type} onChange={(e) => f.setType(e.target.value)} options={typeOptions} />
        <Input label={t("notifications_secret_label")} type="password" value={f.secret} onChange={(e) => f.setSecret(e.target.value)} required />
        {f.type === "telegram" && (
          <Input label={t("notifications_chat_id")} value={f.chatId} onChange={(e) => f.setChatId(e.target.value)} />
        )}
      </form>
    </Modal>
  );
};

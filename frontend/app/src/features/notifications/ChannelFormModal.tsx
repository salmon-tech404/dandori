import { useState } from "react";
import { t } from "../../core/i18n";
import type { CreateChannelRequest } from "./types";

interface Props {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (data: CreateChannelRequest) => Promise<void>;
}

function NameInput({ name, setName }: { name: string; setName: (v: string) => void }) {
  return (
    <div>
      <label className="mb-1 block text-xs text-neutral-300">
        {t("notifications_channel_name")}
      </label>
      <input
        type="text"
        value={name}
        onChange={(e) => setName(e.target.value)}
        className="w-full rounded border border-neutral-700 bg-neutral-800 px-3 py-1.5 text-sm text-white"
        required
      />
    </div>
  );
}

function TypeSelect({ type, setType }: { type: string; setType: (v: string) => void }) {
  return (
    <div>
      <label className="mb-1 block text-xs text-neutral-300">
        {t("notifications_channel_type")}
      </label>
      <select
        value={type}
        onChange={(e) => setType(e.target.value)}
        className="w-full rounded border border-neutral-700 bg-neutral-800 px-3 py-1.5 text-sm text-white"
      >
        <option value="telegram">{t("notification_type_telegram")}</option>
        <option value="discord">{t("notification_type_discord")}</option>
        <option value="slack">{t("notification_type_slack")}</option>
      </select>
    </div>
  );
}

function SecretInput({ secret, setSecret }: { secret: string; setSecret: (v: string) => void }) {
  return (
    <div>
      <label className="mb-1 block text-xs text-neutral-300">
        {t("notifications_secret_label")}
      </label>
      <input
        type="password"
        value={secret}
        onChange={(e) => setSecret(e.target.value)}
        className="w-full rounded border border-neutral-700 bg-neutral-800 px-3 py-1.5 text-sm text-white"
        required
      />
    </div>
  );
}

function ChatIdInput({ chatId, setChatId }: { chatId: string; setChatId: (v: string) => void }) {
  return (
    <div>
      <label className="mb-1 block text-xs text-neutral-300">
        {t("notifications_chat_id")}
      </label>
      <input
        type="text"
        value={chatId}
        onChange={(e) => setChatId(e.target.value)}
        className="w-full rounded border border-neutral-700 bg-neutral-800 px-3 py-1.5 text-sm text-white"
      />
    </div>
  );
}

function FormButtons({ busy, onClose }: { busy: boolean; onClose: () => void }) {
  return (
    <div className="flex justify-end gap-2 pt-2">
      <button
        type="button"
        onClick={onClose}
        className="rounded bg-neutral-700 px-3 py-1.5 text-xs text-neutral-200"
      >
        {t("notifications_cancel")}
      </button>
      <button
        type="submit"
        disabled={busy}
        className="rounded bg-indigo-600 px-4 py-1.5 text-xs font-medium text-white hover:bg-indigo-500 disabled:opacity-50"
      >
        {t("notifications_save")}
      </button>
    </div>
  );
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

export function ChannelFormModal({ isOpen, onClose, onSubmit }: Props) {
  const f = useChannelForm(onSubmit, onClose);
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4">
      <div className="w-full max-w-md rounded-lg border border-neutral-700 bg-neutral-900 p-6">
        <h3 className="mb-4 text-base font-semibold text-white">
          {t("notifications_add_channel")}
        </h3>
        <form onSubmit={f.handleSubmit} className="space-y-4">
          <NameInput name={f.name} setName={f.setName} />
          <TypeSelect type={f.type} setType={f.setType} />
          <SecretInput secret={f.secret} setSecret={f.setSecret} />
          {f.type === "telegram" && <ChatIdInput chatId={f.chatId} setChatId={f.setChatId} />}
          <FormButtons busy={f.busy} onClose={onClose} />
        </form>
      </div>
    </div>
  );
}

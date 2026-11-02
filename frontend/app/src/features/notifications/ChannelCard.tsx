import { useState } from "react";
import { t } from "../../core/i18n";
import type { PublicNotificationChannel } from "./types";

interface Props {
  channel: PublicNotificationChannel;
  onToggle: (id: string, enabled: boolean) => void;
  onTest: (id: string) => void;
  onDelete: (id: string) => void;
}

function ChannelHeader({
  channel,
  onToggle,
}: {
  channel: PublicNotificationChannel;
  onToggle: (id: string, enabled: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between">
      <div>
        <h4 className="font-semibold text-neutral-100">{channel.name}</h4>
        <span className="text-xs uppercase text-neutral-400">
          {channel.channel_type}
        </span>
      </div>
      <button
        type="button"
        onClick={() => onToggle(channel.id, !channel.is_enabled)}
        className={`rounded px-2 py-1 text-xs font-medium ${
          channel.is_enabled
            ? "bg-emerald-800 text-emerald-100"
            : "bg-neutral-600 text-neutral-300"
        }`}
      >
        {channel.is_enabled
          ? t("notifications_status_enabled")
          : t("notifications_status_disabled")}
      </button>
    </div>
  );
}

function ChannelActions({
  channelId,
  onTest,
  onDelete,
}: {
  channelId: string;
  onTest: (id: string) => void;
  onDelete: (id: string) => void;
}) {
  const [busy, setBusy] = useState(false);

  const handleTest = async () => {
    setBusy(true);
    try {
      await onTest(channelId);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="mt-4 flex gap-2">
      <button
        type="button"
        disabled={busy}
        onClick={handleTest}
        className="rounded bg-sky-700 px-3 py-1.5 text-xs font-medium text-white hover:bg-sky-600 disabled:opacity-50"
      >
        {t("notifications_test_send")}
      </button>
      <button
        type="button"
        onClick={() => onDelete(channelId)}
        className="rounded bg-rose-800 px-3 py-1.5 text-xs font-medium text-rose-100 hover:bg-rose-700"
      >
        {t("notifications_delete")}
      </button>
    </div>
  );
}

export function ChannelCard({ channel, onToggle, onTest, onDelete }: Props) {
  return (
    <div className="rounded border border-neutral-700 bg-neutral-800 p-4 shadow-sm">
      <ChannelHeader channel={channel} onToggle={onToggle} />
      <ChannelActions
        channelId={channel.id}
        onTest={onTest}
        onDelete={onDelete}
      />
    </div>
  );
}

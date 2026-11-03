import { useState } from "react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
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
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
      <div>
        <h4 style={{ margin: "0 0 2px 0", fontSize: "var(--fs-14)", fontWeight: 600, color: "var(--fg)" }}>{channel.name}</h4>
        <span style={{ fontSize: "10px", textTransform: "uppercase", color: "var(--mu)", fontWeight: 600 }}>
          {channel.channel_type}
        </span>
      </div>
      <Button variant="ghost" size="sm" type="button" onClick={() => onToggle(channel.id, !channel.is_enabled)}
        style={{
          borderRadius: "var(--radius-pill)",
          padding: "2px 8px",
          fontSize: "var(--fs-12)",
          fontWeight: 600,
          border: "none",
          cursor: "pointer",
          backgroundColor: channel.is_enabled ? "var(--grb)" : "var(--sf2)",
          color: channel.is_enabled ? "var(--gr)" : "var(--mu)",
        }}
      >
        {channel.is_enabled
          ? t("notifications_status_enabled")
          : t("notifications_status_disabled")}
      </Button>
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
    await Promise.resolve(onTest(channelId)).finally(() => setBusy(false));
  };

  return (
    <div style={{ marginTop: "var(--sp-12)", display: "flex", gap: "var(--sp-8)" }}>
      <Button type="button" variant="secondary" size="sm" disabled={busy} loading={busy} onClick={handleTest}>
        {t("notifications_test_send")}
      </Button>
      <Button type="button" variant="ghost" size="sm" onClick={() => onDelete(channelId)} style={{ color: "var(--rd)" }}>
        {t("notifications_delete")}
      </Button>
    </div>
  );
}

export function ChannelCard({ channel, onToggle, onTest, onDelete }: Props) {
  return (
    <div style={{ borderRadius: "var(--radius-8)", border: "1px solid var(--ln)", backgroundColor: "var(--sf2)", padding: "var(--sp-12)" }}>
      <ChannelHeader channel={channel} onToggle={onToggle} />
      <ChannelActions
        channelId={channel.id}
        onTest={onTest}
        onDelete={onDelete}
      />
    </div>
  );
}

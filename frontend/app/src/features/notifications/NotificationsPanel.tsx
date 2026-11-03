import { useState } from "react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { ChannelCard } from "./ChannelCard";
import { ChannelFormModal } from "./ChannelFormModal";
import { NotificationsHistoryTable } from "./NotificationsHistoryTable";
import type { PublicNotificationChannel } from "./types";
import { useNotifications } from "./useNotifications";

interface Props {
  workspaceId: string;
}

function PanelHeader({ onOpenModal }: { onOpenModal: () => void }) {
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "var(--sp-12)" }}>
      <div>
        <h2 style={{ margin: "0 0 2px 0", fontSize: "var(--fs-16)", fontWeight: 600, color: "var(--fg)" }}>{t("notifications_title")}</h2>
        <p style={{ margin: 0, fontSize: "var(--fs-13)", color: "var(--mu)" }}>{t("notifications_desc")}</p>
      </div>
      <Button
        variant="primary"
        size="sm"
        onClick={onOpenModal}
      >
        {t("notifications_add_channel")}
      </Button>
    </div>
  );
}

function TestResultBanner({ result }: { result: string | null }) {
  if (!result) return null;
  const isOk = result === "success";
  const bg = isOk ? "var(--grb)" : "var(--rdb)";
  const color = isOk ? "var(--gr)" : "var(--rd)";
  const msg = isOk ? t("notifications_test_success") : `${t("notifications_test_failed")}: ${result}`;
  return (
    <div style={{ borderRadius: "var(--radius-8)", padding: "var(--sp-8) var(--sp-12)", fontSize: "var(--fs-13)", fontWeight: 500, backgroundColor: bg, color, marginBottom: "var(--sp-16)" }}>
      {msg}
    </div>
  );
}

function ChannelsGrid({
  channels,
  loading,
  onEdit,
  onTest,
  onDelete,
}: {
  channels: PublicNotificationChannel[];
  loading: boolean;
  onEdit: (id: string, enabled: boolean) => void;
  onTest: (id: string) => void;
  onDelete: (id: string) => void;
}) {
  if (loading && channels.length === 0) {
    return <div style={{ padding: "var(--sp-24)", textAlign: "center", fontSize: "var(--fs-13)", color: "var(--mu)" }}>{t("loading")}</div>;
  }
  if (channels.length === 0) {
    return (
      <div style={{ borderRadius: "var(--radius-8)", border: "1px dashed var(--ln)", padding: "var(--sp-24)", textAlign: "center", fontSize: "var(--fs-13)", color: "var(--mu)", marginBottom: "var(--sp-16)" }}>
        {t("notifications_empty")}
      </div>
    );
  }
  return (
    <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))", gap: "var(--sp-12)", marginBottom: "var(--sp-16)" }}>
      {channels.map((ch) => (
        <ChannelCard
          key={ch.id}
          channel={ch}
          onToggle={onEdit}
          onTest={onTest}
          onDelete={onDelete}
        />
      ))}
    </div>
  );
}

function NotificationsHistorySection({ history }: { history: import("./types").NotificationHistoryRecord[] }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
      <h3 style={{ margin: "0 0 var(--sp-4) 0", fontSize: "var(--fs-14)", fontWeight: 600, color: "var(--fg)" }}>
        {t("notifications_history_title")}
      </h3>
      <NotificationsHistoryTable records={history} />
    </div>
  );
}

export function NotificationsPanel({ workspaceId }: Props) {
  const {
    channels,
    history,
    loading,
    testResult,
    addChannel,
    editChannel,
    removeChannel,
    testChannel,
  } = useNotifications(workspaceId);

  const [modalOpen, setModalOpen] = useState(false);

  return (
    <div style={{ backgroundColor: "var(--sf)", borderRadius: "var(--radius-10)", border: "1px solid var(--ln)", padding: "var(--sp-16)", marginBottom: "var(--sp-16)" }}>
      <PanelHeader onOpenModal={() => setModalOpen(true)} />
      <TestResultBanner result={testResult} />
      <ChannelsGrid
        channels={channels}
        loading={loading}
        onEdit={(id, enabled) => editChannel(id, { is_enabled: enabled })}
        onTest={testChannel}
        onDelete={removeChannel}
      />
      <NotificationsHistorySection history={history} />
      {modalOpen && (
        <ChannelFormModal
          isOpen={modalOpen}
          onClose={() => setModalOpen(false)}
          onSubmit={async (data) => {
            await addChannel(data);
            setModalOpen(false);
          }}
        />
      )}
    </div>
  );
}

import { useState } from "react";
import { t } from "../../core/i18n";
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
    <div className="flex items-center justify-between">
      <div>
        <h2 className="text-lg font-bold text-white">{t("notifications_title")}</h2>
        <p className="text-xs text-neutral-400">{t("notifications_desc")}</p>
      </div>
      <button
        type="button"
        onClick={onOpenModal}
        className="rounded bg-indigo-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-indigo-500"
      >
        {t("notifications_add_channel")}
      </button>
    </div>
  );
}

function TestResultBanner({ result }: { result: string | null }) {
  if (!result) return null;
  const isOk = result === "success";
  const cls = isOk ? "bg-emerald-950/80 text-emerald-300" : "bg-rose-950/80 text-rose-300";
  const msg = isOk ? t("notifications_test_success") : `${t("notifications_test_failed")}: ${result}`;
  return <div className={`rounded p-3 text-xs ${cls}`}>{msg}</div>;
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
    return <div className="py-8 text-center text-xs text-neutral-400">{t("loading")}</div>;
  }
  if (channels.length === 0) {
    return (
      <div className="rounded border border-dashed border-neutral-700 py-8 text-center text-xs text-neutral-400">
        {t("notifications_empty")}
      </div>
    );
  }
  return (
    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
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
    <div className="space-y-6 p-4">
      <PanelHeader onOpenModal={() => setModalOpen(true)} />
      <TestResultBanner result={testResult} />
      <ChannelsGrid
        channels={channels}
        loading={loading}
        onEdit={(id, enabled) => editChannel(id, { is_enabled: enabled })}
        onTest={testChannel}
        onDelete={removeChannel}
      />
      <div className="rounded border border-neutral-800 bg-neutral-900/60 p-4">
        <h3 className="mb-3 text-sm font-semibold text-neutral-200">
          {t("notifications_history_title")}
        </h3>
        <NotificationsHistoryTable records={history} />
      </div>
      <ChannelFormModal isOpen={modalOpen} onClose={() => setModalOpen(false)} onSubmit={addChannel} />
    </div>
  );
}

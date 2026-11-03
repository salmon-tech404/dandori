import React, { useEffect, useState } from "react";
import { GitBranch } from "lucide-react";
import { t } from "../../../core/i18n";
import { Button } from "../../../ui/components/Button";
import { Card } from "../../../ui/components/Card";
import { Chip } from "../../../ui/components/Chip";
import { Modal } from "../../../ui/components/Modal";
import { MobileSettingsPanel } from "../../mobile";
import { NotificationsPanel } from "../../notifications";
import { integrationsApi } from "../api/integrationsApi";
import type { PublicIntegrationConnection } from "../types";
import { IntegrationsSettings } from "./IntegrationsSettings";

export interface IntegrationsCardProps {
  workspaceId: string;
}

interface ServiceItem {
  id: string;
  name: string;
  description: string;
  connected: boolean;
  statusText: string;
  btnText: string;
  badge?: string;
  onClick: () => void;
}

function ServiceCard({ item }: { item: ServiceItem }) {
  return (
    <Card style={{ padding: "var(--sp-16)", display: "flex", flexDirection: "column", gap: "var(--sp-12)" }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: "8px" }}>
        <h3 style={{ margin: 0, fontSize: "var(--fs-14)", fontWeight: 700, color: "var(--fg)" }}>{item.name}</h3>
        <Chip variant={item.connected ? "green" : "gray"}>{item.statusText}</Chip>
      </div>
      <p style={{ margin: 0, fontSize: "var(--fs-12)", color: "var(--mu)", flex: 1, minHeight: "36px" }}>
        {item.description}
      </p>
      {item.badge && (
        <div style={{ display: "flex", alignItems: "center", gap: "4px", fontSize: "11px", color: "var(--fg)" }}>
          <GitBranch size={12} style={{ color: "var(--mu)" }} />
          <span>{item.badge}</span>
        </div>
      )}
      {!item.connected && item.id === "github" && (
        <span style={{ fontSize: "11px", color: "var(--mu)" }}>{t("integrations_no_connections")}</span>
      )}
      <div style={{ paddingTop: "var(--sp-4)" }}>
        <Button size="sm" variant={item.connected ? "secondary" : "primary"} onClick={item.onClick}>
          {item.btnText}
        </Button>
      </div>
    </Card>
  );
}

function IntegrationsModals({
  activeModal,
  workspaceId,
  onClose,
}: {
  activeModal: string | null;
  workspaceId: string;
  onClose: () => void;
}) {
  if (!activeModal) return null;
  return (
    <Modal
      open={Boolean(activeModal)}
      onClose={onClose}
      title={activeModal === "git" ? t("integrations_title") : activeModal === "notify" ? t("group_integrations") : t("integ_mobile_title")}
      size="lg"
    >
      {activeModal === "git" && <IntegrationsSettings workspaceId={workspaceId} />}
      {activeModal === "notify" && <NotificationsPanel workspaceId={workspaceId} />}
      {activeModal === "mobile" && <MobileSettingsPanel workspaceId={workspaceId} />}
    </Modal>
  );
}

function buildServices(
  conns: PublicIntegrationConnection[],
  onOpen: (modal: string) => void
): ServiceItem[] {
  const gh = conns.filter((c) => c.provider === "github");
  const gl = conns.filter((c) => c.provider === "gitlab");
  const ln = conns.filter((c) => c.provider === "linear");
  return [
    { id: "github", name: t("integration_provider_github"), description: t("integ_github_desc"), connected: gh.length > 0, statusText: gh.length > 0 ? t("integ_connected") : t("integ_not_connected"), btnText: gh.length > 0 ? t("integ_manage_btn") : t("integ_connect_btn"), badge: gh[0]?.name, onClick: () => onOpen("git") },
    { id: "gitlab", name: t("integration_provider_gitlab"), description: t("integ_gitlab_desc"), connected: gl.length > 0, statusText: gl.length > 0 ? t("integ_connected") : t("integ_not_connected"), btnText: gl.length > 0 ? t("integ_manage_btn") : t("integ_connect_btn"), badge: gl[0]?.name, onClick: () => onOpen("git") },
    { id: "linear", name: t("integration_provider_linear"), description: t("integ_linear_desc"), connected: ln.length > 0, statusText: ln.length > 0 ? t("integ_connected") : t("integ_not_connected"), btnText: ln.length > 0 ? t("integ_manage_btn") : t("integ_connect_btn"), badge: ln[0]?.name, onClick: () => onOpen("git") },
    { id: "telegram", name: t("integ_telegram_title"), description: t("integ_telegram_desc"), connected: false, statusText: t("integ_not_connected"), btnText: t("integ_connect_btn"), onClick: () => onOpen("notify") },
    { id: "discord", name: t("integ_discord_slack_title"), description: t("integ_discord_slack_desc"), connected: false, statusText: t("integ_not_connected"), btnText: t("integ_connect_btn"), onClick: () => onOpen("notify") },
    { id: "mobile", name: t("integ_mobile_title"), description: t("integ_mobile_desc"), connected: false, statusText: t("integ_not_paired"), btnText: t("integ_pair_new_btn"), onClick: () => onOpen("mobile") },
  ];
}

export const IntegrationsCard: React.FC<IntegrationsCardProps> = ({ workspaceId }) => {
  const [conns, setConns] = useState<PublicIntegrationConnection[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeModal, setActiveModal] = useState<string | null>(null);

  useEffect(() => {
    integrationsApi.listConnections(workspaceId).then(setConns).catch(() => {}).finally(() => setLoading(false));
  }, [workspaceId]);

  if (loading) return null;
  const services = buildServices(conns, setActiveModal);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-16)", width: "100%" }}>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(min(100%, 340px), 1fr))", gap: "var(--sp-16)" }}>
        {services.map((item) => (
          <ServiceCard key={item.id} item={item} />
        ))}
      </div>
      <IntegrationsModals activeModal={activeModal} workspaceId={workspaceId} onClose={() => setActiveModal(null)} />
    </div>
  );
};

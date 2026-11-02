import React, { useEffect, useState } from "react";
import { GitBranch, Plug } from "lucide-react";
import { t } from "../../../core/i18n";
import { integrationsApi } from "../api/integrationsApi";
import type { PublicIntegrationConnection } from "../types";

export interface IntegrationsCardProps {
  workspaceId: string;
}

export const IntegrationsCard: React.FC<IntegrationsCardProps> = ({ workspaceId }) => {
  const [conns, setConns] = useState<PublicIntegrationConnection[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    integrationsApi
      .listConnections(workspaceId)
      .then((data) => setConns(data))
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [workspaceId]);

  if (loading) return null;

  return (
    <div className="p-3 border rounded bg-white text-xs space-y-2">
      <div className="flex items-center gap-1.5 font-semibold text-neutral-800">
        <Plug className="w-4 h-4 text-blue-600" />
        <span>{t("integrations_title")}</span>
      </div>
      {conns.length === 0 ? (
        <p className="text-neutral-500">{t("integrations_no_connections")}</p>
      ) : (
        <div className="flex flex-wrap gap-1.5">
          {conns.map((c) => (
            <span
              key={c.id}
              className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-neutral-100 text-neutral-700 font-medium"
            >
              <GitBranch className="w-3 h-3 text-neutral-500" />
              <span>{c.name}</span>
            </span>
          ))}
        </div>
      )}
    </div>
  );
};

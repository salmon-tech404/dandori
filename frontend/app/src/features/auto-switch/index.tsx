import { FC, useEffect, useState } from "react";
import { AutoSwitchNotice } from "./AutoSwitchNotice";
import { AutoSwitchConfigCard } from "./AutoSwitchConfigCard";
import { fetchAutoSwitchConfig, updateAutoSwitchConfig } from "./api";
import type { AutoSwitchConfig, UpdateAutoSwitchConfigPayload } from "./types";

export * from "./types";
export * from "./api";
export * from "./AutoSwitchNotice";
export * from "./AutoSwitchConfigCard";
export * from "./AutoSwitchHistoryList";

export const AutoSwitchSection: FC<{ context?: Record<string, unknown> }> = ({ context }) => {
  const projectId = typeof context?.projectId === "string" ? context.projectId : "";
  const [config, setConfig] = useState<AutoSwitchConfig | null>(null);

  useEffect(() => {
    if (!projectId) return;
    fetchAutoSwitchConfig(projectId).then(setConfig).catch(() => {});
  }, [projectId]);

  const handleSave = async (payload: UpdateAutoSwitchConfigPayload) => {
    if (!projectId) return;
    await updateAutoSwitchConfig(projectId, payload);
    const updated = await fetchAutoSwitchConfig(projectId);
    setConfig(updated);
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)", margin: "var(--sp-12) 0" }}>
      <AutoSwitchNotice />
      {config && <AutoSwitchConfigCard config={config} onSave={handleSave} />}
    </div>
  );
};

export const contributions = {
  "auto-switch-config": AutoSwitchSection,
};

export default contributions;

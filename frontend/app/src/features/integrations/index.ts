import { IntegrationsCard } from "./components/IntegrationsCard";
import { IntegrationsSettings } from "./components/IntegrationsSettings";

export const contributions = {
  "integrations-settings": IntegrationsSettings,
  "integrations-card": IntegrationsCard,
};

export * from "./types";
export * from "./api/integrationsApi";
export { IntegrationsSettings, IntegrationsCard };

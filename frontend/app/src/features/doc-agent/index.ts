import { DocTaskSection } from "./components/DocTaskSection";
import { DocAgentSettings } from "./components/DocAgentSettings";

export const contributions = {
  "doc-agent-task-section": DocTaskSection,
  "doc-agent-project-settings": DocAgentSettings,
};

export * from "./types";
export * from "./api/docAgentApi";
export { DocTaskSection, DocAgentSettings };

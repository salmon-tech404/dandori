import { TestResultBadge } from "./components/TestResultBadge";
import { TestTaskSection } from "./components/TestTaskSection";
import { TestAgentSettings } from "./components/TestAgentSettings";

export const contributions = {
  "test-agent-task-section": TestTaskSection,
  "test-agent-card-badge": TestResultBadge,
  "test-agent-project-settings": TestAgentSettings,
};

export * from "./types";
export * from "./api/testAgentApi";
export { TestResultBadge, TestTaskSection, TestAgentSettings };

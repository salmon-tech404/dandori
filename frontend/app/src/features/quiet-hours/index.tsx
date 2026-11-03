export * from "./types";
export * from "./api";
export * from "./QuietHoursToolbarBadge";
export * from "./QuietHoursSettingsPanel";
export * from "./QuietWindowFormModal";

import { QuietHoursToolbarBadge } from "./QuietHoursToolbarBadge";
import { QuietHoursSettingsPanel } from "./QuietHoursSettingsPanel";

export const contributions = {
  "quiet-hours-toolbar": QuietHoursToolbarBadge,
};

export default {
  QuietHoursToolbarBadge,
  QuietHoursSettingsPanel,
};

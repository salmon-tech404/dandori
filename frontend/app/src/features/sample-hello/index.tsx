import type { ReactElement } from "react";
import { t } from "../../core/i18n";

export function SampleBadge(): ReactElement {
  return <span className="sample-badge">{t("sample_hello_badge")}</span>;
}

export const contributions = {
  "sample-badge": SampleBadge,
};

export default contributions;

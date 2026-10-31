import type { ReactElement } from "react";

export function SampleBadge(): ReactElement {
  return <span className="sample-badge">Mẫu</span>;
}

export const contributions = {
  "sample-badge": SampleBadge,
};

export default contributions;

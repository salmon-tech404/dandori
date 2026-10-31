import { FC, useState, useId } from "react";
import { t } from "../../core/i18n";

export interface TooltipProps {
  content: string;
}

export const Tooltip: FC<TooltipProps> = ({ content }) => {
  const [visible, setVisible] = useState(false);
  const tooltipId = useId();

  return (
    <div style={{ position: "relative", display: "inline-flex", alignItems: "center" }}>
      <button
        type="button"
        aria-describedby={visible ? tooltipId : undefined}
        aria-label={t("tooltip_more_info")}
        onMouseEnter={() => setVisible(true)}
        onMouseLeave={() => setVisible(false)}
        onFocus={() => setVisible(true)}
        onBlur={() => setVisible(false)}
        style={{ display: "inline-flex", alignItems: "center", justifyContent: "center", width: "20px", height: "20px", minWidth: "var(--touch-target)", minHeight: "var(--touch-target)", backgroundColor: "transparent", color: "var(--mu)", border: "none", cursor: "help", padding: 0 }}
      >
        <span style={{ display: "inline-flex", alignItems: "center", justifyContent: "center", width: "18px", height: "18px", borderRadius: "var(--radius-pill)", backgroundColor: "var(--sf2)", border: "1px solid var(--ln)", fontSize: "var(--fs-12)", fontWeight: 600 }}>
          ?
        </span>
      </button>

      {visible && (
        <div id={tooltipId} role="tooltip" style={{ position: "absolute", bottom: "calc(100% + 6px)", left: "50%", transform: "translateX(-50%)", backgroundColor: "var(--fg)", color: "var(--bg)", fontSize: "var(--fs-12)", padding: "4px 8px", borderRadius: "var(--radius-8)", whiteSpace: "nowrap", zIndex: 100, pointerEvents: "none" }}>
          {content}
        </div>
      )}
    </div>
  );
};

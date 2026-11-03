import { FC, useState, useId, ReactNode, cloneElement, isValidElement, CSSProperties } from "react";
import { t } from "../../core/i18n";

export interface TooltipProps {
  content: string;
  children?: ReactNode;
  position?: "top" | "bottom" | "left" | "right";
  className?: string;
  style?: CSSProperties;
}

function getPositionStyle(position: "top" | "bottom" | "left" | "right"): CSSProperties {
  if (position === "bottom") return { top: "calc(100% + 6px)", left: "50%", transform: "translateX(-50%)" };
  if (position === "left") return { right: "calc(100% + 6px)", top: "50%", transform: "translateY(-50%)" };
  if (position === "right") return { left: "calc(100% + 6px)", top: "50%", transform: "translateY(-50%)" };
  return { bottom: "calc(100% + 6px)", left: "50%", transform: "translateX(-50%)" };
}

function TooltipPopup({ id, content, position }: { id: string; content: string; position: "top" | "bottom" | "left" | "right" }) {
  const style: CSSProperties = {
    position: "absolute",
    ...getPositionStyle(position),
    backgroundColor: "var(--fg)",
    color: "var(--bg)",
    fontSize: "var(--fs-12)",
    fontWeight: 500,
    fontFamily: "var(--font-sans)",
    padding: "4px 8px",
    borderRadius: "var(--radius-6)",
    whiteSpace: "nowrap",
    zIndex: 1100,
    pointerEvents: "none",
    boxShadow: "0 2px 8px rgba(0, 0, 0, 0.15)",
  };
  return <span id={id} role="tooltip" style={style}>{content}</span>;
}

function StandaloneTrigger({ tooltipId, visible, onEnter, onLeave }: { tooltipId: string; visible: boolean; onEnter: () => void; onLeave: () => void }) {
  return (
    <button
      type="button"
      aria-describedby={visible ? tooltipId : undefined}
      aria-label={t("tooltip_more_info")}
      onMouseEnter={onEnter}
      onMouseLeave={onLeave}
      onFocus={onEnter}
      onBlur={onLeave}
      style={{ display: "inline-flex", alignItems: "center", justifyContent: "center", width: "20px", height: "20px", minWidth: "var(--touch-target)", minHeight: "var(--touch-target)", backgroundColor: "transparent", color: "var(--mu)", border: "none", cursor: "help", padding: 0 }}
    >
      <span style={{ display: "inline-flex", alignItems: "center", justifyContent: "center", width: "18px", height: "18px", borderRadius: "var(--radius-pill)", backgroundColor: "var(--sf2)", border: "1px solid var(--ln)", fontSize: "var(--fs-12)", fontWeight: 600 }}>
        ?
      </span>
    </button>
  );
}

export const Tooltip: FC<TooltipProps> = ({ content, children, position = "top", className = "", style }) => {
  const [visible, setVisible] = useState(false);
  const tooltipId = useId();
  const onEnter = () => setVisible(true);
  const onLeave = () => setVisible(false);

  if (children && isValidElement(children)) {
    const childEl = children as React.ReactElement<{ "aria-describedby"?: string }>;
    const prevDesc = childEl.props?.["aria-describedby"];
    return (
      <span style={{ position: "relative", display: "inline-flex", alignItems: "center", ...style }} className={`dandori-tooltip-wrapper ${className}`} onMouseEnter={onEnter} onMouseLeave={onLeave} onFocus={onEnter} onBlur={onLeave}>
        {cloneElement(childEl, { "aria-describedby": visible ? tooltipId : prevDesc })}
        {visible && <TooltipPopup id={tooltipId} content={content} position={position} />}
      </span>
    );
  }

  return (
    <div style={{ position: "relative", display: "inline-flex", alignItems: "center", ...style }} className={`dandori-tooltip-wrapper ${className}`}>
      <StandaloneTrigger tooltipId={tooltipId} visible={visible} onEnter={onEnter} onLeave={onLeave} />
      {visible && <TooltipPopup id={tooltipId} content={content} position={position} />}
    </div>
  );
};

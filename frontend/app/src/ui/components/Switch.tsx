import { FC, KeyboardEvent, CSSProperties } from "react";

export interface SwitchProps {
  checked: boolean;
  disabled?: boolean;
  onChange: (checked: boolean) => void;
  "aria-label"?: string;
}

/** Kích thước thanh trượt dạng hình chữ nhật bo góc (không phải hình tròn). */
const TRACK_WIDTH = 40;
const TRACK_HEIGHT = 22;
const KNOB_SIZE = 16;
const KNOB_GAP = (TRACK_HEIGHT - KNOB_SIZE) / 2;

/** Vùng chạm 44px chiều cao nằm ở lớp ngoài trong suốt; phần nhìn thấy luôn là thanh 40x22. */
function getHitAreaStyle(disabled: boolean): CSSProperties {
  return {
    display: "inline-flex",
    alignItems: "center",
    justifyContent: "center",
    minWidth: `${TRACK_WIDTH}px`,
    minHeight: "var(--touch-target)",
    background: "none",
    border: "none",
    padding: 0,
    cursor: disabled ? "not-allowed" : "pointer",
    opacity: disabled ? 0.4 : 1,
    outlineOffset: "2px",
  };
}

function getTrackStyle(checked: boolean): CSSProperties {
  return {
    position: "relative",
    display: "block",
    width: `${TRACK_WIDTH}px`,
    height: `${TRACK_HEIGHT}px`,
    borderRadius: "var(--radius-6)",
    backgroundColor: checked ? "var(--ac)" : "var(--ln)",
    transition: "background-color 0.2s ease",
  };
}

function getKnobStyle(checked: boolean): CSSProperties {
  return {
    position: "absolute",
    top: `${KNOB_GAP}px`,
    left: `${KNOB_GAP}px`,
    width: `${KNOB_SIZE}px`,
    height: `${KNOB_SIZE}px`,
    borderRadius: "var(--radius-4)",
    backgroundColor: "var(--sf)",
    transform: checked ? `translateX(${TRACK_WIDTH - KNOB_SIZE - KNOB_GAP * 2}px)` : "translateX(0)",
    transition: "transform 0.2s ease",
  };
}

/** Công tắc bật/tắt dùng chung toàn ứng dụng (thanh hình chữ nhật bo góc). */
export const Switch: FC<SwitchProps> = ({ checked, disabled = false, onChange, "aria-label": ariaLabel }) => {
  const handleKeyDown = (e: KeyboardEvent) => {
    if (!disabled && (e.key === "Enter" || e.key === " ")) {
      e.preventDefault();
      onChange(!checked);
    }
  };

  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={ariaLabel}
      aria-disabled={disabled}
      disabled={disabled}
      onClick={() => !disabled && onChange(!checked)}
      onKeyDown={handleKeyDown}
      style={getHitAreaStyle(disabled)}
    >
      <span style={getTrackStyle(checked)}>
        <span style={getKnobStyle(checked)} />
      </span>
    </button>
  );
};

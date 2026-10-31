import { FC, KeyboardEvent, CSSProperties } from "react";

export interface SwitchProps {
  checked: boolean;
  disabled?: boolean;
  onChange: (checked: boolean) => void;
  "aria-label"?: string;
}

function getSwitchStyle(checked: boolean, disabled: boolean): CSSProperties {
  return {
    display: "inline-flex",
    alignItems: "center",
    width: "44px",
    height: "24px",
    minHeight: "var(--touch-target)",
    backgroundColor: checked ? "var(--ac)" : "var(--ln)",
    borderRadius: "var(--radius-pill)",
    padding: "2px",
    cursor: disabled ? "not-allowed" : "pointer",
    opacity: disabled ? 0.4 : 1,
    transition: "background-color 0.2s ease",
    outline: "none",
    position: "relative",
  };
}

export const Switch: FC<SwitchProps> = ({ checked, disabled = false, onChange, "aria-label": ariaLabel }) => {
  const handleKeyDown = (e: KeyboardEvent) => {
    if (!disabled && (e.key === "Enter" || e.key === " ")) {
      e.preventDefault();
      onChange(!checked);
    }
  };

  return (
    <div
      role="switch"
      tabIndex={disabled ? -1 : 0}
      aria-checked={checked}
      aria-label={ariaLabel}
      aria-disabled={disabled}
      onClick={() => !disabled && onChange(!checked)}
      onKeyDown={handleKeyDown}
      style={getSwitchStyle(checked, disabled)}
    >
      <span
        style={{
          width: "20px",
          height: "20px",
          backgroundColor: "var(--sf)",
          borderRadius: "var(--radius-pill)",
          transform: checked ? "translateX(20px)" : "translateX(0)",
          transition: "transform 0.2s ease",
        }}
      />
    </div>
  );
};
